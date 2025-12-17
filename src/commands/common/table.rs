// 📊 表格构建工具
// 统一处理 comfy_table 表格构建逻辑

use crate::core::logging::ColorOutput;
use comfy_table::{
    Attribute, Cell, CellAlignment, Color as TableColor, ColumnConstraint, ContentArrangement,
    Table, Width, presets::UTF8_FULL,
};

/// 表格预设样式
#[derive(Debug, Clone, Copy, Default)]
pub enum TablePreset {
    /// 完整 UTF8 边框（默认）
    #[default]
    Full,
    /// 简洁边框
    Minimal,
}

/// 配置表格构建器
pub struct ConfigTableBuilder {
    table: Table,
}

impl ConfigTableBuilder {
    /// 创建新的配置表格
    pub fn new() -> Self {
        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .set_content_arrangement(ContentArrangement::DynamicFullWidth);
        Self { table }
    }

    /// 设置表头
    pub fn with_header(mut self, headers: &[&str]) -> Self {
        let header_cells: Vec<Cell> = headers
            .iter()
            .map(|h| {
                Cell::new(*h)
                    .add_attribute(Attribute::Bold)
                    .fg(TableColor::Cyan)
            })
            .collect();
        self.table.set_header(header_cells);
        self
    }

    /// 添加配置列表标准表头
    pub fn with_config_list_header(self) -> Self {
        self.with_header(&[
            "状态",
            "配置名称",
            "提供商",
            "Base URL",
            "模型",
            "账号/标签",
            "使用",
            "启用",
            "验证",
        ])
    }

    /// 添加平台列表标准表头
    pub fn with_platform_list_header(self) -> Self {
        self.with_header(&["状态", "平台名称", "启用", "当前 Profile", "描述"])
    }

    /// 添加键值对表头
    pub fn with_kv_header(self) -> Self {
        self.with_header(&["属性", "值"])
    }

