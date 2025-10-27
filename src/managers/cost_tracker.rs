// 💰 CCR 成本追踪管理器
// 负责记录和查询 API 使用成本

use crate::core::error::{CcrError, Result};
use crate::models::stats::{
    Cost, CostRecord, CostStats, DailyCost, ModelPricing, TokenStats, TokenUsage,
};
use chrono::{Datelike, DateTime, Duration, Utc};
use std::collections::HashMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// 💰 成本追踪管理器
pub struct CostTracker {
    /// 📁 存储目录
    storage_dir: PathBuf,
    
    /// 💲 模型定价表
    #[allow(dead_code)]  // 预留用于自定义定价，当前使用 ModelPricing::default_pricing
    pricing: HashMap<String, ModelPricing>,
}

impl CostTracker {
    /// 创建新的成本追踪器
    pub fn new(storage_dir: PathBuf) -> Result<Self> {
        // 确保存储目录存在
        if !storage_dir.exists() {
            fs::create_dir_all(&storage_dir)?;
        }
        
        Ok(Self {
            storage_dir,
            pricing: ModelPricing::default_pricing(),
        })
    }
    
    /// 获取默认存储目录
    pub fn default_storage_dir() -> Result<PathBuf> {
        let home = dirs::home_dir()
            .ok_or_else(|| CcrError::ConfigError("无法获取用户主目录".to_string()))?;
        Ok(home.join(".claude").join("stats"))
    }
    
    /// 记录成本
    #[allow(dead_code)]  // 预留用于实际 API 调用时记录成本
    pub fn record(
        &self,
        session_id: Option<String>,
        project: String,
        model: String,
        token_usage: TokenUsage,
        duration_ms: u64,
        platform: Option<String>,
        description: Option<String>,
    ) -> Result<CostRecord> {
        // 计算成本
        let cost = self.calculate_cost(&model, &token_usage)?;
        
        // 创建记录
        let record = CostRecord {
            id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
            session_id,
            project,
            model,
            token_usage,
            cost,
            duration_ms,
            platform,
            description,
        };
        
        // 保存到 CSV
        self.save_to_csv(&record)?;
        
        Ok(record)
    }
    
    /// 计算成本
    #[allow(dead_code)]  // 在 record 方法中使用
    pub fn calculate_cost(&self, model: &str, usage: &TokenUsage) -> Result<Cost> {
        // 查找模型定价
        let pricing = self.pricing.get(model).ok_or_else(|| {
            CcrError::ValidationError(format!("未知模型: {}", model))
        })?;
        
        Ok(pricing.calculate_cost(usage))
    }
    
    /// 保存到 CSV 文件
    fn save_to_csv(&self, record: &CostRecord) -> Result<()> {
        let csv_path = self.get_csv_path()?;
        
        // 检查文件是否存在，如果不存在则创建并写入表头
        let file_exists = csv_path.exists();
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&csv_path)?;
        
        // 如果是新文件，写入 CSV 表头
        if !file_exists {
            writeln!(
                file,
                "timestamp,id,session_id,project,platform,model,input_tokens,output_tokens,\
                cache_read_tokens,cache_write_tokens,input_cost,output_cost,cache_cost,\
                total_cost,duration_ms,description"
            )?;
        }
        
        // 写入记录
        writeln!(
            file,
            "{},{},{},{},{},{},{},{},{},{},{:.6},{:.6},{:.6},{:.6},{},{}",
            record.timestamp.to_rfc3339(),
            record.id,
            record.session_id.as_deref().unwrap_or(""),
            record.project,
            record.platform.as_deref().unwrap_or(""),
            record.model,
            record.token_usage.input_tokens,
            record.token_usage.output_tokens,
            record.token_usage.cache_read_tokens.unwrap_or(0),
            record.token_usage.cache_creation_tokens.unwrap_or(0),
            record.cost.input_cost,
            record.cost.output_cost,
            record.cost.cache_cost,
            record.cost.total_cost,
            record.duration_ms,
            record.description.as_deref().unwrap_or("")
        )?;
        
