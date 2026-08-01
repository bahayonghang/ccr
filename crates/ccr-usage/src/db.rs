use chrono::{NaiveDate, SecondsFormat};
use rusqlite::{Connection, OpenFlags, ToSql, params_from_iter, types::Value};

use crate::{
    AppPaths, FeatureKey, UsageError,
    capabilities::{
        DbCapabilitySnapshot, MIN_SUPPORTED_SCHEMA_VERSION, read_schema_version, table_exists,
    },
    queries::{
        DailyTrendDto, HeatmapPoint, HomeOverviewPayload, HomeOverviewPlatformStats,
        HomeOverviewSeriesItem, HomeOverviewSummary, ModelBreakdown, OverviewPayload,
        ProjectBreakdown, ProviderBreakdownDto, SourceBreakdownDto, TaggedProviderBreakdown,
        TokenSummary, UsageRecordDto, generated_at,
    },
    source::{SourceKind, parse_source_filter},
    timezone::{ResolvedZone, register_functions},
};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ReportTimezone {
    #[default]
    Local,
    Utc,
}

#[derive(Debug, Clone, Default)]
pub struct QueryFilter {
    pub source: Option<SourceKind>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub since: Option<NaiveDate>,
    pub until: Option<NaiveDate>,
    pub project_hash: Option<String>,
    pub timezone: ReportTimezone,
}

#[derive(Debug, Clone)]
struct SqlFilter {
    clauses: Vec<String>,
    params: Vec<Value>,
}

impl SqlFilter {
    fn new() -> Self {
        Self {
            clauses: Vec::new(),
            params: Vec::new(),
        }
    }

