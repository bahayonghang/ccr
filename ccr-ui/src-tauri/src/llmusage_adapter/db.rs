use chrono::NaiveDate;
use rusqlite::{Connection, OpenFlags, ToSql, params_from_iter};

use super::{
    AppPaths, FeatureKey,
    capabilities::{
        MIN_SUPPORTED_SCHEMA_VERSION, ensure_feature, read_schema_version, table_exists,
    },
    error::LlmusageAdapterError,
    queries::{
        DailyTrendDto, HeatmapPoint, HomeOverviewPayload, HomeOverviewPlatformStats,
        HomeOverviewSeriesItem, HomeOverviewSummary, ModelBreakdown, OverviewPayload,
        ProjectBreakdown, SourceBreakdownDto, TokenSummary, UsageRecordDto,
    },
    source::{SourceKind, parse_source_filter},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ReportTimezone {
    #[default]
    Local,
    #[allow(dead_code)]
    Utc,
}

#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    pub source: Option<SourceKind>,
    pub model: Option<String>,
    pub since: Option<NaiveDate>,
    pub until: Option<NaiveDate>,
    pub project_hash: Option<String>,
    pub timezone: ReportTimezone,
}

#[derive(Debug, Clone)]
struct SqlFilter {
    clauses: Vec<String>,
    params: Vec<String>,
}

impl SqlFilter {
    fn new() -> Self {
        Self {
            clauses: Vec::new(),
            params: Vec::new(),
        }
    }

    fn push(&mut self, clause: impl Into<String>, value: impl Into<String>) {
        self.clauses.push(clause.into());
        self.params.push(value.into());
    }

    fn push_raw(&mut self, clause: impl Into<String>) {
        self.clauses.push(clause.into());
    }

    fn where_sql(&self) -> String {
        if self.clauses.is_empty() {
            String::new()
        } else {
            format!(" WHERE {}", self.clauses.join(" AND "))
        }
    }

    fn param_refs(&self) -> Vec<&dyn ToSql> {
        self.params
            .iter()
            .map(|value| value as &dyn ToSql)
            .collect()
    }
}

impl QueryFilter {
    fn local_time_modifier(&self) -> &'static str {
        match self.timezone {
            ReportTimezone::Local => "localtime",
            ReportTimezone::Utc => "utc",
        }
    }

    fn bucket_filter(&self, alias: Option<&str>) -> SqlFilter {
        let prefix = alias.map(|value| format!("{value}.")).unwrap_or_default();
        let mut filter = SqlFilter::new();
        if let Some(source) = self.source {
            filter.push(format!("{prefix}source = ?"), source.as_str());
        }
        if let Some(model) = self.model.as_ref() {
            filter.push(format!("{prefix}model = ?"), model);
        }
        if let Some(since) = self.since {
            filter.push(
                format!(
                    "date({prefix}hour_start, '{}') >= ?",
                    self.local_time_modifier()
                ),
                since.to_string(),
            );
        }
        if let Some(until) = self.until {
            filter.push(
                format!(
                    "date({prefix}hour_start, '{}') <= ?",
                    self.local_time_modifier()
                ),
                until.to_string(),
            );
        }
        if let Some(project_hash) = self.project_hash.as_ref() {
            filter.push(format!("{prefix}project_hash = ?"), project_hash);
        }
        filter
    }

    fn event_filter(&self, alias: Option<&str>) -> SqlFilter {
        let prefix = alias.map(|value| format!("{value}.")).unwrap_or_default();
        let mut filter = SqlFilter::new();
        if let Some(source) = self.source {
            filter.push(format!("{prefix}source = ?"), source.as_str());
        }
        if let Some(model) = self.model.as_ref() {
            filter.push(format!("{prefix}model = ?"), model);
        }
        if let Some(since) = self.since {
            filter.push(
                format!(
                    "date({prefix}event_at, '{}') >= ?",
                    self.local_time_modifier()
                ),
                since.to_string(),
            );
        }
        if let Some(until) = self.until {
            filter.push(
                format!(
                    "date({prefix}event_at, '{}') <= ?",
                    self.local_time_modifier()
                ),
                until.to_string(),
            );
        }
        if let Some(project_hash) = self.project_hash.as_ref() {
            filter.push(format!("{prefix}project_hash = ?"), project_hash);
        }
        filter
    }

    fn without_source(&self) -> Self {
        Self {
            source: None,
            model: self.model.clone(),
            since: self.since,
            until: self.until,
            project_hash: self.project_hash.clone(),
            timezone: self.timezone,
        }
    }
}

pub fn build_filter(
    platform: Option<String>,
    model: Option<String>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<QueryFilter, String> {
    Ok(QueryFilter {
        source: platform.as_deref().and_then(parse_source_filter),
        model: model.and_then(non_empty_string),
        since: parse_optional_date(start_date.as_deref(), "start_date")?,
        until: parse_optional_date(end_date.as_deref(), "end_date")?,
        project_hash: None,
        timezone: ReportTimezone::Local,
    })
}

fn parse_optional_date(value: Option<&str>, label: &str) -> Result<Option<NaiveDate>, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(Some)
        .map_err(|error| format!("Invalid {label} '{value}': {error}"))
}