        Ok(())
    }
    
    /// 获取 CSV 文件路径（当前月份）
    fn get_csv_path(&self) -> Result<PathBuf> {
        let now = Utc::now();
        let filename = format!("costs_{}.csv", now.format("%Y%m"));
        Ok(self.storage_dir.join(filename))
    }
    
    /// 读取所有成本记录
    pub fn read_all(&self) -> Result<Vec<CostRecord>> {
        let mut records = Vec::new();
        
        // 读取所有 costs_*.csv 文件
        let entries = fs::read_dir(&self.storage_dir)?;
        
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name() {
                if filename.to_string_lossy().starts_with("costs_")
                    && filename.to_string_lossy().ends_with(".csv")
                {
                    records.extend(self.read_csv_file(&path)?);
                }
            }
        }
        
        // 按时间排序
        records.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        
        Ok(records)
    }
    
    /// 读取单个 CSV 文件
    fn read_csv_file(&self, path: &Path) -> Result<Vec<CostRecord>> {
        let file = fs::File::open(path)?;
        
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        
        for (i, line) in reader.lines().enumerate() {
            // 跳过表头
            if i == 0 {
                continue;
            }
            
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            
            let record = self.parse_csv_line(&line)?;
            records.push(record);
        }
        
        Ok(records)
    }
    
    /// 解析 CSV 行
    fn parse_csv_line(&self, line: &str) -> Result<CostRecord> {
        let parts: Vec<&str> = line.split(',').collect();
        
        if parts.len() < 16 {
            return Err(CcrError::ValidationError("CSV 行格式不正确".to_string()));
        }
        
        let timestamp = DateTime::parse_from_rfc3339(parts[0])
            .map_err(|_| CcrError::ValidationError("时间戳格式不正确".to_string()))?
            .with_timezone(&Utc);
        
        let session_id = if parts[2].is_empty() {
            None
        } else {
            Some(parts[2].to_string())
        };
        
        let platform = if parts[4].is_empty() {
            None
        } else {
            Some(parts[4].to_string())
        };
        
        let description = if parts[15].is_empty() {
            None
        } else {
            Some(parts[15].to_string())
        };
        
        Ok(CostRecord {
            id: parts[1].to_string(),
            timestamp,
            session_id,
            project: parts[3].to_string(),
            platform,
            model: parts[5].to_string(),
            token_usage: TokenUsage {
                input_tokens: parts[6].parse().unwrap_or(0),
                output_tokens: parts[7].parse().unwrap_or(0),
                cache_read_tokens: parts[8].parse().ok(),
                cache_creation_tokens: parts[9].parse().ok(),
            },
            cost: Cost {
                input_cost: parts[10].parse().unwrap_or(0.0),
                output_cost: parts[11].parse().unwrap_or(0.0),
                cache_cost: parts[12].parse().unwrap_or(0.0),
                total_cost: parts[13].parse().unwrap_or(0.0),
            },
            duration_ms: parts[14].parse().unwrap_or(0),
            description,
        })
    }
    
    /// 按时间范围筛选
    pub fn filter_by_time_range(
        &self,
        records: &[CostRecord],
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Vec<CostRecord> {
        records
            .iter()
            .filter(|r| r.timestamp >= start && r.timestamp <= end)
            .cloned()
            .collect()
    }
    
    /// 获取今日成本
    #[allow(dead_code)]  // 预留用于统计功能
    pub fn get_today_cost(&self) -> Result<f64> {
        let now = Utc::now();
        let start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end = now;
        
        let records = self.read_all()?;
        let filtered = self.filter_by_time_range(&records, start, end);
        
        Ok(filtered.iter().map(|r| r.cost.total_cost).sum())
    }
    
    /// 获取本周成本
    #[allow(dead_code)]  // 预留用于统计功能
    pub fn get_week_cost(&self) -> Result<f64> {
        let now = Utc::now();
        let start = now - Duration::days(7);
        
        let records = self.read_all()?;
        let filtered = self.filter_by_time_range(&records, start, now);
        
        Ok(filtered.iter().map(|r| r.cost.total_cost).sum())
    }
    
    /// 获取本月成本
    #[allow(dead_code)]  // 预留用于统计功能
    pub fn get_month_cost(&self) -> Result<f64> {
        let now = Utc::now();
        let start = now.date_naive().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc();
        
        let records = self.read_all()?;
        let filtered = self.filter_by_time_range(&records, start, now);
        
        Ok(filtered.iter().map(|r| r.cost.total_cost).sum())
    }
    
    /// 生成成本统计
    pub fn generate_stats(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<CostStats> {
        let records = self.read_all()?;
        let filtered = self.filter_by_time_range(&records, start, end);
        
        if filtered.is_empty() {
            return Ok(CostStats {
                total_cost: 0.0,
                record_count: 0,
                token_stats: TokenStats {
                    total_input_tokens: 0,
                    total_output_tokens: 0,
                    total_cache_tokens: 0,
                    cache_efficiency: 0.0,
                },
                by_model: HashMap::new(),
                by_project: HashMap::new(),
                trend: None,
            });
        }
        
        // 计算总成本
        let total_cost: f64 = filtered.iter().map(|r| r.cost.total_cost).sum();
        
        // 计算 Token 统计
        let total_input_tokens: u64 = filtered
            .iter()
            .map(|r| r.token_usage.input_tokens as u64)
            .sum();
        let total_output_tokens: u64 = filtered
            .iter()
            .map(|r| r.token_usage.output_tokens as u64)
            .sum();
        let total_cache_read: u64 = filtered
            .iter()
            .map(|r| r.token_usage.cache_read_tokens.unwrap_or(0) as u64)
            .sum();
        let total_cache_write: u64 = filtered
            .iter()
            .map(|r| r.token_usage.cache_creation_tokens.unwrap_or(0) as u64)
            .sum();
        let total_cache_tokens = total_cache_read + total_cache_write;
        
        // 计算 Cache 效率
        let cache_efficiency = if total_cache_tokens > 0 {
            (total_cache_read as f64) / (total_cache_tokens as f64) * 100.0
        } else {
            0.0
        };
        
        // 按模型分组
        let mut by_model: HashMap<String, f64> = HashMap::new();
        for record in &filtered {
            *by_model.entry(record.model.clone()).or_insert(0.0) += record.cost.total_cost;
        }
        
        // 按项目分组
        let mut by_project: HashMap<String, f64> = HashMap::new();
        for record in &filtered {
            *by_project.entry(record.project.clone()).or_insert(0.0) += record.cost.total_cost;
        }
        
        // 生成趋势数据
        let trend = self.generate_daily_trend(&filtered);
        
        Ok(CostStats {
            total_cost,
            record_count: filtered.len(),
            token_stats: TokenStats {
                total_input_tokens,
                total_output_tokens,
                total_cache_tokens,
                cache_efficiency,
            },
            by_model,
            by_project,
            trend: Some(trend),
        })
    }
    
    /// 生成每日趋势
    fn generate_daily_trend(&self, records: &[CostRecord]) -> Vec<DailyCost> {
        let mut daily_map: HashMap<String, (f64, usize)> = HashMap::new();
        
        for record in records {
            let date = record.timestamp.format("%Y-%m-%d").to_string();
            let entry = daily_map.entry(date).or_insert((0.0, 0));
            entry.0 += record.cost.total_cost;
            entry.1 += 1;
        }
        
        let mut trend: Vec<DailyCost> = daily_map
            .into_iter()
            .map(|(date, (cost, count))| DailyCost { date, cost, count })
            .collect();
        
        trend.sort_by(|a, b| a.date.cmp(&b.date));
        
        trend
    }
    
    /// 获取成本最高的会话
    pub fn get_top_sessions(&self, limit: usize) -> Result<Vec<(String, f64)>> {
        let records = self.read_all()?;
        
        // 按会话 ID 分组
        let mut session_costs: HashMap<String, f64> = HashMap::new();
        for record in records {
            if let Some(session_id) = &record.session_id {
                *session_costs.entry(session_id.clone()).or_insert(0.0) += record.cost.total_cost;
            }
        }
        
        // 排序并取前 N 个
        let mut sessions: Vec<(String, f64)> = session_costs.into_iter().collect();
        sessions.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        sessions.truncate(limit);
        
        Ok(sessions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[test]
    fn test_cost_calculation() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = CostTracker::new(temp_dir.path().to_path_buf()).unwrap();
        
        let usage = TokenUsage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: Some(200),
            cache_creation_tokens: Some(100),
        };
        
        let cost = tracker
            .calculate_cost("claude-3-5-sonnet-20241022", &usage)
            .unwrap();
        
        assert!(cost.total_cost > 0.0);
        assert_eq!(cost.total_cost, cost.input_cost + cost.output_cost + cost.cache_cost);
    }
    
    #[test]
    fn test_record_and_read() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = CostTracker::new(temp_dir.path().to_path_buf()).unwrap();
        
        let usage = TokenUsage {
            input_tokens: 1000,
            output_tokens: 500,
            cache_read_tokens: None,
            cache_creation_tokens: None,
        };
        
        tracker
            .record(
                Some("sess_123".to_string()),
                "/path/to/project".to_string(),
                "claude-3-5-sonnet-20241022".to_string(),
                usage,
                1000,
                Some("claude".to_string()),
                Some("Test record".to_string()),
            )
            .unwrap();
        
        let records = tracker.read_all().unwrap();
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].session_id, Some("sess_123".to_string()));
    }
}
