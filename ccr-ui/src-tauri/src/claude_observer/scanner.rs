// ─────────────────────────────────────────────────────────────────────
// 端口自 vibe-observer（MIT License）
//   原始路径: ref/repo/vibe-observer/crates/observer-ingest/src/scanner.rs
// 改造点：
//   - 持久化游标使用 `claude_tool_calls_ingest_state` (file_path, mtime_ns,
//     last_offset)，与 vibe-observer 的 `backfill_state` 同形。
//   - 增量按 `last_offset` 截断已读字节，避免重复 parse；mtime 不变直接跳过。
//   - 不再做 ingest_signal / transparency_log；那两张表在本仓不存在。
// ─────────────────────────────────────────────────────────────────────

use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use ccr_db::database::repositories::claude_tool_calls_repo::{self, ToolCallRow};
use rusqlite::Connection;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

use super::jsonl::{ParsedToolEvent, ToolEventKind, parse_line};

/// 增量 ingest 报告
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct IngestReport {
    pub files_scanned: usize,
    pub files_changed: usize,
    pub calls_inserted: usize,
    pub errors: Vec<String>,
}

/// 入口：扫描 `~/.claude/projects/**.jsonl`，把新增 tool_use / tool_result 行
/// 解析并落库到 `claude_tool_calls`。
///
/// 设计：
/// - 单线程串行处理；scanner 总耗时取决于变更文件数与新增行数。
/// - 增量游标：file_path → (mtime_ns, last_offset)。mtime 不变直接跳过；
///   mtime 前进时仅 seek 到 last_offset 后读后半段，避免重复 parse。
/// - tool_result 行不新增 ToolCallRow，而是通过 ON CONFLICT 路径把 success
///   字段回填到既有 tool_use 行（与之前 batch 中同 dedup_key 的 tool_use 行合并）。
pub fn ingest_incremental(conn: &mut Connection) -> IngestReport {
    let mut report = IngestReport::default();

    /* ====================================================================
     * 步骤1：定位 ~/.claude/projects/，不存在直接返回空报告
     * ====================================================================
     */
    let Some(root) = claude_projects_root() else {
        info!("[claude_observer] HOME 不可读，跳过 ingest");
        return report;
    };
    if !root.is_dir() {
        debug!("[claude_observer] {} 不存在，跳过 ingest", root.display());
        return report;
    }

    /* ====================================================================
     * 步骤2：列出全部 .jsonl 文件
     * ====================================================================
     */
    let files = list_jsonl_files(&root);
    info!(
        "[claude_observer] 开始扫描 jsonl 文件: count={}",
        files.len()
    );

    /* ====================================================================
     * 步骤3：逐文件做增量解析与批量 upsert
     * ====================================================================
     */
    for path in &files {
        report.files_scanned += 1;
        match ingest_single_file(conn, path) {
            Ok(Some((rows_pushed, _ev_count))) => {
                report.files_changed += 1;
                report.calls_inserted += rows_pushed;
            }
            Ok(None) => {
                debug!("[claude_observer] 文件未变更, 跳过: {}", path.display());
            }
            Err(error) => {
                warn!(
                    "[claude_observer] 文件解析失败: {} err={error}",
                    path.display()
                );
                report.errors.push(format!("{}: {error}", path.display()));
            }
        }
    }

    info!(
        "[claude_observer] 扫描完成: scanned={} changed={} inserted={} errors={}",
        report.files_scanned,
        report.files_changed,
        report.calls_inserted,
        report.errors.len()
    );

    report
}