    fn push(&mut self, clause: impl Into<String>, value: impl Into<Value>) {
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
    fn bucket_filter(
        &self,
        alias: Option<&str>,
        zone: &ResolvedZone,
        schema_version: i64,
    ) -> Result<SqlFilter, UsageError> {
        self.sql_filter(alias, "hour_start", zone, schema_version)
    }

    fn event_filter(
        &self,
        alias: Option<&str>,
        zone: &ResolvedZone,
        schema_version: i64,
    ) -> Result<SqlFilter, UsageError> {
        self.sql_filter(alias, "event_at", zone, schema_version)
    }

    fn sql_filter(
        &self,
        alias: Option<&str>,
        time_column: &str,
        zone: &ResolvedZone,
        schema_version: i64,
    ) -> Result<SqlFilter, UsageError> {
        let prefix = alias.map(|value| format!("{value}.")).unwrap_or_default();
        let mut filter = SqlFilter::new();
        if let Some(source) = self.source {
            filter.push(
                format!("{prefix}source = ?"),
                source.storage_key(schema_version).to_string(),
            );
        }
        if let Some(model) = self.model.as_ref() {
            filter.push(format!("{prefix}model = ?"), model.clone());
        }
        if let Some(provider) = self.provider.as_ref() {
            filter.push(format!("{prefix}provider_label = ?"), provider.clone());
        }
        if let Some(since) = self.since {
            filter.push(
                format!("{prefix}{time_column} >= ?"),
                zone.local_date_start_utc(since)?
                    .to_rfc3339_opts(SecondsFormat::Secs, true),
            );
        }
        if let Some(until) = self.until.and_then(|date| date.succ_opt()) {
            filter.push(
                format!("{prefix}{time_column} < ?"),
                zone.local_date_start_utc(until)?
                    .to_rfc3339_opts(SecondsFormat::Secs, true),
            );
        }
        if let Some(project_hash) = self.project_hash.as_ref() {
            filter.push(format!("{prefix}project_hash = ?"), project_hash.clone());
        }
        Ok(filter)
    }

    fn without_source(&self) -> Self {
        Self {
            source: None,
            model: self.model.clone(),
            provider: self.provider.clone(),
            since: self.since,
            until: self.until,
            project_hash: self.project_hash.clone(),
            timezone: self.timezone,
        }
    }

    pub fn with_provider(mut self, provider: Option<String>) -> Self {
        self.provider = provider.map(|value| value.trim().to_string());
        self
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
        provider: None,
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
    capabilities: DbCapabilitySnapshot,
    local_zone: ResolvedZone,
}

pub fn open_dashboard(paths: AppPaths) -> Result<Dashboard, UsageError> {
    Dashboard::open(paths)
}

impl Dashboard {
    pub fn open(paths: AppPaths) -> Result<Self, UsageError> {
        if !paths.db_path.is_file() {
            return Err(UsageError::DbMissing(paths.db_path.clone()));
        }
        let conn = Connection::open_with_flags(
            &paths.db_path,
            OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI,
        )
        .map_err(|error| UsageError::DbUnreadable(error.to_string()))?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        register_functions(&conn)?;
        let schema_version = read_schema_version(&conn)?;
        if schema_version.unwrap_or_default() < MIN_SUPPORTED_SCHEMA_VERSION {
            return Err(UsageError::SchemaUnsupported {
                expected: MIN_SUPPORTED_SCHEMA_VERSION,
                actual: schema_version,
            });
        }
        let capabilities = DbCapabilitySnapshot::from_connection(&conn)?;
        Ok(Self {
            paths,
            conn,
            capabilities,
            local_zone: ResolvedZone::local()?,
        })
    }

    // provider filter 附着在任意查询上时都要求 ProviderBreakdown 能力，
    // 否则旧 schema 会静默返回未过滤数据（契约禁止）。
    fn ensure_feature_for_filter(
        &self,
        feature: FeatureKey,
        filter: &QueryFilter,
    ) -> Result<(), UsageError> {
        self.capabilities.ensure(feature)?;
        if filter.provider.is_some() && feature != FeatureKey::ProviderBreakdown {
            self.capabilities.ensure(FeatureKey::ProviderBreakdown)?;
        }
        Ok(())
    }

    fn schema_version(&self) -> i64 {
        self.capabilities.schema_version.unwrap_or_default()
    }

    fn zone(&self, timezone: ReportTimezone) -> ResolvedZone {
        match timezone {
            ReportTimezone::Local => self.local_zone.clone(),
            ReportTimezone::Utc => ResolvedZone::utc(),
        }
    }

    pub fn overview(&self, filter: &QueryFilter) -> Result<OverviewPayload, UsageError> {
        self.ensure_feature_for_filter(FeatureKey::Overview, filter)?;
        let zone = self.zone(filter.timezone);
        let bucket_filter = filter.bucket_filter(None, &zone, self.schema_version())?;
        let cutoff = (chrono::Utc::now() - chrono::Duration::hours(24))
            .to_rfc3339_opts(SecondsFormat::Secs, true);
        let sql = format!(
            r#"
            WITH filtered AS (
                SELECT * FROM usage_bucket_30m{}
            )
            SELECT
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(cache_read_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(reasoning_output_tokens), 0),
                COALESCE(SUM(total_tokens), 0),
                COALESCE(SUM(CASE WHEN hour_start >= cutoff.value THEN input_tokens ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN hour_start >= cutoff.value THEN cache_read_tokens ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN hour_start >= cutoff.value THEN output_tokens ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN hour_start >= cutoff.value THEN reasoning_output_tokens ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN hour_start >= cutoff.value THEN total_tokens ELSE 0 END), 0),
                COALESCE(SUM(event_count), 0),
                COALESCE(SUM(CASE WHEN hour_start >= cutoff.value THEN event_count ELSE 0 END), 0),
                COALESCE(SUM(cost_with_cache_usd), 0.0),
                COUNT(DISTINCT source),
                COUNT(*)
            FROM filtered
            CROSS JOIN (SELECT ? AS value) AS cutoff
            "#,
            bucket_filter.where_sql()
        );
        let mut params = bucket_filter.params.clone();
        params.push(Value::Text(cutoff));
        let values = self
            .conn
            .query_row(&sql, params_from_iter(params.iter()), |row| {
                Ok((
                    TokenSummary {
                        input_tokens: row.get(0)?,
                        cache_read_tokens: row.get(1)?,
                        output_tokens: row.get(2)?,
                        reasoning_output_tokens: row.get(3)?,
                        total_tokens: row.get(4)?,
                    },
                    TokenSummary {
                        input_tokens: row.get(5)?,
                        cache_read_tokens: row.get(6)?,
                        output_tokens: row.get(7)?,
                        reasoning_output_tokens: row.get(8)?,
                        total_tokens: row.get(9)?,
                    },
                    row.get(10)?,
                    row.get(11)?,
                    row.get(12)?,
                    row.get(13)?,
                    row.get(14)?,
                ))
            })?;
        let (
            total,
            last_24h,
            total_events,
            last_24h_events,
            total_cost_usd,
            source_count,
            bucket_count,
        ) = values;

        let freshness = self.conn.query_row(
            r#"
            SELECT
                MAX(CASE WHEN command IN ('sync', 'hook-run') THEN finished_at END),
                MAX(CASE WHEN command = 'export html' THEN finished_at END)
            FROM run_log
            WHERE status = 'success'
            "#,
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );
        let (last_sync_at, last_export_at) = freshness.unwrap_or_else(|error| {
            tracing::warn!(?error, "llmusage overview: freshness query failed");
            (None, None)
        });
        Ok(OverviewPayload {
            generated_at: generated_at(),
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

    pub fn trends_daily(&self, filter: &QueryFilter) -> Result<Vec<DailyTrendDto>, UsageError> {
        self.ensure_feature_for_filter(FeatureKey::DailyTrends, filter)?;
        let zone = self.zone(filter.timezone);
        let sql_filter = filter.bucket_filter(None, &zone, self.schema_version())?;
        let local_date = zone.local_date_expr("hour_start");
        let sql = format!(
            r#"
            SELECT
                {local_date} AS local_date,
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

    pub fn model_breakdown(&self, filter: &QueryFilter) -> Result<Vec<ModelBreakdown>, UsageError> {
        self.ensure_feature_for_filter(FeatureKey::ModelBreakdown, filter)?;
        let zone = self.zone(filter.timezone);
        let sql_filter = filter.bucket_filter(None, &zone, self.schema_version())?;
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

    pub fn provider_breakdown(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<ProviderBreakdownDto>, UsageError> {
        self.ensure_feature_for_filter(FeatureKey::ProviderBreakdown, filter)?;
        let zone = self.zone(filter.timezone);
        let sql_filter = filter.bucket_filter(None, &zone, self.schema_version())?;
        let sql = format!(
            r#"
            SELECT
                provider_label,
                COALESCE(SUM(event_count), 0),
                COALESCE(SUM(input_tokens), 0),
                COALESCE(SUM(cache_read_tokens), 0),
                COALESCE(SUM(cache_creation_tokens), 0),
                COALESCE(SUM(output_tokens), 0),
                COALESCE(SUM(reasoning_output_tokens), 0),
                COALESCE(SUM(total_tokens), 0),
                COALESCE(SUM(cost_with_cache_usd), 0.0),
                COALESCE(SUM(cost_without_cache_usd), 0.0)
            FROM usage_bucket_30m
            {}
            GROUP BY provider_label
            ORDER BY SUM(total_tokens) DESC, provider_label ASC
        "#,
            sql_filter.where_sql()
        );
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params_from_iter(sql_filter.param_refs()), |row| {
            Ok(ProviderBreakdownDto {
                provider: non_empty(row.get(0)?),
                request_count: row.get(1)?,
                input_tokens: row.get(2)?,
                cache_read_tokens: row.get(3)?,
                cache_creation_tokens: row.get(4)?,
                output_tokens: row.get(5)?,
                reasoning_output_tokens: row.get(6)?,
                total_tokens: row.get(7)?,
                cost_with_cache_usd: row.get(8)?,
                cost_without_cache_usd: row.get(9)?,
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    /// Queries each source separately (contract: per-source sections must not
    /// blend rows) and tags every row with the source it belongs to.
    pub fn provider_breakdown_by_source(
        &self,
        sources: &[SourceKind],
        filter: &QueryFilter,
    ) -> Result<Vec<TaggedProviderBreakdown>, UsageError> {
        let mut rows = Vec::new();
        for source in sources {
            let scoped = QueryFilter {
                source: Some(*source),
                ..filter.clone()
            };
            rows.extend(
                self.provider_breakdown(&scoped)?
                    .into_iter()
                    .map(|breakdown| TaggedProviderBreakdown {
                        source: *source,
                        breakdown,
                    }),
            );
        }
        Ok(rows)
    }

    pub fn project_breakdown(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<ProjectBreakdown>, UsageError> {
        self.ensure_feature_for_filter(FeatureKey::ProjectBreakdown, filter)?;
        let zone = self.zone(filter.timezone);
        let mut sql_filter = filter.bucket_filter(None, &zone, self.schema_version())?;
        sql_filter.push_raw("project_hash <> ''");
        let sql = format!(
            r#"
            SELECT
                project_hash,
                MAX(project_label),
                MAX(project_ref),
                SUM(total_tokens),
                SUM(event_count),
                SUM(cost_with_cache_usd)
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
            Ok(ProjectBreakdown {
                project_hash: row.get(0)?,
                project_label: project_label.clone(),
                project_ref: project_ref.clone(),
                total_tokens: row.get::<_, Option<i64>>(3)?.unwrap_or_default(),
                event_count: row.get::<_, Option<i64>>(4)?.unwrap_or_default(),
                total_cost_usd: row.get::<_, Option<f64>>(5)?.unwrap_or_default(),
                project_path: project_ref.or(Some(project_label)),
            })
        })?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn source_breakdown(
        &self,
        filter: &QueryFilter,
    ) -> Result<Vec<SourceBreakdownDto>, UsageError> {
        self.ensure_feature_for_filter(FeatureKey::HomeOverview, filter)?;
        let source_filter = filter.without_source();
        let zone = self.zone(source_filter.timezone);
        let sql_filter = source_filter.bucket_filter(None, &zone, self.schema_version())?;
        let local_date = zone.local_date_expr("hour_start");
        let sql = format!(
            r#"
            SELECT
                CASE WHEN source = 'gemini' THEN 'antigravity' ELSE source END AS canonical_source,
                COALESCE(SUM(event_count), 0),
                COALESCE(SUM(total_tokens), 0),
                COALESCE(SUM(cost_with_cache_usd), 0.0),
                COUNT(DISTINCT {local_date})
            FROM usage_bucket_30m
            {}
            GROUP BY canonical_source
            ORDER BY SUM(total_tokens) DESC, canonical_source ASC
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

        let total_tokens: i64 = raw_rows.iter().map(|row| row.total_tokens).sum();
        let total_cost: f64 = raw_rows.iter().map(|row| row.total_cost).sum();

        Ok(raw_rows
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
    ) -> Result<Vec<HeatmapPoint>, UsageError> {
        self.ensure_feature_for_filter(FeatureKey::Heatmap, filter)?;
        let days = days.clamp(1, 366);
        let zone = self.zone(filter.timezone);
        let end = zone.date_at(chrono::Utc::now());
        let start = end - chrono::Duration::days((days - 1) as i64);
        // 用 heatmap 自身的滚动窗口覆盖 caller 透传的 since/until，
        // 避免 bucket_filter() 在 WHERE 里推出两份 date(...) 约束。
        // source/model/project_hash/timezone 仍来自 caller。
        let scoped_filter = QueryFilter {
            since: Some(start),
            until: Some(end),
            ..filter.clone()
        };
        let sql_filter = scoped_filter.bucket_filter(None, &zone, self.schema_version())?;
        let local_date = zone.local_date_expr("hour_start");
        let sql = format!(
            r#"
            SELECT {local_date} AS date, COALESCE(SUM(event_count), 0)
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

    pub fn logs(&self, query: &LogsQuery) -> Result<LogsPage, UsageError> {
        self.ensure_feature_for_filter(FeatureKey::Logs, &query.filter)?;
        let zone = self.zone(query.filter.timezone);
        let mut sql_filter = query
            .filter
            .event_filter(Some("e"), &zone, self.schema_version())?;
        if let Some(cursor) = query.cursor.as_ref().and_then(|value| decode_cursor(value)) {
            sql_filter.push_raw("(e.event_at < ? OR (e.event_at = ? AND e.event_key < ?))");
            sql_filter.params.push(Value::Text(cursor.0.clone()));
            sql_filter.params.push(Value::Text(cursor.0));
            sql_filter.params.push(Value::Text(cursor.1));
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
                CASE WHEN e.source = 'gemini' THEN 'antigravity' ELSE e.source END,
                COALESCE(e.project_path, e.project_ref, e.project_label, e.project_hash, ''),
                COALESCE({raw_expr}, ''),
                e.event_at,
                CASE WHEN e.source = 'gemini' THEN 'antigravity' ELSE e.source END,
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
        params.push(Value::Integer(i64::from(query.page_size) + 1));
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
            let count_filter =
                query
                    .filter
                    .event_filter(Some("e"), &zone, self.schema_version())?;
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

    pub fn diagnostics(&self) -> Result<DiagnosticsPayload, UsageError> {
        self.capabilities.ensure(FeatureKey::Diagnostics)?;
        let sql = r#"
            SELECT
                CASE WHEN f.source = 'gemini' THEN 'antigravity' ELSE f.source END AS canonical_source,
                SUM(CASE WHEN f.state = 'live' THEN 1 ELSE 0 END),
                SUM(CASE WHEN f.state = 'missing' THEN 1 ELSE 0 END),
                SUM(CASE WHEN f.state = 'deleted_by_user' THEN 1 ELSE 0 END),
                MAX(s.recent_completed_at),
                MAX(s.history_completed_at)
            FROM source_file f
            LEFT JOIN source_sync_status s ON s.source = f.source
            GROUP BY canonical_source
            ORDER BY canonical_source ASC
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

    pub fn home_overview(&self, filter: &QueryFilter) -> Result<HomeOverviewPayload, UsageError> {
        self.ensure_feature_for_filter(FeatureKey::HomeOverview, filter)?;
        let mut by_platform = std::collections::BTreeMap::new();
        for source in SourceKind::ALL {
            by_platform.insert(
                source.as_str().to_string(),
                HomeOverviewPlatformStats::default(),
            );
        }
        let mut by_day = std::collections::BTreeMap::<
            String,
            std::collections::BTreeMap<String, HomeOverviewPlatformStats>,
        >::new();

        let zone = self.zone(filter.timezone);
        let sql_filter = filter.bucket_filter(None, &zone, self.schema_version())?;
        let local_date = zone.local_date_expr("hour_start");
        let sql = format!(
            r#"
            SELECT
                {local_date},
                CASE WHEN source = 'gemini' THEN 'antigravity' ELSE source END AS canonical_source,
                COALESCE(SUM(event_count), 0),
                COALESCE(SUM(total_tokens), 0)
            FROM usage_bucket_30m
            {}
            GROUP BY 1, canonical_source
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
            let total = by_platform.entry(source.clone()).or_default();
            total.requests += requests;
            total.tokens += tokens;
            by_day.entry(date).or_default().insert(source, stats);
        }
        let series = by_day
            .into_iter()
            .map(|(date, mut day)| HomeOverviewSeriesItem {
                date,
                claude: day.remove("claude").unwrap_or_default(),
                codex: day.remove("codex").unwrap_or_default(),
                antigravity: day.remove("antigravity").unwrap_or_default(),
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
                active_days: series.len() as i64,
                platforms,
            },
            by_platform,
            series,
        })
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
    use crate::capabilities::{
        capability_connection_open_count, reset_capability_connection_open_count,
    };
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
        let zone = ResolvedZone::utc();
        let bucket = filter
            .bucket_filter(None, &zone, 19)
            .expect("empty bucket filter should build");
        assert_eq!(bucket.where_sql(), "");
        assert!(bucket.params.is_empty());
        let event = filter
            .event_filter(None, &zone, 19)
            .expect("empty event filter should build");
        assert_eq!(event.where_sql(), "");
        assert!(event.params.is_empty());
    }

    #[test]
    fn source_only_filter_uses_single_clause_with_canonical_value() {
        let filter = QueryFilter {
            source: Some(SourceKind::Claude),
            ..QueryFilter::default()
        };
        let bucket = filter
            .bucket_filter(None, &ResolvedZone::utc(), 19)
            .expect("source bucket filter should build");
        assert_eq!(bucket.where_sql(), " WHERE source = ?");
        assert_eq!(bucket.params, vec![Value::Text("claude".to_string())]);
    }

    #[test]
    fn since_until_use_dst_aware_utc_half_open_bounds() {
        let filter = QueryFilter {
            since: NaiveDate::from_ymd_opt(2026, 5, 1),
            until: NaiveDate::from_ymd_opt(2026, 5, 9),
            ..QueryFilter::default()
        };
        let zone = ResolvedZone::Iana(chrono_tz::America::New_York);
        let bucket = filter
            .bucket_filter(None, &zone, 19)
            .expect("DST-aware bucket filter should build");
        let sql = bucket.where_sql();
        assert_eq!(sql, " WHERE hour_start >= ? AND hour_start < ?");
        assert_eq!(
            bucket.params,
            vec![
                Value::Text("2026-05-01T04:00:00Z".to_string()),
                Value::Text("2026-05-10T04:00:00Z".to_string()),
            ]
        );
    }

    #[test]
    fn dst_transition_days_emit_23_and_25_hour_half_open_bounds() {
        let zone = ResolvedZone::Iana(chrono_tz::America::New_York);
        for (date, expected_start, expected_end) in [
            (
                NaiveDate::from_ymd_opt(2026, 3, 8).expect("spring-forward date should be valid"),
                "2026-03-08T05:00:00Z",
                "2026-03-09T04:00:00Z",
            ),
            (
                NaiveDate::from_ymd_opt(2026, 11, 1).expect("fall-back date should be valid"),
                "2026-11-01T04:00:00Z",
                "2026-11-02T05:00:00Z",
            ),
        ] {
            let filter = QueryFilter {
                since: Some(date),
                until: Some(date),
                ..QueryFilter::default()
            };
            let bucket = filter
                .bucket_filter(None, &zone, 19)
                .expect("DST transition filter should build");

            assert_eq!(
                bucket.params,
                vec![
                    Value::Text(expected_start.to_string()),
                    Value::Text(expected_end.to_string()),
                ]
            );
        }
    }

    #[test]
    fn utc_dates_emit_midnight_z_bounds() {
        let filter = QueryFilter {
            since: NaiveDate::from_ymd_opt(2026, 5, 1),
            until: NaiveDate::from_ymd_opt(2026, 5, 1),
            timezone: ReportTimezone::Utc,
            ..QueryFilter::default()
        };
        let bucket = filter
            .bucket_filter(None, &ResolvedZone::utc(), 19)
            .expect("UTC bucket filter should build");
        assert_eq!(
            bucket.where_sql(),
            " WHERE hour_start >= ? AND hour_start < ?"
        );
        assert_eq!(
            bucket.params,
            vec![
                Value::Text("2026-05-01T00:00:00Z".to_string()),
                Value::Text("2026-05-02T00:00:00Z".to_string()),
            ]
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
        let event = filter
            .event_filter(Some("e"), &ResolvedZone::utc(), 19)
            .expect("aliased event filter should build");
        let sql = event.where_sql();
        assert!(sql.contains("e.source = ?"));
        assert!(sql.contains("e.model = ?"));
        assert!(sql.contains("e.event_at >= ?"));
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
        let zone = ResolvedZone::utc();
        let bucket = filter
            .bucket_filter(None, &zone, 19)
            .expect("project bucket filter should build");
        let event = filter
            .event_filter(None, &zone, 19)
            .expect("project event filter should build");
        assert!(bucket.where_sql().contains("project_hash = ?"));
        assert!(event.where_sql().contains("project_hash = ?"));
        assert_eq!(bucket.params, vec![Value::Text("hash-xyz".to_string())]);
        assert_eq!(event.params, vec![Value::Text("hash-xyz".to_string())]);
    }

    #[test]
    fn antigravity_filter_uses_legacy_storage_key_before_schema_13() {
        let filter = QueryFilter {
            source: Some(SourceKind::Antigravity),
            ..QueryFilter::default()
        };
        let zone = ResolvedZone::utc();
        assert_eq!(
            filter
                .bucket_filter(None, &zone, 10)
                .expect("legacy Antigravity filter should build")
                .params,
            vec![Value::Text("gemini".to_string())]
        );
        assert_eq!(
            filter
                .bucket_filter(None, &zone, 13)
                .expect("current Antigravity filter should build")
                .params,
            vec![Value::Text("antigravity".to_string())]
        );
    }

    #[test]
    fn half_open_ranges_use_upstream_time_indexes() {
        let conn = Connection::open_in_memory().expect("in-memory DB should open");
        conn.execute_batch(
            r#"
            CREATE TABLE usage_bucket_30m(hour_start TEXT NOT NULL);
            CREATE INDEX idx_usage_bucket_30m_hour_start ON usage_bucket_30m(hour_start);
            CREATE TABLE usage_event(event_at TEXT NOT NULL);
            CREATE INDEX idx_usage_event_event_at ON usage_event(event_at);
            "#,
        )
        .expect("range index fixture should be created");
        let filter = QueryFilter {
            since: NaiveDate::from_ymd_opt(2026, 5, 1),
            until: NaiveDate::from_ymd_opt(2026, 5, 9),
            timezone: ReportTimezone::Utc,
            ..QueryFilter::default()
        };
        let zone = ResolvedZone::utc();
        for (table, sql_filter, expected_index) in [
            (
                "usage_bucket_30m",
                filter
                    .bucket_filter(None, &zone, 19)
                    .expect("indexed bucket filter should build"),
                "idx_usage_bucket_30m_hour_start",
            ),
            (
                "usage_event",
                filter
                    .event_filter(None, &zone, 19)
                    .expect("indexed event filter should build"),
                "idx_usage_event_event_at",
            ),
        ] {
            let sql = format!(
                "EXPLAIN QUERY PLAN SELECT * FROM {table}{}",
                sql_filter.where_sql()
            );
            let mut stmt = conn.prepare(&sql).expect("query plan should prepare");
            let details = stmt
                .query_map(params_from_iter(sql_filter.param_refs()), |row| {
                    row.get::<_, String>(3)
                })
                .expect("query plan rows should execute")
                .collect::<rusqlite::Result<Vec<_>>>()
                .expect("query plan rows should decode")
                .join("\n");
            assert!(
                details.contains(expected_index),
                "unexpected plan: {details}"
            );
        }
    }

    fn create_provider_fixture(root: &std::path::Path) -> Dashboard {
        let db_path = root.join("llmusage.db");
        let conn = Connection::open(&db_path).expect("test db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta(key, value) VALUES ('schema_version', '14');
            CREATE TABLE usage_event(provider_label TEXT NOT NULL DEFAULT '');
            CREATE TABLE usage_bucket_30m(
                source TEXT NOT NULL,
                provider_label TEXT NOT NULL DEFAULT '',
                model TEXT NOT NULL,
                hour_start TEXT NOT NULL,
                project_hash TEXT NOT NULL DEFAULT '',
                project_label TEXT NOT NULL DEFAULT '',
                project_ref TEXT,
                input_tokens INTEGER NOT NULL,
                cache_read_tokens INTEGER NOT NULL,
                cache_creation_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                reasoning_output_tokens INTEGER NOT NULL,
                total_tokens INTEGER NOT NULL,
                event_count INTEGER NOT NULL,
                cost_with_cache_usd REAL NOT NULL,
                cost_without_cache_usd REAL NOT NULL,
                pricing_status TEXT NOT NULL,
                pricing_source TEXT,
                pricing_rate TEXT
            );
            INSERT INTO usage_bucket_30m VALUES
              ('codex', 'openai', 'gpt-5', '2026-07-01T00:00:00Z', 'p1', 'Project 1', NULL, 40, 10, 5, 30, 15, 100, 2, 0.10, 0.15, 'priced', 'catalog', 'rate-a'),
              ('codex', 'openai', 'gpt-4', '2026-07-01T00:30:00Z', 'p1', 'Project 1', NULL, 10, 0, 0, 5, 5, 20, 1, 0.02, 0.03, 'priced', 'catalog', 'rate-b'),
              ('claude', 'anthropic', 'claude-sonnet', '2026-07-01T01:00:00Z', 'p2', 'Project 2', NULL, 20, 0, 0, 25, 5, 50, 1, 0.20, 0.25, 'priced', 'catalog', 'rate-c'),
              ('codex', '', 'gpt-5', '2026-07-01T01:30:00Z', 'p3', 'Project 3', NULL, 10, 0, 0, 10, 10, 30, 1, 0.03, 0.04, 'priced', 'catalog', 'rate-a');
            "#,
        )
        .expect("provider fixture should be created");
        drop(conn);

        Dashboard::open(AppPaths::from_root(root)).expect("dashboard should open test db")
    }

    #[test]
    fn provider_breakdown_sums_to_source_totals_and_marks_unattributed() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let dashboard = create_provider_fixture(temp.path());

        let provider_stats = dashboard
            .provider_breakdown(&QueryFilter::default())
            .expect("provider breakdown should query");
        let source_total = dashboard
            .source_breakdown(&QueryFilter::default())
            .expect("source breakdown should query")
            .iter()
            .map(|row| row.total_tokens)
            .sum::<i64>();

        assert_eq!(
            provider_stats
                .iter()
                .map(|row| row.total_tokens)
                .sum::<i64>(),
            source_total
        );
        assert_eq!(source_total, 200);
        let openai = provider_stats
            .iter()
            .find(|row| row.provider.as_deref() == Some("openai"))
            .expect("openai row should exist");
        assert_eq!(openai.total_tokens, 120);
        assert_eq!(openai.request_count, 3);
        let unattributed = provider_stats
            .iter()
            .find(|row| row.provider.is_none())
            .expect("empty provider label should map to unattributed");
        assert_eq!(unattributed.total_tokens, 30);
    }

    #[test]
    fn dashboard_sections_reuse_the_open_connection_capability_snapshot() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        reset_capability_connection_open_count();
        let dashboard = create_provider_fixture(temp.path());

        dashboard
            .overview(&QueryFilter::default())
            .expect("overview should query");
        dashboard
            .model_breakdown(&QueryFilter::default())
            .expect("model breakdown should query");
        dashboard
            .provider_breakdown(&QueryFilter::default())
            .expect("provider breakdown should query");

        assert_eq!(capability_connection_open_count(), 0);
    }

    #[test]
    fn provider_breakdown_respects_source_filter() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let dashboard = create_provider_fixture(temp.path());

        let stats = dashboard
            .provider_breakdown(&QueryFilter {
                source: Some(SourceKind::Claude),
                ..QueryFilter::default()
            })
            .expect("provider breakdown should query");

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].provider.as_deref(), Some("anthropic"));
        assert_eq!(stats[0].total_tokens, 50);
    }

    #[test]
    fn provider_breakdown_by_source_tags_rows_per_platform() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let dashboard = create_provider_fixture(temp.path());

        let rows = dashboard
            .provider_breakdown_by_source(
                &[SourceKind::Claude, SourceKind::Codex],
                &QueryFilter::default(),
            )
            .expect("tagged provider breakdown should query");

        let claude_rows: Vec<_> = rows
            .iter()
            .filter(|row| row.source == SourceKind::Claude)
            .collect();
        let codex_rows: Vec<_> = rows
            .iter()
            .filter(|row| row.source == SourceKind::Codex)
            .collect();
        assert_eq!(claude_rows.len(), 1);
        assert_eq!(
            claude_rows[0].breakdown.provider.as_deref(),
            Some("anthropic")
        );
        assert_eq!(codex_rows.len(), 2);
        assert_eq!(
            codex_rows
                .iter()
                .map(|row| row.breakdown.total_tokens)
                .sum::<i64>(),
            150
        );
        assert!(
            codex_rows
                .iter()
                .any(|row| row.breakdown.provider.is_none()),
            "codex unattributed row must keep its tag"
        );
    }

    #[test]
    fn provider_filter_narrows_overview_model_and_source_totals() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let dashboard = create_provider_fixture(temp.path());
        let filter = QueryFilter {
            provider: Some("openai".to_string()),
            ..QueryFilter::default()
        };

        let overview = dashboard
            .overview(&filter)
            .expect("overview should accept provider filter");
        assert_eq!(overview.total.total_tokens, 120);
        assert_eq!(overview.total_events, 3);

        let model_total = dashboard
            .model_breakdown(&filter)
            .expect("model breakdown should accept provider filter")
            .iter()
            .map(|row| row.total_tokens)
            .sum::<i64>();
        assert_eq!(model_total, 120);

        let source_total = dashboard
            .source_breakdown(&filter)
            .expect("source breakdown should accept provider filter")
            .iter()
            .map(|row| row.total_tokens)
            .sum::<i64>();
        assert_eq!(source_total, 120);
    }

    #[test]
    fn provider_filter_can_match_unattributed_rows() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let dashboard = create_provider_fixture(temp.path());
        let filter = QueryFilter {
            provider: Some(String::new()),
            ..QueryFilter::default()
        };

        let stats = dashboard
            .provider_breakdown(&filter)
            .expect("empty provider filter should query unattributed rows");
        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].provider, None);
        assert_eq!(stats[0].total_tokens, 30);

        let overview = dashboard
            .overview(&filter)
            .expect("empty provider filter should narrow overview too");
        assert_eq!(overview.total.total_tokens, 30);
        assert_eq!(overview.total_events, 1);
    }

    #[test]
    fn provider_breakdown_rejects_schema_below_14() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let db_path = temp.path().join("llmusage.db");
        let conn = Connection::open(&db_path).expect("test db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta(key, value) VALUES ('schema_version', '13');
            "#,
        )
        .expect("old schema should be created");
        drop(conn);

        let dashboard = Dashboard::open(AppPaths::from_root(temp.path()))
            .expect("dashboard should open schema 13 DB");
        let error = dashboard
            .provider_breakdown(&QueryFilter::default())
            .expect_err("provider breakdown should reject old schema");

        assert!(matches!(
            error,
            UsageError::SchemaUnsupported {
                expected: 14,
                actual: Some(13)
            }
        ));
    }

    #[test]
    fn provider_breakdown_rejects_missing_provider_label_column() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let db_path = temp.path().join("llmusage.db");
        let conn = Connection::open(&db_path).expect("test db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta(key, value) VALUES ('schema_version', '14');
            CREATE TABLE usage_bucket_30m(total_tokens INTEGER NOT NULL);
            CREATE TABLE usage_event(event_key TEXT NOT NULL);
            "#,
        )
        .expect("schema 14 missing provider columns should be created");
        drop(conn);

        let dashboard = Dashboard::open(AppPaths::from_root(temp.path()))
            .expect("dashboard should open schema 14 DB");
        let error = dashboard
            .provider_breakdown(&QueryFilter::default())
            .expect_err("provider breakdown should require provider_label");

        assert!(matches!(
            error,
            UsageError::FeatureUnavailable {
                feature: "provider_breakdown",
                ..
            }
        ));
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

        let dashboard = Dashboard::open(AppPaths::from_root(temp.path()))
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
    fn antigravity_filter_reads_both_sides_of_schema_13_cutover() {
        for (schema_version, stored_source) in [(10, "gemini"), (13, "antigravity")] {
            let temp = tempfile::TempDir::new().expect("temp dir should be created");
            let db_path = temp.path().join("llmusage.db");
            let conn = Connection::open(&db_path).expect("test db should open");
            conn.execute_batch(&format!(
                r#"
                CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
                INSERT INTO meta(key, value) VALUES ('schema_version', '{schema_version}');
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
                  ('{stored_source}', '2026-05-21T00:00:00Z', 10, 0, 0, 5, 0, 15, 1, 0.30);
                "#
            ))
            .expect("cutover fixture should be created");
            drop(conn);

            let dashboard = Dashboard::open(AppPaths::from_root(temp.path()))
                .expect("cutover fixture dashboard should open");
            let rows = dashboard
                .trends_daily(&QueryFilter {
                    source: Some(SourceKind::Antigravity),
                    timezone: ReportTimezone::Utc,
                    ..QueryFilter::default()
                })
                .expect("cutover trend query should succeed");
            assert_eq!(rows.len(), 1, "schema {schema_version}");
            assert_eq!(rows[0].total_tokens, 15, "schema {schema_version}");
        }
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

        let dashboard = Dashboard::open(AppPaths::from_root(temp.path()))
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
            vec!["legacy", "antigravity", "codex", "claude"]
        );
        assert_eq!(stats.iter().map(|row| row.total_tokens).sum::<i64>(), 705);
        assert_eq!(
            stats
                .iter()
                .find(|row| row.source == "codex")
                .expect("codex row should exist")
                .event_count,
            2
        );
        assert!((stats.iter().map(|row| row.share_tokens).sum::<f64>() - 1.0).abs() < f64::EPSILON);
        assert!(stats.iter().all(|row| row.source != "opencode"));
    }

    #[test]
    fn home_overview_aggregates_platform_totals_and_series() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let dashboard = create_provider_fixture(temp.path());

        let payload = dashboard
            .home_overview(&QueryFilter::default())
            .expect("home overview should query");

        assert_eq!(payload.summary.total_tokens, 200);
        assert_eq!(payload.summary.total_requests, 5);
        assert_eq!(payload.summary.platforms, 2);
        for (platform, expected_tokens) in [
            ("codex", 150),
            ("claude", 50),
            ("antigravity", 0),
            ("kimi_code", 0),
            ("pi", 0),
            ("grok", 0),
        ] {
            assert_eq!(
                payload
                    .by_platform
                    .get(platform)
                    .expect("platform key should exist")
                    .tokens,
                expected_tokens
            );
        }
        assert_eq!(payload.series.len(), 1);
        assert_eq!(payload.series[0].codex.tokens, 150);
    }

    fn create_logs_fixture(root: &std::path::Path) -> Dashboard {
        let db_path = root.join("llmusage.db");
        let conn = Connection::open(&db_path).expect("test db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta(key, value) VALUES ('schema_version', '10');
            CREATE TABLE usage_event(
                event_key TEXT NOT NULL,
                source TEXT NOT NULL,
                model TEXT NOT NULL,
                event_at TEXT NOT NULL,
                input_tokens INTEGER NOT NULL,
                cache_read_tokens INTEGER NOT NULL,
                cache_creation_tokens INTEGER NOT NULL,
                output_tokens INTEGER NOT NULL,
                reasoning_output_tokens INTEGER NOT NULL,
                total_tokens INTEGER NOT NULL,
                project_hash TEXT NOT NULL DEFAULT '',
                project_label TEXT NOT NULL DEFAULT '',
                project_ref TEXT,
                project_path TEXT,
                cost_with_cache_usd REAL NOT NULL,
                cost_without_cache_usd REAL NOT NULL,
                pricing_status TEXT NOT NULL,
                pricing_source TEXT
            );
            INSERT INTO usage_event VALUES
              ('ev-3', 'codex', 'gpt-5', '2026-07-01T02:00:00Z', 10, 0, 0, 5, 1, 16, 'p1', 'Project 1', NULL, '/repo/p1', 0.03, 0.04, 'priced', 'catalog'),
              ('ev-2', 'gemini', 'gemini-pro', '2026-07-01T01:00:00Z', 10, 0, 0, 5, 1, 16, 'p1', 'Project 1', NULL, '/repo/p1', 0.03, 0.04, 'priced', 'catalog'),
              ('ev-1', 'claude', 'claude-sonnet', '2026-07-01T00:00:00Z', 10, 0, 0, 5, 1, 16, 'p2', 'Project 2', NULL, NULL, 0.03, 0.04, 'priced', 'catalog');
            "#,
        )
        .expect("logs fixture should be created");
        drop(conn);

        Dashboard::open(AppPaths::from_root(root)).expect("dashboard should open test db")
    }

    #[test]
    fn logs_paginate_with_cursor_and_expose_total() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let dashboard = create_logs_fixture(temp.path());

        let first_page = dashboard
            .logs(&LogsQuery {
                filter: QueryFilter::default(),
                page_size: 2,
                cursor: None,
                include_total: true,
                include_raw_json: false,
            })
            .expect("logs should query");

        assert_eq!(first_page.records.len(), 2);
        assert_eq!(first_page.total, Some(3));
        assert_eq!(first_page.records[0].id, "ev-3");
        assert_eq!(first_page.records[0].output_tokens, 6);
        assert_eq!(first_page.records[0].project_path, "/repo/p1");
        assert_eq!(first_page.records[1].id, "ev-2");
        assert_eq!(first_page.records[1].platform, "antigravity");
        assert_eq!(first_page.records[1].source_id, "antigravity");
        let cursor = first_page.next_cursor.expect("first page must have cursor");

        let second_page = dashboard
            .logs(&LogsQuery {
                filter: QueryFilter::default(),
                page_size: 2,
                cursor: Some(cursor),
                include_total: false,
                include_raw_json: false,
            })
            .expect("second page should query");

        assert_eq!(second_page.records.len(), 1);
        assert_eq!(second_page.records[0].id, "ev-1");
        // project_path 为 NULL 时回退 project_label 链
        assert_eq!(second_page.records[0].project_path, "Project 2");
        assert_eq!(second_page.next_cursor, None);
        assert_eq!(second_page.total, None);
    }

    #[test]
    fn diagnostics_aggregate_file_states_by_source() {
        let temp = tempfile::TempDir::new().expect("temp dir should be created");
        let db_path = temp.path().join("llmusage.db");
        let conn = Connection::open(&db_path).expect("test db should open");
        conn.execute_batch(
            r#"
            CREATE TABLE meta(key TEXT PRIMARY KEY, value TEXT NOT NULL);
            INSERT INTO meta(key, value) VALUES ('schema_version', '10');
            CREATE TABLE source_file(source TEXT NOT NULL, state TEXT NOT NULL);
            CREATE TABLE source_sync_status(
                source TEXT NOT NULL,
                recent_completed_at TEXT,
                history_completed_at TEXT
            );
            INSERT INTO source_file VALUES
              ('claude', 'live'), ('claude', 'live'), ('claude', 'missing'),
              ('gemini', 'live'), ('gemini', 'deleted_by_user');
            INSERT INTO source_sync_status VALUES
              ('claude', '2026-07-01T00:00:00Z', '2026-06-30T00:00:00Z');
            "#,
        )
        .expect("diagnostics fixture should be created");
        drop(conn);

        let dashboard = Dashboard::open(AppPaths::from_root(temp.path()))
            .expect("dashboard should open test db");
        let payload = dashboard.diagnostics().expect("diagnostics should query");

        assert_eq!(payload.by_source.len(), 2);
        let antigravity = &payload.by_source[0];
        assert_eq!(antigravity.source, "antigravity");
        assert_eq!(antigravity.live_files, 1);
        assert_eq!(antigravity.deleted_files, 1);
        assert_eq!(antigravity.recent_completed_at, None);
        let claude = &payload.by_source[1];
        assert_eq!(claude.source, "claude");
        assert_eq!(claude.live_files, 2);
        assert_eq!(claude.missing_files, 1);
        assert_eq!(
            claude.recent_completed_at.as_deref(),
            Some("2026-07-01T00:00:00Z")
        );
    }
}