fn non_empty_string(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[derive(Debug)]
pub struct Dashboard {
    paths: AppPaths,
    conn: Connection,
}

pub fn open_dashboard(paths: AppPaths) -> Result<Dashboard, LlmusageAdapterError> {
    Dashboard::open(paths)
}

impl Dashboard {
    pub fn open(paths: AppPaths) -> Result<Self, LlmusageAdapterError> {
        if !paths.db_path.is_file() {
            return Err(LlmusageAdapterError::DbMissing(paths.db_path.clone()));
        }
        let conn = Connection::open_with_flags(
            &paths.db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(|error| LlmusageAdapterError::DbUnreadable(error.to_string()))?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        let schema_version = read_schema_version(&conn)?;
        if schema_version.unwrap_or_default() < MIN_SUPPORTED_SCHEMA_VERSION {
            return Err(LlmusageAdapterError::SchemaUnsupported {
                expected: MIN_SUPPORTED_SCHEMA_VERSION,
                actual: schema_version,
            });
        }
        Ok(Self { paths, conn })
    }

    pub fn overview(&self, filter: &QueryFilter) -> Result<OverviewPayload, LlmusageAdapterError> {
        ensure_feature(&self.paths, FeatureKey::Overview)?;
        let total = self.query_token_summary(filter, None)?;
        let cutoff = (chrono::Utc::now() - chrono::Duration::hours(24)).to_rfc3339();
        let last_24h = self.query_token_summary(filter, Some(&cutoff))?;
        let total_events = self.query_event_count(filter, None)?;
        let last_24h_events = self.query_event_count(filter, Some(&cutoff))?;
        let total_cost_usd = self.query_cost_with_cache(filter, None)?;
        let bucket_filter = filter.bucket_filter(None);
        let source_count = scalar_i64(
            &self.conn,
            &format!(
                "SELECT COUNT(DISTINCT source) FROM usage_bucket_30m{}",
                bucket_filter.where_sql()
            ),
            &bucket_filter.param_refs(),
        )?;
        let bucket_count = scalar_i64(
            &self.conn,
            &format!(
                "SELECT COUNT(*) FROM usage_bucket_30m{}",
                bucket_filter.where_sql()
            ),
            &bucket_filter.param_refs(),
        )?;
        // last_sync_at / last_export_at 仅作展示字段，遇到 DB 层意外错误时降级为 None
        // 但记 warn 以便排查；以前用 unwrap_or(None) 会把任何 rusqlite::Error 都当无数据处理。
        let last_sync_at = scalar_optional_string(
            &self.conn,
            "SELECT MAX(finished_at) FROM run_log WHERE command IN ('sync', 'hook-run') AND status = 'success'",
            &[],
        )
        .unwrap_or_else(|error| {
            tracing::warn!(?error, "llmusage overview: last_sync_at query failed");
            None
        });
        let last_export_at = scalar_optional_string(
            &self.conn,
            "SELECT MAX(finished_at) FROM run_log WHERE command = 'export html' AND status = 'success'",
            &[],
        )
        .unwrap_or_else(|error| {
            tracing::warn!(?error, "llmusage overview: last_export_at query failed");
            None
        });
        Ok(OverviewPayload {
            generated_at: super::queries::generated_at(),
            cache_efficiency: total.cache_efficiency(),
            total,
            last_24h,
            source_count,
            bucket_count,
            total_events,
            last_24h_events,
            total_cost_usd,
            last_sync_at,
            last_export_at,
        })
    }

    pub fn trends_daily(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<DailyTrendDto>, LlmusageAdapterError> {
        ensure_feature(&self.paths, FeatureKey::DailyTrends)?;
        let sql_filter = filter.bucket_filter(None);
        let modifier = filter.local_time_modifier();
        let sql = format!(
            r#"
            SELECT
                date(hour_start, '{modifier}') AS local_date,
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(cache_read_tokens), 0),
                COALESCE(SUM(cache_creation_tokens), 0),
                COALESCE(SUM(output_tokens), 0) + COALESCE(SUM(reasoning_output_tokens), 0),
                COALESCE(SUM(reasoning_output_tokens), 0),
                COALESCE(SUM(total_tokens), 0),
                COALESCE(SUM(event_count), 0),
                COALESCE(SUM(cost_with_cache_usd), 0.0)
            FROM usage_bucket_30m
            {}
            GROUP BY local_date
            ORDER BY local_date ASC
        "#,
            sql_filter.where_sql()
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(sql_filter.param_refs()), |row| {
            Ok(DailyTrendDto {
                date: row.get(0)?,
                input_tokens: row.get(1)?,
                cache_read_tokens: row.get(2)?,
                cache_creation_tokens: row.get(3)?,
                output_tokens: row.get(4)?,
                reasoning_output_tokens: row.get(5)?,
                total_tokens: row.get(6)?,
                request_count: row.get(7)?,
                cost_usd: row.get(8)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn model_breakdown(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<ModelBreakdown>, LlmusageAdapterError> {
        ensure_feature(&self.paths, FeatureKey::ModelBreakdown)?;
        let sql_filter = filter.bucket_filter(None);
        let sql = format!(
            r#"
            SELECT
                model,
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(cache_read_tokens), 0),
                COALESCE(SUM(cache_creation_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(reasoning_output_tokens), 0),
                COALESCE(SUM(total_tokens), 0),
                COALESCE(SUM(event_count), 0),
                COALESCE(SUM(cost_with_cache_usd), 0.0),
                COALESCE(SUM(cost_without_cache_usd), 0.0),
                CASE WHEN COUNT(DISTINCT pricing_status) = 1 THEN MAX(pricing_status) ELSE 'mixed' END,
                CASE WHEN COUNT(DISTINCT COALESCE(pricing_source, '__llmusage_null__')) = 1 THEN MAX(pricing_source) ELSE 'mixed' END,
                CASE WHEN COUNT(DISTINCT COALESCE(pricing_rate, '__llmusage_null__')) = 1 THEN MAX(pricing_rate) ELSE 'mixed' END
            FROM usage_bucket_30m
            {}
            GROUP BY model
            ORDER BY SUM(total_tokens) DESC, model ASC
        "#,
            sql_filter.where_sql()
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(sql_filter.param_refs()), |row| {
            let cost_with_cache_usd: f64 = row.get(8)?;
            let cost_without_cache_usd: f64 = row.get(9)?;
            Ok(ModelBreakdown {
                model: row.get(0)?,
                input_tokens: row.get(1)?,
                cache_read_tokens: row.get(2)?,
                cache_creation_tokens: row.get(3)?,
                output_tokens: row.get(4)?,
                reasoning_output_tokens: row.get(5)?,
                total_tokens: row.get(6)?,
                event_count: row.get(7)?,
                cost_with_cache_usd,
                cost_without_cache_usd,
                cache_savings_usd: (cost_without_cache_usd - cost_with_cache_usd).max(0.0),
                pricing_status: row
                    .get::<_, Option<String>>(10)?
                    .unwrap_or_else(|| "unpriced".to_string()),
                pricing_source: row.get(11)?,
                pricing_rate: row.get(12)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn project_breakdown(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<ProjectBreakdown>, LlmusageAdapterError> {
        ensure_feature(&self.paths, FeatureKey::ProjectBreakdown)?;
        let mut sql_filter = filter.bucket_filter(None);
        sql_filter.push_raw("project_hash <> ''");
        // 老版本 llmusage 的 usage_bucket_30m 没有 project_path 列，select 时改 NULL；
        // 之前用 sql.replace 容易因换行/空格漂移失效，改成在拼 SQL 时直接选定表达式。
        let has_project_path =
            super::capabilities::column_exists(&self.conn, "usage_bucket_30m", "project_path")
                .unwrap_or(false);
        let project_path_expr = if has_project_path {
            "MAX(project_path)"
        } else {
            "NULL"
        };
        let sql = format!(
            r#"
            SELECT
                project_hash,
                MAX(project_label),
                MAX(project_ref),
                SUM(total_tokens),
                SUM(event_count),
                SUM(cost_with_cache_usd),
                {project_path_expr}
            FROM usage_bucket_30m
            {where_clause}
            GROUP BY project_hash
            ORDER BY SUM(total_tokens) DESC, MAX(project_label) ASC
        "#,
            where_clause = sql_filter.where_sql()
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(sql_filter.param_refs()), |row| {
            let project_label = row
                .get::<_, Option<String>>(1)?
                .unwrap_or_else(|| "unknown-project".to_string());
            let project_ref: Option<String> = row.get(2)?;
            let project_path: Option<String> = row.get(6)?;
            Ok(ProjectBreakdown {
                project_hash: row.get(0)?,
                project_label: project_label.clone(),
                project_ref: project_ref.clone(),
                total_tokens: row.get::<_, Option<i64>>(3)?.unwrap_or_default(),
                event_count: row.get::<_, Option<i64>>(4)?.unwrap_or_default(),
                total_cost_usd: row.get::<_, Option<f64>>(5)?.unwrap_or_default(),
                project_path: project_path.or(project_ref).or(Some(project_label)),
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn source_breakdown(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<SourceBreakdownDto>, LlmusageAdapterError> {
        ensure_feature(&self.paths, FeatureKey::HomeOverview)?;
        let source_filter = filter.without_source();
        let sql_filter = source_filter.bucket_filter(None);
        let modifier = source_filter.local_time_modifier();
        let sql = format!(
            r#"
            SELECT
                source,
                COALESCE(SUM(event_count), 0),
                COALESCE(SUM(total_tokens), 0),
                COALESCE(SUM(cost_with_cache_usd), 0.0),
                COUNT(DISTINCT date(hour_start, '{modifier}'))
            FROM usage_bucket_30m
            {}
            GROUP BY source
            ORDER BY SUM(total_tokens) DESC, source ASC
        "#,
            sql_filter.where_sql()
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let raw_rows = stmt
            .query_map(params_from_iter(sql_filter.param_refs()), |row| {
                Ok(SourceBreakdownDto {
                    source: row.get(0)?,
                    event_count: row.get(1)?,
                    total_tokens: row.get(2)?,
                    total_cost: row.get(3)?,
                    active_days: row.get(4)?,
                    share_tokens: 0.0,
                    share_cost: 0.0,
                })
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;

        let canonical_rows = raw_rows
            .into_iter()
            .filter(|row| {
                matches!(
                    row.source.as_str(),
                    "claude" | "codex" | "gemini" | "opencode"
                )
            })
            .collect::<Vec<_>>();
        let total_tokens: i64 = canonical_rows.iter().map(|row| row.total_tokens).sum();
        let total_cost: f64 = canonical_rows.iter().map(|row| row.total_cost).sum();

        Ok(canonical_rows
            .into_iter()
            .map(|mut row| {
                row.share_tokens = if total_tokens > 0 {
                    row.total_tokens as f64 / total_tokens as f64
                } else {
                    0.0
                };
                row.share_cost = if total_cost > 0.0 {
                    row.total_cost / total_cost
                } else {
                    0.0
                };
                row
            })
            .collect())
    }

    pub fn heatmap(
        &self,
        filter: &QueryFilter,
        days: u32,
    ) -> Result<Vec<HeatmapPoint>, LlmusageAdapterError> {
        ensure_feature(&self.paths, FeatureKey::Heatmap)?;
        let days = days.clamp(1, 366);
        let end = chrono::Local::now().date_naive();
        let start = end - chrono::Duration::days((days - 1) as i64);
        // 用 heatmap 自身的滚动窗口覆盖 caller 透传的 since/until，
        // 避免 bucket_filter() 在 WHERE 里推出两份 date(...) 约束。
        // source/model/project_hash/timezone 仍来自 caller。
        let scoped_filter = QueryFilter {
            since: Some(start),
            until: Some(end),
            ..filter.clone()
        };
        let sql_filter = scoped_filter.bucket_filter(None);
        let modifier = scoped_filter.local_time_modifier();
        let sql = format!(
            r#"
            SELECT date(hour_start, '{modifier}') AS date, COALESCE(SUM(event_count), 0)
            FROM usage_bucket_30m
            {}
            GROUP BY date
            ORDER BY date ASC
        "#,
            sql_filter.where_sql()
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(sql_filter.param_refs()), |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?;
        let mut map = std::collections::HashMap::new();
        for row in rows {
            let (date, value) = row?;
            map.insert(date, value);
        }
        let mut points = Vec::new();
        for offset in 0..days {
            let date = start + chrono::Duration::days(offset as i64);
            let key = date.format("%Y-%m-%d").to_string();
            points.push(HeatmapPoint {
                date: key.clone(),
                event_count: *map.get(&key).unwrap_or(&0),
            });
        }
        Ok(points)
    }

    pub fn logs(&self, query: &LogsQuery) -> Result<LogsPage, LlmusageAdapterError> {
        ensure_feature(&self.paths, FeatureKey::Logs)?;
        let mut sql_filter = query.filter.event_filter(Some("e"));
        if let Some(cursor) = query.cursor.as_ref().and_then(|value| decode_cursor(value)) {
            sql_filter.push_raw("(e.event_at < ? OR (e.event_at = ? AND e.event_key < ?))");
            sql_filter.params.push(cursor.0.clone());
            sql_filter.params.push(cursor.0);
            sql_filter.params.push(cursor.1);
        }
        let raw_join = if table_exists(&self.conn, "usage_event_raw").unwrap_or(false) {
            "LEFT JOIN usage_event_raw raw ON raw.event_key = e.event_key"
        } else {
            ""
        };
        let raw_expr = if query.include_raw_json && !raw_join.is_empty() {
            "raw.raw_json"
        } else {
            "NULL"
        };
        let sql = format!(
            r#"
            SELECT
                e.event_key,
                e.source,
                COALESCE(e.project_path, e.project_ref, e.project_label, e.project_hash, ''),
                COALESCE({raw_expr}, ''),
                e.event_at,
                e.source,
                e.model,
                e.input_tokens,
                e.output_tokens + e.reasoning_output_tokens,
                e.cache_read_tokens,
                e.cache_creation_tokens,
                e.cost_with_cache_usd,
                e.cost_without_cache_usd,
                e.pricing_status,
                e.pricing_source,
                e.event_at,
                e.event_key
            FROM usage_event e
            {raw_join}
            {}
            ORDER BY e.event_at DESC, e.event_key DESC
            LIMIT ?
        "#,
            sql_filter.where_sql()
        );
        let mut params = sql_filter.params.clone();
        params.push((query.page_size + 1).to_string());
        let param_refs: Vec<&dyn ToSql> = params.iter().map(|value| value as &dyn ToSql).collect();
        let mut stmt = self.conn.prepare(&sql)?;
        let mut rows = stmt
            .query_map(params_from_iter(param_refs), map_usage_record)?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let has_next = rows.len() > query.page_size as usize;
        if has_next {
            rows.truncate(query.page_size as usize);
        }
        let next_cursor = if has_next {
            rows.last()
                .map(|row| encode_cursor(&row.recorded_at, &row.id))
        } else {
            None
        };
        let total = if query.include_total {
            let count_filter = query.filter.event_filter(Some("e"));
            Some(scalar_i64(
                &self.conn,
                &format!(
                    "SELECT COUNT(*) FROM usage_event e{}",
                    count_filter.where_sql()
                ),
                &count_filter.param_refs(),
            )?)
        } else {
            None
        };
        Ok(LogsPage {
            records: rows,
            total,
            next_cursor,
        })
    }

    pub fn diagnostics(&self) -> Result<DiagnosticsPayload, LlmusageAdapterError> {
        ensure_feature(&self.paths, FeatureKey::Diagnostics)?;
        let sql = r#"
            SELECT
                f.source,
                SUM(CASE WHEN f.state = 'live' THEN 1 ELSE 0 END),
                SUM(CASE WHEN f.state = 'missing' THEN 1 ELSE 0 END),
                SUM(CASE WHEN f.state = 'deleted_by_user' THEN 1 ELSE 0 END),
                s.recent_completed_at,
                s.history_completed_at
            FROM source_file f
            LEFT JOIN source_sync_status s ON s.source = f.source
            GROUP BY f.source, s.recent_completed_at, s.history_completed_at
            ORDER BY f.source ASC
        "#;
        let mut stmt = self.conn.prepare(sql)?;
        let rows = stmt.query_map([], |row| {
            Ok(SourceDiagnostics {
                source: row.get(0)?,
                live_files: row.get::<_, Option<i64>>(1)?.unwrap_or_default().max(0) as u64,
                missing_files: row.get::<_, Option<i64>>(2)?.unwrap_or_default().max(0) as u64,
                deleted_files: row.get::<_, Option<i64>>(3)?.unwrap_or_default().max(0) as u64,
                recent_completed_at: row.get(4)?,
                history_completed_at: row.get(5)?,
            })
        })?;
        Ok(DiagnosticsPayload {
            archive_root: self.paths.root_dir.display().to_string(),
            by_source: rows.collect::<rusqlite::Result<Vec<_>>>()?,
        })
    }

    pub fn home_overview(
        &self,
        filter: &QueryFilter,
    ) -> Result<HomeOverviewPayload, LlmusageAdapterError> {
        ensure_feature(&self.paths, FeatureKey::HomeOverview)?;
        let trends = self.trends_daily(filter)?;
        let mut by_platform = std::collections::BTreeMap::new();
        for source in ["claude", "codex", "gemini", "opencode"] {
            by_platform.insert(source.to_string(), HomeOverviewPlatformStats::default());
        }
        let mut by_day = std::collections::BTreeMap::<
            String,
            std::collections::BTreeMap<String, HomeOverviewPlatformStats>,
        >::new();

        let sql_filter = filter.bucket_filter(None);
        let modifier = filter.local_time_modifier();
        let sql = format!(
            r#"
            SELECT date(hour_start, '{modifier}'), source, COALESCE(SUM(event_count), 0), COALESCE(SUM(total_tokens), 0)
            FROM usage_bucket_30m
            {}
            GROUP BY date(hour_start, '{modifier}'), source
            ORDER BY 1 ASC, 2 ASC
        "#,
            sql_filter.where_sql()
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(sql_filter.param_refs()), |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
            ))
        })?;
        for row in rows {
            let (date, source, requests, tokens) = row?;
            let stats = HomeOverviewPlatformStats {
                sessions: 0,
                requests,
                tokens,
            };
            if let Some(total) = by_platform.get_mut(&source) {
                total.requests += requests;
                total.tokens += tokens;
            }
            by_day.entry(date).or_default().insert(source, stats);
        }
        let series = by_day
            .into_iter()
            .map(|(date, mut day)| HomeOverviewSeriesItem {
                date,
                claude: day.remove("claude").unwrap_or_default(),
                codex: day.remove("codex").unwrap_or_default(),
                gemini: day.remove("gemini").unwrap_or_default(),
                opencode: day.remove("opencode").unwrap_or_default(),
            })
            .collect::<Vec<_>>();
        let total_requests = by_platform.values().map(|stats| stats.requests).sum();
        let total_tokens = by_platform.values().map(|stats| stats.tokens).sum();
        let platforms = by_platform
            .values()
            .filter(|stats| stats.requests > 0 || stats.tokens > 0)
            .count() as i64;
        Ok(HomeOverviewPayload {
            summary: HomeOverviewSummary {
                total_sessions: 0,
                total_requests,
                total_tokens,
                active_days: trends.len() as i64,
                platforms,
            },
            by_platform,
            series,
        })
    }

    fn query_token_summary(
        &self,
        filter: &QueryFilter,
        cutoff: Option<&str>,
    ) -> Result<TokenSummary, LlmusageAdapterError> {
        let mut sql_filter = filter.bucket_filter(None);
        if let Some(cutoff) = cutoff {
            sql_filter.push("hour_start >= ?", cutoff);
        }
        let sql = format!(
            r#"
            SELECT COALESCE(SUM(input_tokens), 0), COALESCE(SUM(cache_read_tokens), 0), COALESCE(SUM(output_tokens), 0), COALESCE(SUM(reasoning_output_tokens), 0), COALESCE(SUM(total_tokens), 0)
            FROM usage_bucket_30m{}
        "#,
            sql_filter.where_sql()
        );
        self.conn
            .query_row(&sql, params_from_iter(sql_filter.param_refs()), |row| {
                Ok(TokenSummary {
                    input_tokens: row.get(0)?,
                    cache_read_tokens: row.get(1)?,
                    output_tokens: row.get(2)?,
                    reasoning_output_tokens: row.get(3)?,
                    total_tokens: row.get(4)?,
                })
            })
            .map_err(Into::into)
    }

    fn query_event_count(
        &self,
        filter: &QueryFilter,
        cutoff: Option<&str>,
    ) -> Result<i64, LlmusageAdapterError> {
        let mut sql_filter = filter.bucket_filter(None);
        if let Some(cutoff) = cutoff {
            sql_filter.push("hour_start >= ?", cutoff);
        }
        scalar_i64(
            &self.conn,
            &format!(
                "SELECT COALESCE(SUM(event_count), 0) FROM usage_bucket_30m{}",
                sql_filter.where_sql()
            ),
            &sql_filter.param_refs(),
        )
        .map_err(Into::into)
    }

    fn query_cost_with_cache(
        &self,
        filter: &QueryFilter,
        cutoff: Option<&str>,
    ) -> Result<f64, LlmusageAdapterError> {
        let mut sql_filter = filter.bucket_filter(None);
        if let Some(cutoff) = cutoff {
            sql_filter.push("hour_start >= ?", cutoff);
        }
        scalar_f64(
            &self.conn,
            &format!(
                "SELECT COALESCE(SUM(cost_with_cache_usd), 0.0) FROM usage_bucket_30m{}",
                sql_filter.where_sql()
            ),
            &sql_filter.param_refs(),
        )
        .map_err(Into::into)
    }
}

#[derive(Debug, Clone)]
pub struct LogsQuery {
    pub filter: QueryFilter,
    pub page_size: u32,
    pub cursor: Option<String>,
    pub include_total: bool,
    pub include_raw_json: bool,
}

#[derive(Debug, Clone)]
pub struct LogsPage {
    pub records: Vec<UsageRecordDto>,
    pub total: Option<i64>,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SourceDiagnostics {
    #[allow(dead_code)]
    pub source: String,
    pub live_files: u64,
    pub missing_files: u64,
    pub deleted_files: u64,
    pub recent_completed_at: Option<String>,
    pub history_completed_at: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiagnosticsPayload {
    pub archive_root: String,
    pub by_source: Vec<SourceDiagnostics>,
}

fn scalar_i64(conn: &Connection, sql: &str, params: &[&dyn ToSql]) -> rusqlite::Result<i64> {
    conn.query_row(sql, params_from_iter(params.iter().copied()), |row| {
        row.get::<_, Option<i64>>(0)
    })
    .map(|value| value.unwrap_or_default())
}

fn scalar_f64(conn: &Connection, sql: &str, params: &[&dyn ToSql]) -> rusqlite::Result<f64> {
    conn.query_row(sql, params_from_iter(params.iter().copied()), |row| {
        row.get::<_, Option<f64>>(0)
    })
    .map(|value| value.unwrap_or_default())
}

fn scalar_optional_string(
    conn: &Connection,
    sql: &str,
    params: &[&dyn ToSql],
) -> rusqlite::Result<Option<String>> {
    // 没有匹配行时 SQLite 返回 QueryReturnedNoRows；语义上等价于 None，避免上层
    // 误把"无数据"当成真实错误吞掉。其他 rusqlite::Error 继续向上传播。
    match conn.query_row(sql, params_from_iter(params.iter().copied()), |row| {
        row.get::<_, Option<String>>(0)
    }) {
        Ok(value) => Ok(value),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error),
    }
}

fn map_usage_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<UsageRecordDto> {
    Ok(UsageRecordDto {
        id: row.get(0)?,
        platform: row.get(1)?,
        project_path: row.get(2)?,
        record_json: row.get(3)?,
        recorded_at: row.get(4)?,
        source_id: row.get(5)?,
        model: non_empty(row.get(6)?),
        input_tokens: row.get(7)?,
        output_tokens: row.get(8)?,
        cache_read_tokens: row.get(9)?,
        cache_creation_tokens: row.get(10)?,
        cost_with_cache_usd: row.get(11)?,
        cost_without_cache_usd: row.get(12)?,
        pricing_status: row.get(13)?,
        pricing_source: row.get(14)?,
    })
}

fn non_empty(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

fn encode_cursor(event_at: &str, event_key: &str) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(format!("{event_at}\n{event_key}"))
}

fn decode_cursor(cursor: &str) -> Option<(String, String)> {
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(cursor)
        .ok()?;
    let value = String::from_utf8(decoded).ok()?;
    let (event_at, event_key) = value.split_once('\n')?;
    Some((event_at.to_string(), event_key.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn build_filter_rejects_invalid_dates() {
        let error = build_filter(None, None, Some("2026/05/09".to_string()), None)
            .expect_err("invalid date should be rejected");
        assert!(error.contains("start_date"));
    }

    #[test]
    fn cursor_round_trip_preserves_order_keys() {
        let cursor = encode_cursor("2026-05-12T00:00:00Z", "abc");
        assert_eq!(
            decode_cursor(&cursor),
            Some(("2026-05-12T00:00:00Z".to_string(), "abc".to_string()))
        );
    }

    #[test]
    fn empty_filter_emits_no_where_clause() {
        let filter = QueryFilter::default();
        let bucket = filter.bucket_filter(None);
        assert_eq!(bucket.where_sql(), "");
        assert!(bucket.params.is_empty());
        let event = filter.event_filter(None);
        assert_eq!(event.where_sql(), "");
        assert!(event.params.is_empty());
    }

    #[test]
    fn source_only_filter_uses_single_clause_with_canonical_value() {
        let filter = QueryFilter {
            source: Some(SourceKind::Claude),
            ..QueryFilter::default()
        };
        let bucket = filter.bucket_filter(None);
        assert_eq!(bucket.where_sql(), " WHERE source = ?");
        assert_eq!(bucket.params, vec!["claude".to_string()]);
    }

    #[test]
    fn since_until_use_local_modifier_in_local_timezone() {
        let filter = QueryFilter {
            since: NaiveDate::from_ymd_opt(2026, 5, 1),
            until: NaiveDate::from_ymd_opt(2026, 5, 9),
            ..QueryFilter::default()
        };
        let bucket = filter.bucket_filter(None);
        let sql = bucket.where_sql();
        assert!(
            sql.contains("date(hour_start, 'localtime') >= ?"),
            "since clause must use localtime modifier; got {sql}"
        );
        assert!(
            sql.contains("date(hour_start, 'localtime') <= ?"),
            "until clause must use localtime modifier; got {sql}"
        );
        assert_eq!(bucket.params, vec!["2026-05-01", "2026-05-09"]);
    }

    #[test]
    fn since_until_switch_to_utc_modifier_in_utc_timezone() {
        let filter = QueryFilter {
            since: NaiveDate::from_ymd_opt(2026, 5, 1),
            timezone: ReportTimezone::Utc,
            ..QueryFilter::default()
        };
        let sql = filter.bucket_filter(None).where_sql();
        assert!(
            sql.contains("date(hour_start, 'utc') >= ?"),
            "utc tz must emit utc modifier; got {sql}"
        );
        assert!(
            !sql.contains("localtime"),
            "utc tz must not emit localtime; got {sql}"
        );
    }

    #[test]
    fn alias_prefix_propagates_to_every_column_reference() {
        let filter = QueryFilter {
            source: Some(SourceKind::Codex),
            model: Some("gpt-5".to_string()),
            since: NaiveDate::from_ymd_opt(2026, 5, 1),
            project_hash: Some("hash-abc".to_string()),
            ..QueryFilter::default()
        };
        let event = filter.event_filter(Some("e"));
        let sql = event.where_sql();
        assert!(sql.contains("e.source = ?"));
        assert!(sql.contains("e.model = ?"));
        assert!(sql.contains("date(e.event_at, 'localtime') >= ?"));
        assert!(sql.contains("e.project_hash = ?"));
        assert!(
            !sql.contains(" source = ?") || sql.contains("e.source = ?"),
            "no unprefixed column reference should leak: {sql}"
        );
    }

    #[test]
    fn project_hash_propagates_to_both_bucket_and_event_filters() {
        let filter = QueryFilter {
            project_hash: Some("hash-xyz".to_string()),
            ..QueryFilter::default()
        };
        let bucket = filter.bucket_filter(None);
        let event = filter.event_filter(None);
        assert!(bucket.where_sql().contains("project_hash = ?"));
        assert!(event.where_sql().contains("project_hash = ?"));
        assert_eq!(bucket.params, vec!["hash-xyz".to_string()]);
        assert_eq!(event.params, vec!["hash-xyz".to_string()]);
    }

    #[test]
    fn trends_daily_keeps_output_compatible_and_exposes_reasoning() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let db_path = temp.path().join("llmusage.db");
        let conn = Connection::open(&db_path).expect("test db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta(key, value) VALUES ('schema_version', '10');
            CREATE TABLE usage_bucket_30m(
                source TEXT NOT NULL,
                hour_start TEXT NOT NULL,
                input_tokens INTEGER NOT NULL,
                cache_read_tokens INTEGER NOT NULL,
                cache_creation_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                reasoning_output_tokens INTEGER NOT NULL,
                total_tokens INTEGER NOT NULL,
                event_count INTEGER NOT NULL,
                cost_with_cache_usd REAL NOT NULL
            );
            INSERT INTO usage_bucket_30m VALUES
              ('codex', '2026-05-21T00:00:00Z', 40, 10, 5, 20, 7, 82, 2, 0.12),
              ('codex', '2026-05-21T12:00:00Z', 30, 8, 3, 10, 4, 55, 1, 0.08),
              ('codex', '2026-05-22T00:00:00Z', 5, 1, 0, 2, 1, 9, 1, 0.01);
            "#,
        )
        .expect("test schema should be created");
        drop(conn);

        let dashboard = Dashboard::open(AppPaths::from_root_for_test(temp.path()))
            .expect("dashboard should open test db");
        let trends = dashboard
            .trends_daily(&QueryFilter::default())
            .expect("daily trends should query");

        assert_eq!(trends.len(), 2);
        assert_eq!(trends[0].date, "2026-05-21");
        assert_eq!(trends[0].output_tokens, 41);
        assert_eq!(trends[0].reasoning_output_tokens, 11);
        assert_eq!(trends[0].total_tokens, 137);
    }

    #[test]
    fn source_breakdown_respects_date_range_and_ignores_source_filter() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let db_path = temp.path().join("llmusage.db");
        let conn = Connection::open(&db_path).expect("test db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta(key, value) VALUES ('schema_version', '10');
            CREATE TABLE usage_bucket_30m(
                source TEXT NOT NULL,
                hour_start TEXT NOT NULL,
                input_tokens INTEGER NOT NULL,
                cache_read_tokens INTEGER NOT NULL,
                cache_creation_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                reasoning_output_tokens INTEGER NOT NULL,
                total_tokens INTEGER NOT NULL,
                event_count INTEGER NOT NULL,
                cost_with_cache_usd REAL NOT NULL
            );
            INSERT INTO usage_bucket_30m VALUES
              ('claude', '2026-05-20T00:00:00Z', 10, 0, 0, 5, 0, 15, 1, 0.30),
              ('codex', '2026-05-20T12:00:00Z', 20, 0, 0, 10, 0, 30, 2, 0.70),
              ('gemini', '2026-05-21T00:00:00Z', 40, 0, 0, 20, 0, 60, 3, 1.00),
              ('legacy', '2026-05-21T12:00:00Z', 400, 0, 0, 200, 0, 600, 5, 9.00),
              ('codex', '2026-05-22T00:00:00Z', 80, 0, 0, 40, 0, 120, 4, 5.00);
            "#,
        )
        .expect("test schema should be created");
        drop(conn);

        let dashboard = Dashboard::open(AppPaths::from_root_for_test(temp.path()))
            .expect("dashboard should open test db");
        let stats = dashboard
            .source_breakdown(&QueryFilter {
                source: Some(SourceKind::Claude),
                since: NaiveDate::from_ymd_opt(2026, 5, 20),
                until: NaiveDate::from_ymd_opt(2026, 5, 21),
                ..QueryFilter::default()
            })
            .expect("source breakdown should query");

        assert_eq!(
            stats
                .iter()
                .map(|row| row.source.as_str())
                .collect::<Vec<_>>(),
            vec!["gemini", "codex", "claude"]
        );
        assert_eq!(stats.iter().map(|row| row.total_tokens).sum::<i64>(), 105);
        assert_eq!(
            stats
                .iter()
                .find(|row| row.source == "codex")
                .unwrap()
                .event_count,
            2
        );
        assert!((stats.iter().map(|row| row.share_tokens).sum::<f64>() - 1.0).abs() < f64::EPSILON);
        assert!(
            stats
                .iter()
                .all(|row| row.source != "opencode" && row.total_cost < 5.0)
        );
    }
}