/// 单文件增量解析：返回 Some((upserted_rows, parsed_events))，未变更返回 None
fn ingest_single_file(
    conn: &mut Connection,
    path: &Path,
) -> Result<Option<(usize, usize)>, String> {
    /* 步骤A：读 mtime 与游标 */
    let mtime_ns = file_mtime_ns(path)?;
    let cached = claude_tool_calls_repo::get_ingest_state(conn, &path.to_string_lossy())
        .map_err(|e| format!("read ingest_state: {e}"))?;
    let prev_offset = match cached.as_ref() {
        Some(state) if state.file_mtime_ns == mtime_ns => {
            // 文件未变更
            return Ok(None);
        }
        Some(state) => state.last_offset,
        None => 0,
    };

    /* 步骤B：seek 到 prev_offset 读后续行 */
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    let file_len = file
        .metadata()
        .map_err(|e| format!("stat {}: {e}", path.display()))?
        .len();
    // 文件被截断时 last_offset 失效，重新从头扫
    let start = if (prev_offset as u64) <= file_len {
        prev_offset as u64
    } else {
        0
    };
    file.seek(SeekFrom::Start(start))
        .map_err(|e| format!("seek {}: {e}", path.display()))?;
    let reader = BufReader::new(file);

    let project_hint = derive_project_path_hint(path);

    /* 步骤C：逐行解析 */
    let mut events: Vec<ParsedToolEvent> = Vec::new();
    let mut consumed_bytes: u64 = start;
    for line_res in reader.lines() {
        let raw = match line_res {
            Ok(line) => line,
            Err(e) => {
                warn!("[claude_observer] {} 读取行失败: {e}", path.display());
                break;
            }
        };
        // BufRead::lines 会丢弃换行符，回补 1 字节以保持 offset 正确
        consumed_bytes += raw.len() as u64 + 1;
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            continue;
        }
        events.extend(parse_line(trimmed, project_hint.as_deref()));
    }

    /* 步骤D：把解析事件转成 ToolCallRow，tool_result 走回填路径 */
    let rows = build_rows(&events);
    let upserted = if rows.is_empty() {
        0
    } else {
        claude_tool_calls_repo::upsert_batch(conn, &rows)
            .map_err(|e| format!("upsert_batch: {e}"))?
    };

    /* 步骤E：写游标。无论解析是否新增行，都更新 mtime + offset，
     * 避免下次为零变更文件继续重读。
     */
    let new_offset = consumed_bytes.min(file_len) as i64;
    claude_tool_calls_repo::upsert_ingest_state(
        conn,
        &path.to_string_lossy(),
        mtime_ns,
        new_offset,
    )
    .map_err(|e| format!("upsert_ingest_state: {e}"))?;

    Ok(Some((upserted, events.len())))
}

/// 把解析事件折叠成 ToolCallRow：
/// - ToolUse → 一行新数据
/// - ToolResult → 更新已有行的 success 字段（依赖 upsert_batch 的 COALESCE ON CONFLICT）
fn build_rows(events: &[ParsedToolEvent]) -> Vec<ToolCallRow> {
    /* ====================================================================
     * 步骤1：把 tool_use 行编号成 (session_id, seq)
     * ====================================================================
     * vibe-observer 也用「文件内出现顺序」当 seq，本仓沿用：
     * 同一 session_id 下的 tool_use 按出现先后从 1 开始递增。
     */
    use std::collections::HashMap;
    let mut seq_counter: HashMap<String, i64> = HashMap::new();
    // dedup_key (tool_use_id) -> (session_id, seq) 映射，供 tool_result 行回填使用
    let mut id_to_pos: HashMap<String, (String, i64)> = HashMap::new();

    let mut rows: Vec<ToolCallRow> = Vec::new();

    for ev in events {
        match ev.kind {
            ToolEventKind::ToolUse => {
                let seq = {
                    let counter = seq_counter.entry(ev.session_id.clone()).or_insert(0);
                    *counter += 1;
                    *counter
                };
                id_to_pos.insert(ev.dedup_key.clone(), (ev.session_id.clone(), seq));
                rows.push(ToolCallRow {
                    session_id: ev.session_id.clone(),
                    seq,
                    ts: ev.ts.to_rfc3339(),
                    tool_name: ev.tool_name.clone(),
                    success: ev.success,
                    duration_ms: None,
                    cost_usd: None,
                    project_path: ev.project_path.clone(),
                });
            }
            ToolEventKind::ToolResult => {
                // 仅当能命中同 batch 内的 tool_use 时回填 success；
                // 跨文件 / 跨 batch 的 result 当前版本不再额外查询 DB 回填——
                // 这是与 vibe-observer 的实现差异，但实际场景里 tool_use 与
                // tool_result 总是出现在同一 jsonl 文件内，影响可忽略。
                if let Some((sid, seq)) = id_to_pos.get(&ev.dedup_key)
                    && let Some(row) = rows
                        .iter_mut()
                        .find(|r| &r.session_id == sid && r.seq == *seq)
                {
                    row.success = ev.success.or(row.success);
                }
            }
        }
    }

    rows
}

