use crate::core::error::Result;
use crate::managers::conflict_checker::{ConflictChecker, ConflictSeverity};
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table, presets::UTF8_FULL};

pub fn check_conflicts_command() -> Result<()> {
    println!("🔍 Checking for environment variable conflicts across platforms...\n");

    let checker = ConflictChecker::new();
    let report = checker.check_conflicts()?;

    // Display warnings
    if !report.warnings.is_empty() {
        println!("⚠️  Warnings:");
        for warning in &report.warnings {
            println!("   {}", warning);
        }
        println!();
    }

    // Display conflicts
    if report.conflicts.is_empty() {
        println!("✅ No conflicts detected.");
        return Ok(());
    }

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Key").add_attribute(Attribute::Bold),
            Cell::new("Platform").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
            Cell::new("Severity").add_attribute(Attribute::Bold),
        ]);

    let mut critical_count = 0;
    let mut warning_count = 0;
    let mut info_count = 0;

    for conflict in &report.conflicts {
        let severity_color = match conflict.severity {
            ConflictSeverity::Critical => {
                critical_count += 1;
                Color::Red
            }
            ConflictSeverity::Warning => {
                warning_count += 1;
                Color::Yellow
            }
            ConflictSeverity::Info => {
                info_count += 1;
                Color::Green
            }
        };

        for (i, pv) in conflict.platforms.iter().enumerate() {
            let key_cell = if i == 0 {
                Cell::new(&conflict.key).fg(Color::Cyan)
            } else {
                Cell::new("")
            };

            let severity_cell = if i == 0 {
                Cell::new(format!("{:?}", conflict.severity)).fg(severity_color)
            } else {
                Cell::new("")
            };

            table.add_row(vec![
                key_cell,
                Cell::new(&pv.platform),
                Cell::new(&pv.value),
                severity_cell,
            ]);
        }

        // Add suggestion row
        table.add_row(vec![
            Cell::new(""),
            Cell::new("Suggestion").add_attribute(Attribute::Italic),
            Cell::new(&conflict.suggestion)
                .add_attribute(Attribute::Italic)
                .fg(Color::Grey),
            Cell::new(""),
        ]);
    }

    println!("{}", table);

    println!("\n📊 Summary:");
    if critical_count > 0 {
        println!("   🔴 Critical conflicts: {}", critical_count);
    }
    if warning_count > 0 {
        println!("   🟡 Warnings: {}", warning_count);
    }
    if info_count > 0 {
        println!("   🟢 Info: {}", info_count);
    }

    Ok(())
}