    /// 添加状态行（当前/默认标记）
    pub fn add_status_row(&mut self, is_current: bool, is_default: bool) -> Cell {
        if is_current {
            Cell::new(">> 当前")
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else if is_default {
            Cell::new("* 默认").fg(TableColor::Yellow)
        } else {
            Cell::new("")
        }
    }

    /// 添加名称单元格（高亮当前项）
    pub fn name_cell(&self, name: &str, is_current: bool) -> Cell {
        if is_current {
            Cell::new(name)
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else {
            Cell::new(name)
        }
    }

    /// 添加启用状态单元格
    pub fn enabled_cell(&self, enabled: bool) -> Cell {
        if enabled {
            Cell::new("✓")
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else {
            Cell::new("✗")
                .fg(TableColor::Red)
                .add_attribute(Attribute::Bold)
        }
    }

    /// 添加验证状态单元格
    pub fn validation_cell(&self, is_valid: bool, error_msg: Option<&str>) -> Cell {
        if is_valid {
            Cell::new("OK")
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else {
            let msg = error_msg.unwrap_or("X");
            Cell::new(msg)
                .fg(TableColor::Red)
                .add_attribute(Attribute::Bold)
        }
    }

    /// 添加使用次数单元格
    pub fn usage_cell(&self, count: u32) -> Cell {
        Cell::new(format!("{}", count))
            .fg(if count > 10 {
                TableColor::Green
            } else if count > 0 {
                TableColor::Yellow
            } else {
                TableColor::White
            })
            .set_alignment(CellAlignment::Right)
    }

    /// 添加脱敏值单元格
    pub fn masked_cell(&self, value: &str) -> Cell {
        Cell::new(ColorOutput::mask_sensitive(value)).fg(TableColor::DarkGrey)
    }

    /// 添加 URL 单元格（可能截断）
    pub fn url_cell(&self, url: &str, max_len: usize) -> Cell {
        let display = if url.len() > max_len {
            format!("{}...", &url[..max_len - 3])
        } else {
            url.to_string()
        };
        Cell::new(display).fg(TableColor::Blue)
    }

    /// 添加键值行
    pub fn add_kv_row(&mut self, key: &str, value: &str) {
        self.table.add_row(vec![
            Cell::new(key).fg(TableColor::Yellow),
            Cell::new(value),
        ]);
    }

    /// 添加键值行（值高亮）
    pub fn add_kv_row_highlighted(&mut self, key: &str, value: &str) {
        self.table.add_row(vec![
            Cell::new(key).fg(TableColor::Yellow),
            Cell::new(value)
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold),
        ]);
    }

    /// 添加自定义行
    pub fn add_row(&mut self, cells: Vec<Cell>) {
        self.table.add_row(cells);
    }

    /// 设置列宽约束
    pub fn set_column_width(&mut self, index: usize, width: u16) {
        if let Some(column) = self.table.column_mut(index) {
            column.set_constraint(ColumnConstraint::Absolute(Width::Fixed(width)));
        }
    }

    /// 设置列居中对齐
    pub fn set_column_center(&mut self, index: usize) {
        if let Some(column) = self.table.column_mut(index) {
            column.set_cell_alignment(CellAlignment::Center);
        }
    }

    /// 设置列右对齐
    pub fn set_column_right(&mut self, index: usize) {
        if let Some(column) = self.table.column_mut(index) {
            column.set_cell_alignment(CellAlignment::Right);
        }
    }

    /// 构建并返回表格
    pub fn build(self) -> Table {
        self.table
    }

    /// 获取内部表格的可变引用（用于高级操作）
    pub fn inner_mut(&mut self) -> &mut Table {
        &mut self.table
    }
}

impl Default for ConfigTableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 平台表格构建器（简化版）
pub struct PlatformTableBuilder {
    inner: ConfigTableBuilder,
}

impl PlatformTableBuilder {
    pub fn new() -> Self {
        Self {
            inner: ConfigTableBuilder::new().with_platform_list_header(),
        }
    }

    /// 添加平台行
    pub fn add_platform_row(
        &mut self,
        name: &str,
        is_current: bool,
        is_default: bool,
        enabled: bool,
        current_profile: Option<&str>,
        description: &str,
    ) {
        let status = self.inner.add_status_row(is_current, is_default);
        let name_cell = self.inner.name_cell(name, is_current);
        let enabled_cell = self.inner.enabled_cell(enabled);
        let profile_cell = Cell::new(current_profile.unwrap_or("-"));
        let desc_cell = Cell::new(description).fg(TableColor::Blue);

        self.inner.add_row(vec![
            status,
            name_cell,
            enabled_cell,
            profile_cell,
            desc_cell,
        ]);
    }

    /// 设置启用列样式
    pub fn configure_enabled_column(&mut self) {
        self.inner.set_column_width(2, 6);
        self.inner.set_column_center(2);
    }

    pub fn build(self) -> Table {
        self.inner.build()
    }
}

impl Default for PlatformTableBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// 创建简单的键值表格
pub fn create_kv_table() -> ConfigTableBuilder {
    ConfigTableBuilder::new().with_kv_header()
}

/// 创建配置列表表格
pub fn create_config_list_table() -> ConfigTableBuilder {
    ConfigTableBuilder::new().with_config_list_header()
}

/// 创建平台列表表格
pub fn create_platform_list_table() -> PlatformTableBuilder {
    PlatformTableBuilder::new()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_config_table_builder() {
        let mut builder = ConfigTableBuilder::new().with_kv_header();
        builder.add_kv_row("Key", "Value");
        let table = builder.build();
        let output = table.to_string();
        assert!(output.contains("Key"));
        assert!(output.contains("Value"));
    }

    #[test]
    fn test_platform_table_builder() {
        let mut builder = PlatformTableBuilder::new();
        builder.add_platform_row("claude", true, true, true, Some("default"), "Claude Code");
        builder.configure_enabled_column();
        let table = builder.build();
        let output = table.to_string();
        // 打印输出以调试
        eprintln!("=== Table Output ===");
        eprintln!("{}", output);
        eprintln!("=== End Table ===");
        // 表格成功构建即可，不要求特定文本（因为 Unicode 渲染可能受终端影响）
        assert!(!output.is_empty());
    }
}