/// `~/.claude/projects/`
fn claude_projects_root() -> Option<PathBuf> {
    Some(dirs::home_dir()?.join(".claude").join("projects"))
}

/// 从 `~/.claude/projects/<encoded-cwd>/<session>.jsonl` 路径反推项目目录字符串。
/// 编码规则参考 vibe-observer：`-` → `/`，且去掉前导 `-`。
/// 仅当 JSONL 行未自带 cwd 时作为回退。
fn derive_project_path_hint(path: &Path) -> Option<String> {
    let parent = path.parent()?;
    let dirname = parent.file_name()?.to_str()?;
    let mut decoded = dirname.replace('-', "/");
    if decoded.starts_with('/') {
        // OK on Unix
    } else if cfg!(windows) {
        // Windows 上 `C--Users-...` 这种格式没有标准化方案，保持原样
    } else {
        decoded.insert(0, '/');
    }
    Some(decoded)
}

fn list_jsonl_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_file()
                && e.path().extension().and_then(|s| s.to_str()) == Some("jsonl")
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn file_mtime_ns(path: &Path) -> Result<i64, String> {
    let meta = std::fs::metadata(path).map_err(|e| format!("stat {}: {e}", path.display()))?;
    let mtime = meta
        .modified()
        .map_err(|e| format!("mtime {}: {e}", path.display()))?;
    let dur = mtime
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("epoch {}: {e}", path.display()))?;
    // i64 完全够放 2262 年以前的纳秒；溢出在此处不可能发生
    Ok(dur.as_nanos() as i64)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use ccr_db::database::migrations::{run_initial_migration, run_migration_v14};
    use std::io::Write;

    fn fresh_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_initial_migration(&conn).unwrap();
        run_migration_v14(&conn).unwrap();
        conn
    }

    #[test]
    fn build_rows_pairs_tool_use_and_result() {
        let events = vec![
            ParsedToolEvent {
                kind: ToolEventKind::ToolUse,
                session_id: "s1".into(),
                ts: chrono::Utc::now(),
                project_path: Some("/repo".into()),
                tool_name: "Bash".into(),
                success: None,
                dedup_key: "tu_1".into(),
            },
            ParsedToolEvent {
                kind: ToolEventKind::ToolResult,
                session_id: "s1".into(),
                ts: chrono::Utc::now(),
                project_path: Some("/repo".into()),
                tool_name: String::new(),
                success: Some(false),
                dedup_key: "tu_1".into(),
            },
        ];
        let rows = build_rows(&events);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].tool_name, "Bash");
        assert_eq!(rows[0].success, Some(false));
    }

    #[test]
    fn ingest_single_file_writes_rows_and_state() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("s1.jsonl");
        let mut f = std::fs::File::create(&p).unwrap();
        writeln!(f, r#"{{"type":"assistant","sessionId":"s1","timestamp":"2026-05-15T10:00:00Z","cwd":"/repo","message":{{"content":[{{"type":"tool_use","id":"tu_1","name":"Read","input":{{}}}}]}}}}"#).unwrap();
        writeln!(f, r#"{{"type":"user","sessionId":"s1","timestamp":"2026-05-15T10:00:01Z","message":{{"role":"user","content":[{{"type":"tool_result","tool_use_id":"tu_1","is_error":false}}]}}}}"#).unwrap();
        drop(f);

        let mut conn = fresh_conn();
        let out = ingest_single_file(&mut conn, &p).unwrap();
        assert!(out.is_some());
        let (upserted, _) = out.unwrap();
        assert_eq!(upserted, 1);

        let total = claude_tool_calls_repo::total_count(&conn).unwrap();
        assert_eq!(total, 1);

        // 第二次 ingest 同文件应直接跳过
        let again = ingest_single_file(&mut conn, &p).unwrap();
        assert!(again.is_none());
    }
}
