#![allow(clippy::unused_async)]

use super::types::{PlatformListItem, PlatformListOutput};
use crate::managers::PlatformConfigManager;
use crate::platforms::PlatformRegistry;
use ccr_core::core::error::Result;
use ccr_core::core::logging::ColorOutput;
use comfy_table::{
    Attribute, Cell, CellAlignment, Color as TableColor, ColumnConstraint, ContentArrangement,
    Table, presets::UTF8_FULL,
};

pub async fn platform_list_command(json: bool) -> Result<()> {
    let manager = PlatformConfigManager::with_default()?;
    let config = manager.load_or_create_default()?;
    let registry = PlatformRegistry::new();
    let all_platforms = registry.list_platform_info();

    let mut platforms_data = Vec::new();
    for platform_info in &all_platforms {
        let platform_name = &platform_info.short_name;
        let registry_entry = config.platforms.get(platform_name);
        let enabled = registry_entry.map(|entry| entry.enabled).unwrap_or(false);
        let current_profile = registry_entry.and_then(|entry| entry.current_profile.clone());
        let description = registry_entry
            .and_then(|entry| entry.description.clone())
            .unwrap_or_else(|| platform_info.name.to_string());

        platforms_data.push(PlatformListItem {
            name: platform_name.clone(),
            enabled,
            current_profile,
            description,
        });
    }

    if json {
        let output = PlatformListOutput {
            config_file: manager.config_path().display().to_string(),
            platforms: platforms_data,
        };
        println!("{}", serde_json::to_string_pretty(&output)?);
        return Ok(());
    }

    ColorOutput::title("Platform registry");
    println!();
    ColorOutput::info(&format!("Config file: {}", manager.config_path().display()));
    ColorOutput::info(
        "Profile routing is tracked per platform; legacy current_platform is ignored.",
    );
    println!();

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_header(vec![
            Cell::new("State")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("Platform")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("Enabled")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("Current Profile")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
            Cell::new("Description")
                .add_attribute(Attribute::Bold)
                .fg(TableColor::Cyan),
        ]);

    for platform in &platforms_data {
        let status = if platform.current_profile.is_some() {
            Cell::new("profile").fg(TableColor::Green)
        } else {
            Cell::new("")
        };

        let enabled = if platform.enabled {
            Cell::new("OK")
                .fg(TableColor::Green)
                .add_attribute(Attribute::Bold)
        } else {
            Cell::new("X").fg(TableColor::Red)
        };

        table.add_row(vec![
            status,
            Cell::new(&platform.name),
            enabled,
            Cell::new(platform.current_profile.as_deref().unwrap_or("-")),
            Cell::new(&platform.description).fg(TableColor::Blue),
        ]);
    }

    if let Some(column) = table.column_mut(2) {
        column.set_constraint(ColumnConstraint::ContentWidth);
        column.set_cell_alignment(CellAlignment::Center);
    }

    println!("{table}");
    println!();
    ColorOutput::success(&format!("Found {} platforms", platforms_data.len()));
    println!();
    ColorOutput::info("Hints:");
    println!("  - Use 'ccr current' to view Claude/Codex runtime status");
    println!("  - Use 'ccr claude profile list' or 'ccr codex profile list' for profiles");

    Ok(())
}
