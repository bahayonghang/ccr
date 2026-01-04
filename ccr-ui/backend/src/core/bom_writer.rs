//! UTF-8 BOM Writer Module
//!
//! Provides a writer wrapper that automatically adds UTF-8 BOM (Byte Order Mark)
//! at the beginning of each new log file. This ensures Windows text editors
//! (like Notepad) correctly identify the file as UTF-8 encoded.
//!
//! The UTF-8 BOM is the byte sequence: 0xEF 0xBB 0xBF

use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tracing_appender::rolling::RollingFileAppender;

/// UTF-8 BOM bytes
const UTF8_BOM: [u8; 3] = [0xEF, 0xBB, 0xBF];

/// A MakeWriter implementation that produces BOM-aware writers
pub struct MakeBomWriter {
    appender: Arc<Mutex<RollingFileAppender>>,
    log_dir: PathBuf,
    bom_written: Arc<AtomicBool>,
}

impl MakeBomWriter {
    /// Create a new MakeBomWriter
    pub fn new(appender: RollingFileAppender, log_dir: impl AsRef<Path>) -> Self {
        Self {
            appender: Arc::new(Mutex::new(appender)),
            log_dir: log_dir.as_ref().to_path_buf(),
            bom_written: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for MakeBomWriter {
    type Writer = BomWriterHandle;

    fn make_writer(&'a self) -> Self::Writer {
        BomWriterHandle {
            appender: self.appender.clone(),
            log_dir: self.log_dir.clone(),
            bom_written: self.bom_written.clone(),
        }
    }
}

/// Writer handle returned by MakeBomWriter
pub struct BomWriterHandle {
    appender: Arc<Mutex<RollingFileAppender>>,
    log_dir: PathBuf,
    bom_written: Arc<AtomicBool>,
}

impl Write for BomWriterHandle {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut appender = self
            .appender
            .lock()
            .map_err(|e| io::Error::other(format!("Failed to lock appender: {}", e)))?;

        // Check if we need to write BOM first
        if self
            .bom_written
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            // First write in this session - check if file needs BOM
            if needs_bom_for_dir(&self.log_dir) {
                // Write BOM first
                if let Err(e) = appender.write_all(&UTF8_BOM) {
                    // Log warning but continue - BOM is nice-to-have
                    eprintln!("Warning: Failed to write UTF-8 BOM: {}", e);
                }
            }
        }

        // Write the actual content
        appender.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut appender = self
            .appender
            .lock()
            .map_err(|e| io::Error::other(format!("Failed to lock appender: {}", e)))?;
        appender.flush()
    }
}

/// Check if any file in the directory needs BOM
fn needs_bom_for_dir(log_dir: &Path) -> bool {
    if let Ok(entries) = std::fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && let Ok(contents) = std::fs::read(&path)
            {
                // If file has content and already has BOM, no need to add more
                if contents.len() >= 3 && contents[0..3] == UTF8_BOM {
                    // Found a file with BOM - existing files are OK
                    return false;
                }
            }
        }
    }
    // No files with BOM found, or directory is empty - add BOM to new files
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_utf8_bom_constant() {
        assert_eq!(UTF8_BOM, [0xEF, 0xBB, 0xBF]);
    }

    #[test]
    fn test_needs_bom_empty_dir() {
        let temp = tempdir().expect("Failed to create temp dir");
        assert!(needs_bom_for_dir(temp.path()));
    }

    #[test]
    fn test_needs_bom_with_bom_file() {
        let temp = tempdir().expect("Failed to create temp dir");
        let file_path = temp.path().join("test.log");

        // Write a file with BOM
        let mut content = Vec::from(UTF8_BOM);
        content.extend_from_slice(b"test content");
        std::fs::write(&file_path, content).expect("Failed to write");

        assert!(!needs_bom_for_dir(temp.path()));
    }

    #[test]
    fn test_needs_bom_without_bom_file() {
        let temp = tempdir().expect("Failed to create temp dir");
        let file_path = temp.path().join("test.log");

        // Write a file without BOM
        std::fs::write(&file_path, b"test content without BOM").expect("Failed to write");

        // Empty files or files without BOM mean we should add BOM to new writes
        assert!(needs_bom_for_dir(temp.path()));
    }
}
