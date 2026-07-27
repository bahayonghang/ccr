use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

fn registry_commands() -> Result<&'static [&'static str], Box<dyn Error>> {
    let manifest_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../src/api/generated/command-manifest.json");
    println!("cargo:rerun-if-changed={}", manifest_path.display());

    let manifest: serde_json::Value = serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
    if manifest
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        != Some(2)
    {
        return Err("command manifest schema_version must be 2".into());
    }

    let expected_count = manifest
        .get("windows_command_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or("command manifest is missing windows_command_count")?
        as usize;
    let entries = manifest
        .get("commands")
        .and_then(serde_json::Value::as_array)
        .ok_or("command manifest is missing commands")?;
    if entries.len() != expected_count {
        return Err(format!(
            "command manifest count mismatch: expected {expected_count}, found {}",
            entries.len()
        )
        .into());
    }

    let mut unique = HashSet::with_capacity(entries.len());
    let mut commands = Vec::with_capacity(entries.len());
    for entry in entries {
        let command = entry
            .get("id")
            .and_then(serde_json::Value::as_str)
            .filter(|command| !command.is_empty())
            .ok_or("command manifest contains an empty command id")?;
        if !unique.insert(command) {
            return Err(
                format!("command manifest contains duplicate command id: {command}").into(),
            );
        }
        commands.push(command.to_owned());
    }

    let commands = commands
        .into_iter()
        .map(|command| Box::leak(command.into_boxed_str()) as &'static str)
        .collect::<Vec<_>>();
    Ok(Box::leak(commands.into_boxed_slice()))
}

fn main() -> Result<(), Box<dyn Error>> {
    let app_manifest = tauri_build::AppManifest::new().commands(registry_commands()?);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(app_manifest))?;
    Ok(())
}
