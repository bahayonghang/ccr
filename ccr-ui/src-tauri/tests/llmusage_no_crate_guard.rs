use std::fs;
use std::path::Path;

#[test]
fn ccr_ui_tauri_does_not_depend_on_llmusage_crate() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("Cargo.toml should be readable");
    for line in manifest.lines() {
        let trimmed = line.trim_start();
        assert!(
            !trimmed.starts_with("llmusage ="),
            "ccr-ui must not depend on the llmusage Rust crate: {trimmed}"
        );
    }
}

#[test]
fn ccr_ui_tauri_source_does_not_import_llmusage_crate() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut stack = vec![src];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("source directory should be readable") {
            let entry = entry.expect("source entry should be readable");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|value| value.to_str()) != Some("rs") {
                continue;
            }
            let content = fs::read_to_string(&path).expect("source file should be readable");
            let forbidden = ["llmusage", "::"].concat();
            assert!(
                !content.contains(&forbidden),
                "{} must not import/use the llmusage Rust crate",
                path.display()
            );
        }
    }
}
