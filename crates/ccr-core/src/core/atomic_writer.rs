// 📝 原子写入器
// 提供安全的原子文件写入功能,避免数据损坏

use crate::core::error::{CcrError, Result};
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(windows)]
use std::thread;
#[cfg(windows)]
use std::time::Duration;
#[cfg(windows)]
use std::{ffi::OsStr, iter::once, os::windows::ffi::OsStrExt};
use tempfile::NamedTempFile;
use tokio::fs as async_fs;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

/// 📝 原子写入器
///
/// 使用临时文件 + 原子重命名模式确保文件写入的原子性
///
/// 工作原理:
/// 1. 创建临时文件在同一目录(确保在同一文件系统)
/// 2. 写入内容到临时文件
/// 3. 使用原子 rename 操作替换目标文件
///
/// 优点:
/// - ✅ 即使写入中断,原文件保持完整
/// - ✅ 读取者不会看到部分写入的数据
/// - ✅ 跨平台支持(Unix 和 Windows)
///
/// # Examples
///
/// ```rust,ignore
/// use ccr_core::core::AtomicWriter;
/// use std::path::Path;
///
/// let writer = AtomicWriter::new(Path::new("/path/to/file.json"))?;
/// let content = r#"{"key": "value"}"#;
/// writer.write(content.as_bytes())?;
/// ```
#[allow(dead_code)]
pub struct AtomicWriter {
    target_path: PathBuf,
    secret: bool,
}

/// 📝 异步原子写入器
pub struct AsyncAtomicWriter {
    target_path: PathBuf,
    options: AsyncAtomicWriterOptions,
}

/// Security and metadata policy for asynchronous atomic writes.
#[derive(Debug, Clone, Copy, Default)]
pub struct AsyncAtomicWriterOptions {
    /// Restrict new files to the current user before writing any bytes.
    pub secret: bool,
    /// Preserve an existing target's stricter mode or Windows DACL.
    pub preserve_mode: bool,
}

#[cfg(windows)]
const ATOMIC_WRITE_RETRY_LIMIT: usize = 6;

#[cfg(windows)]
fn is_windows_atomic_retry_error(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::PermissionDenied
            | std::io::ErrorKind::WouldBlock
            | std::io::ErrorKind::Interrupted
    ) || matches!(error.raw_os_error(), Some(5 | 32 | 33))
}

#[cfg(windows)]
const MOVEFILE_REPLACE_EXISTING: u32 = 0x0000_0001;
#[cfg(windows)]
const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn MoveFileExW(existing_file_name: *const u16, new_file_name: *const u16, flags: u32) -> i32;
}

#[cfg(windows)]
fn path_to_wide(path: &Path) -> Vec<u16> {
    OsStr::new(path)
        .encode_wide()
        .chain(once(0))
        .collect::<Vec<_>>()
}

#[cfg(windows)]
fn move_file_replace_existing(source: &Path, target: &Path) -> std::io::Result<()> {
    let source = path_to_wide(source);
    let target = path_to_wide(target);
    let flags = MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH;

    // SAFETY: Pointers come from nul-terminated UTF-16 buffers that live for the
    // duration of the call. MoveFileExW does not retain these pointers.
    let ok = unsafe { MoveFileExW(source.as_ptr(), target.as_ptr(), flags) };
    if ok == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(windows)]
fn retry_windows_atomic_replace<F>(target_path: &Path, mut replace: F) -> std::io::Result<()>
where
    F: FnMut() -> std::io::Result<()>,
{
    for attempt in 0..ATOMIC_WRITE_RETRY_LIMIT {
        match replace() {
            Ok(()) => return Ok(()),
            Err(error) => {
                if !is_windows_atomic_retry_error(&error) || attempt + 1 == ATOMIC_WRITE_RETRY_LIMIT
                {
                    return Err(error);
                }

                tracing::warn!(
                    target_path = %target_path.display(),
                    attempt = attempt + 1,
                    error = %error,
                    "⚠️ Windows 原子替换失败，准备重试"
                );

                thread::sleep(Duration::from_millis(((attempt + 1) * 20) as u64));
            }
        }
    }

    Err(std::io::Error::other(format!(
        "retry limit exhausted for {}",
        target_path.display()
    )))
}

#[cfg(windows)]
fn replace_path_windows(source: &Path, target: &Path) -> std::io::Result<()> {
    retry_windows_atomic_replace(target, || move_file_replace_existing(source, target))
}

#[allow(dead_code)]
impl AtomicWriter {
    /// 🏗️ 创建新的原子写入器
    ///
    /// # Arguments
    /// - `target_path` - 目标文件路径
    ///
    /// # Returns
    /// 新的 `AtomicWriter` 实例
    #[allow(dead_code)]
    pub fn new<P: AsRef<Path>>(target_path: P) -> Self {
        Self {
            target_path: target_path.as_ref().to_path_buf(),
            secret: false,
        }
    }

    /// Marks the target as secret material.
    ///
    /// When enabled, the temporary file is restricted to owner-only
    /// permissions (0o600) on Unix **before** any content is written, so the
    /// secret bytes are never readable by other users. Windows initializes a
    /// protected current-user DACL for new targets and preserves existing DACLs.
    #[must_use]
    pub fn secret(mut self, secret: bool) -> Self {
        self.secret = secret;
        self
    }

    /// 💾 原子写入内容到文件
    ///
    /// # Arguments
    /// - `content` - 要写入的字节内容
    ///
    /// # Returns
    /// - `Ok(())` - 写入成功
    /// - `Err(CcrError)` - 写入失败
    ///
    /// # Process
    /// 1. 在目标目录创建临时文件
    /// 2. 写入内容到临时文件
    /// 3. 原子替换目标文件
    pub fn write(&self, content: &[u8]) -> Result<()> {
        // 📁 确保目标目录存在
        if let Some(parent) = self.target_path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                CcrError::IoError(std::io::Error::other(format!("创建目录失败: {}", e)))
            })?;
        }

        // 📄 在同一目录创建临时文件(确保在同一文件系统)
        let mut temp_file = if let Some(parent) = self.target_path.parent() {
            NamedTempFile::new_in(parent)
        } else {
            NamedTempFile::new()
        }
        .map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!("创建临时文件失败: {}", e)))
        })?;

        // 🔐 secret 模式：写入内容之前先收紧权限，避免敏感字节以宽权限落盘。
        // Existing owner-only modes are preserved so a read-only secret does
        // not become writable after replacement.
        #[cfg(unix)]
        if self.secret {
            use std::os::unix::fs::{MetadataExt, PermissionsExt};
            let mode = match fs::metadata(&self.target_path) {
                Ok(metadata) if metadata.mode() & 0o077 == 0 => metadata.mode() & 0o777,
                Ok(_) => 0o600,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => 0o600,
                Err(error) => {
                    return Err(CcrError::IoError(std::io::Error::other(format!(
                        "读取目标文件权限失败: {}",
                        error
                    ))));
                }
            };
            temp_file
                .as_file()
                .set_permissions(fs::Permissions::from_mode(mode))
                .map_err(|e| {
                    CcrError::IoError(std::io::Error::other(format!(
                        "设置临时文件权限失败: {}",
                        e
                    )))
                })?;
        }
        #[cfg(windows)]
        if self.secret {
            let descriptor = secret_windows_dacl(&self.target_path).map_err(|e| {
                CcrError::IoError(std::io::Error::other(format!(
                    "读取目标文件 ACL 失败: {}",
                    e
                )))
            })?;
            apply_windows_dacl(temp_file.path(), &descriptor).map_err(|e| {
                CcrError::IoError(std::io::Error::other(format!(
                    "设置临时文件 ACL 失败: {}",
                    e
                )))
            })?;
        }

        // ✍️ 通过临时文件句柄写入内容（避免二次打开）
        {
            use std::io::Write;
            temp_file.as_file_mut().write_all(content).map_err(|e| {
                CcrError::IoError(std::io::Error::other(format!("写入临时文件失败: {}", e)))
            })?;
        }

        // 💽 rename 前 fsync，确保数据先于替换操作落盘
        temp_file.as_file().sync_all().map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!("刷写临时文件失败: {}", e)))
        })?;

        self.persist_temp_file(temp_file)?;
        sync_parent_dir(&self.target_path).map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!("刷写父目录失败: {}", e)))
        })?;

        tracing::debug!("✅ 文件已原子写入: {:?}", self.target_path);
        Ok(())
    }

    /// 💾 原子写入字符串内容到文件
    ///
    /// # Arguments
    /// - `content` - 要写入的字符串内容
    ///
    /// # Returns
    /// - `Ok(())` - 写入成功
    /// - `Err(CcrError)` - 写入失败
    pub fn write_string(&self, content: &str) -> Result<()> {
        self.write(content.as_bytes())
    }

    fn persist_temp_file(&self, temp_file: NamedTempFile) -> Result<()> {
        #[cfg(windows)]
        {
            self.persist_temp_file_windows(temp_file)
        }

        #[cfg(not(windows))]
        {
            temp_file.persist(&self.target_path).map_err(|e| {
                CcrError::IoError(std::io::Error::other(format!(
                    "原子替换文件失败 (target: {}): {}",
                    self.target_path.display(),
                    e
                )))
            })?;
            Ok(())
        }
    }

    #[cfg(windows)]
    fn persist_temp_file_windows(&self, temp_file: NamedTempFile) -> Result<()> {
        let temp_path = temp_file.into_temp_path();

        replace_path_windows(temp_path.as_ref(), &self.target_path).map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!(
                "原子替换文件失败 (target: {}): {}",
                self.target_path.display(),
                e
            )))
        })
    }
}

impl AsyncAtomicWriter {
    /// 🏗️ 创建新的异步原子写入器
    pub fn new<P: AsRef<Path>>(target_path: P) -> Self {
        Self {
            target_path: target_path.as_ref().to_path_buf(),
            options: AsyncAtomicWriterOptions::default(),
        }
    }

    /// Applies an explicit security and metadata policy.
    #[must_use]
    pub fn options(mut self, options: AsyncAtomicWriterOptions) -> Self {
        self.options = options;
        self
    }

    /// Marks the target as secret material.
    #[must_use]
    pub fn secret(mut self, secret: bool) -> Self {
        self.options.secret = secret;
        self
    }

    /// Preserves an existing target's stricter mode or Windows DACL.
    #[must_use]
    pub fn preserve_mode(mut self, preserve_mode: bool) -> Self {
        self.options.preserve_mode = preserve_mode;
        self
    }

    /// 💾 异步原子写入内容到文件
    pub async fn write_async(&self, content: &[u8]) -> Result<()> {
        if let Some(parent) = self.target_path.parent() {
            async_fs::create_dir_all(parent).await.map_err(|e| {
                CcrError::IoError(std::io::Error::other(format!("创建目录失败: {}", e)))
            })?;
        }

        let temp_path = self.temp_path();

        #[cfg(windows)]
        let preserved_dacl = if self.options.secret {
            let target = self.target_path.clone();
            Some(
                tokio::task::spawn_blocking(move || secret_windows_dacl(&target))
                    .await
                    .map_err(|_| CcrError::IoError(std::io::Error::other("ACL task failed")))??,
            )
        } else if self.options.preserve_mode && self.target_path.exists() {
            Some(
                capture_windows_dacl_async(self.target_path.clone())
                    .await
                    .map_err(|e| {
                        CcrError::IoError(std::io::Error::other(format!(
                            "读取目标文件 ACL 失败: {}",
                            e
                        )))
                    })?,
            )
        } else {
            None
        };

        let mut open_options = async_fs::OpenOptions::new();
        open_options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;

            let existing_mode = if self.options.preserve_mode {
                match async_fs::metadata(&self.target_path).await {
                    Ok(metadata) => Some(metadata.mode() & 0o777),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
                    Err(error) => {
                        return Err(CcrError::IoError(std::io::Error::other(format!(
                            "读取目标文件权限失败: {}",
                            error
                        ))));
                    }
                }
            } else {
                None
            };
            let mode = if self.options.secret {
                existing_mode
                    .filter(|mode| mode & 0o077 == 0)
                    .unwrap_or(0o600)
            } else {
                existing_mode.unwrap_or(0o666)
            };
            open_options.mode(mode);
        }

        let mut temp_file = open_options.open(&temp_path).await.map_err(|e| {
            CcrError::IoError(std::io::Error::other(format!("创建临时文件失败: {}", e)))
        })?;

        #[cfg(windows)]
        if let Some(dacl) = preserved_dacl
            && let Err(error) = apply_windows_dacl_async(temp_path.clone(), dacl).await
        {
            drop(temp_file);
            let _ = async_fs::remove_file(&temp_path).await;
            return Err(CcrError::IoError(std::io::Error::other(format!(
                "设置临时文件 ACL 失败: {}",
                error
            ))));
        }

        if let Err(e) = temp_file.write_all(content).await {
            drop(temp_file);
            let _ = async_fs::remove_file(&temp_path).await;
            return Err(CcrError::IoError(std::io::Error::other(format!(
                "写入临时文件失败: {}",
                e
            ))));
        }

        if let Err(e) = temp_file.sync_all().await {
            drop(temp_file);
            let _ = async_fs::remove_file(&temp_path).await;
            return Err(CcrError::IoError(std::io::Error::other(format!(
                "刷写临时文件失败: {}",
                e
            ))));
        }
        drop(temp_file);

        if let Err(e) = persist_temp_path_async(temp_path.clone(), self.target_path.clone()).await {
            let _ = async_fs::remove_file(&temp_path).await;
            return Err(CcrError::IoError(std::io::Error::other(format!(
                "原子替换文件失败: {}",
                e
            ))));
        }

        if let Err(e) = sync_parent_dir_async(self.target_path.clone()).await {
            return Err(CcrError::IoError(std::io::Error::other(format!(
                "刷写父目录失败: {}",
                e
            ))));
        }

        tracing::debug!("✅ 文件已原子写入: {:?}", self.target_path);
        Ok(())
    }

    /// 💾 异步原子写入字符串内容到文件
    pub async fn write_string_async(&self, content: &str) -> Result<()> {
        self.write_async(content.as_bytes()).await
    }

    fn temp_path(&self) -> PathBuf {
        let parent = self.target_path.parent().unwrap_or_else(|| Path::new("."));
        let file_name = self
            .target_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("tmp");
        let temp_name = format!(".{}.tmp-{}", file_name, Uuid::new_v4());
        parent.join(temp_name)
    }
}

async fn persist_temp_path_async(temp_path: PathBuf, target_path: PathBuf) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        tokio::task::spawn_blocking(move || replace_path_windows(&temp_path, &target_path))
            .await
            .map_err(|e| std::io::Error::other(format!("原子替换任务失败: {}", e)))?
    }

    #[cfg(not(windows))]
    {
        async_fs::rename(temp_path, target_path).await
    }
}

#[cfg(unix)]
fn sync_parent_dir(target_path: &Path) -> std::io::Result<()> {
    let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
    fs::File::open(parent)?.sync_all()
}

#[cfg(windows)]
fn sync_parent_dir(_target_path: &Path) -> std::io::Result<()> {
    Ok(())
}

#[cfg(not(any(unix, windows)))]
fn sync_parent_dir(_target_path: &Path) -> std::io::Result<()> {
    tracing::warn!("parent-directory fsync is unsupported on this platform");
    Ok(())
}

#[cfg(unix)]
async fn sync_parent_dir_async(target_path: PathBuf) -> std::io::Result<()> {
    tokio::task::spawn_blocking(move || {
        let parent = target_path.parent().unwrap_or_else(|| Path::new("."));
        fs::File::open(parent)?.sync_all()
    })
    .await
    .map_err(|e| std::io::Error::other(format!("父目录刷写任务失败: {}", e)))?
}

#[cfg(windows)]
async fn sync_parent_dir_async(_target_path: PathBuf) -> std::io::Result<()> {
    // MoveFileExW uses MOVEFILE_WRITE_THROUGH above. Windows does not expose a
    // portable directory fsync handle through std, so the replacement call is
    // the durability boundary on this platform.
    Ok(())
}

#[cfg(not(any(unix, windows)))]
async fn sync_parent_dir_async(_target_path: PathBuf) -> std::io::Result<()> {
    tracing::warn!("parent-directory fsync is unsupported on this platform");
    Ok(())
}

#[cfg(windows)]
const DACL_SECURITY_INFORMATION: u32 = 0x0000_0004;

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
    fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    fn LocalFree(memory: *mut std::ffi::c_void) -> *mut std::ffi::c_void;
}
#[cfg(windows)]
#[link(name = "advapi32")]
unsafe extern "system" {
    fn OpenProcessToken(
        process: *mut std::ffi::c_void,
        access: u32,
        token: *mut *mut std::ffi::c_void,
    ) -> i32;
    fn GetTokenInformation(
        token: *mut std::ffi::c_void,
        class: u32,
        data: *mut std::ffi::c_void,
        length: u32,
        needed: *mut u32,
    ) -> i32;
    fn ConvertSidToStringSidW(sid: *const std::ffi::c_void, text: *mut *mut u16) -> i32;
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        text: *const u16,
        revision: u32,
        descriptor: *mut *mut std::ffi::c_void,
        length: *mut u32,
    ) -> i32;
}

/// Obtain the existing DACL, or a protected ACL granting only the token user access.
#[cfg(windows)]
fn secret_windows_dacl(path: &Path) -> std::io::Result<Vec<u8>> {
    match fs::metadata(path) {
        Ok(_) => return capture_windows_dacl(path),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
        Err(e) => return Err(e),
    }
    // SAFETY: All WinAPI pointers reference owned, aligned buffers. Handles and
    // LocalAlloc buffers are released on every path after acquisition.
    unsafe {
        let mut token = std::ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), 0x0008, &mut token) == 0 {
            return Err(std::io::Error::last_os_error());
        }
        let mut needed = 0;
        GetTokenInformation(token, 1, std::ptr::null_mut(), 0, &mut needed);
        let mut info = vec![0_usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
        let ok = GetTokenInformation(token, 1, info.as_mut_ptr().cast(), needed, &mut needed);
        let failure = std::io::Error::last_os_error();
        CloseHandle(token);
        if ok == 0 {
            return Err(failure);
        }
        let sid = *(info.as_ptr().cast::<*const std::ffi::c_void>());
        let mut sid_text = std::ptr::null_mut();
        if ConvertSidToStringSidW(sid, &mut sid_text) == 0 {
            return Err(std::io::Error::last_os_error());
        }
        let mut length = 0;
        while *sid_text.add(length) != 0 {
            length += 1;
        }
        let user = String::from_utf16_lossy(std::slice::from_raw_parts(sid_text, length));
        LocalFree(sid_text.cast());
        let sddl: Vec<u16> = format!("D:P(A;;FA;;;{user})")
            .encode_utf16()
            .chain(once(0))
            .collect();
        let mut descriptor = std::ptr::null_mut();
        let mut size = 0;
        if ConvertStringSecurityDescriptorToSecurityDescriptorW(
            sddl.as_ptr(),
            1,
            &mut descriptor,
            &mut size,
        ) == 0
        {
            return Err(std::io::Error::last_os_error());
        }
        let bytes = std::slice::from_raw_parts(descriptor.cast::<u8>(), size as usize).to_vec();
        LocalFree(descriptor);
        Ok(bytes)
    }
}

#[cfg(windows)]
#[link(name = "advapi32")]
unsafe extern "system" {
    fn GetSecurityDescriptorControl(
        descriptor: *const std::ffi::c_void,
        control: *mut u16,
        revision: *mut u32,
    ) -> i32;
    fn GetFileSecurityW(
        file_name: *const u16,
        requested_information: u32,
        security_descriptor: *mut std::ffi::c_void,
        length: u32,
        length_needed: *mut u32,
    ) -> i32;
    fn SetFileSecurityW(
        file_name: *const u16,
        security_information: u32,
        security_descriptor: *const std::ffi::c_void,
    ) -> i32;
}

#[cfg(windows)]
fn capture_windows_dacl(path: &Path) -> std::io::Result<Vec<u8>> {
    let path = path_to_wide(path);
    let mut needed = 0_u32;
    // SAFETY: The first call intentionally supplies a null buffer to obtain
    // the required self-relative security descriptor size.
    unsafe {
        GetFileSecurityW(
            path.as_ptr(),
            DACL_SECURITY_INFORMATION,
            std::ptr::null_mut(),
            0,
            &mut needed,
        );
    }
    if needed == 0 {
        return Err(std::io::Error::last_os_error());
    }

    let mut descriptor = vec![0_u8; needed as usize];
    // SAFETY: The buffer has the size requested by GetFileSecurityW and lives
    // for the duration of the call.
    let ok = unsafe {
        GetFileSecurityW(
            path.as_ptr(),
            DACL_SECURITY_INFORMATION,
            descriptor.as_mut_ptr().cast(),
            needed,
            &mut needed,
        )
    };
    if ok == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(descriptor)
    }
}

#[cfg(windows)]
fn apply_windows_dacl(path: &Path, descriptor: &[u8]) -> std::io::Result<()> {
    #[cfg(test)]
    {
        let mut failure = DACL_SETUP_FAILURE
            .lock()
            .map_err(|_| std::io::Error::other("ACL failure fixture lock poisoned"))?;
        if let Some(failure) = failure.as_mut()
            && path.parent() == Some(failure.directory.as_path())
        {
            // Observe the real temporary file at the permission boundary, not
            // a duplicate mock writer. A moved write-before-ACL is caught here.
            assert!(
                fs::read(path)?.is_empty(),
                "secret payload preceded ACL setup"
            );
            failure.empty_temporaries.push(path.to_path_buf());
            return Err(std::io::Error::from(std::io::ErrorKind::PermissionDenied));
        }
    }
    let path = path_to_wide(path);
    let mut control = 0_u16;
    let mut revision = 0_u32;
    // SAFETY: The descriptor comes from GetFileSecurityW or the SDDL converter.
    if unsafe {
        GetSecurityDescriptorControl(descriptor.as_ptr().cast(), &mut control, &mut revision)
    } == 0
    {
        return Err(std::io::Error::last_os_error());
    }
    let inheritance = if control & 0x1000 != 0 {
        0x8000_0000
    } else {
        0x2000_0000
    };
    // SAFETY: descriptor is the self-relative security descriptor returned by
    // GetFileSecurityW and remains alive for the call.
    let ok = unsafe {
        SetFileSecurityW(
            path.as_ptr(),
            DACL_SECURITY_INFORMATION | inheritance,
            descriptor.as_ptr().cast(),
        )
    };
    if ok == 0 {
        Err(std::io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(all(windows, test))]
struct DaclSetupFailure {
    directory: PathBuf,
    empty_temporaries: Vec<PathBuf>,
}

#[cfg(all(windows, test))]
static DACL_SETUP_FAILURE: std::sync::Mutex<Option<DaclSetupFailure>> = std::sync::Mutex::new(None);

#[cfg(windows)]
async fn capture_windows_dacl_async(path: PathBuf) -> std::io::Result<Vec<u8>> {
    tokio::task::spawn_blocking(move || capture_windows_dacl(&path))
        .await
        .map_err(|e| std::io::Error::other(format!("ACL 读取任务失败: {}", e)))?
}

#[cfg(windows)]
async fn apply_windows_dacl_async(path: PathBuf, descriptor: Vec<u8>) -> std::io::Result<()> {
    tokio::task::spawn_blocking(move || apply_windows_dacl(&path, &descriptor))
        .await
        .map_err(|e| std::io::Error::other(format!("ACL 设置任务失败: {}", e)))?
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::fs;
    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;
    use tempfile::tempdir;

    #[test]
    fn test_atomic_write() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test.txt");

        let writer = AtomicWriter::new(&target_path);
        writer.write(b"Hello, World!").unwrap();

        // 验证文件存在且内容正确
        assert!(target_path.exists());
        let content = fs::read_to_string(&target_path).unwrap();
        assert_eq!(content, "Hello, World!");
    }

    #[test]
    fn test_atomic_write_string() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test.txt");

        let writer = AtomicWriter::new(&target_path);
        writer.write_string("Hello, Rust!").unwrap();

        // 验证文件内容
        let content = fs::read_to_string(&target_path).unwrap();
        assert_eq!(content, "Hello, Rust!");
    }

    #[test]
    fn test_atomic_overwrite() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test.txt");

        // 第一次写入
        let writer = AtomicWriter::new(&target_path);
        writer.write_string("First content").unwrap();
        assert_eq!(fs::read_to_string(&target_path).unwrap(), "First content");

        // 第二次覆盖写入
        writer.write_string("Second content").unwrap();
        assert_eq!(fs::read_to_string(&target_path).unwrap(), "Second content");
    }

    #[test]
    fn test_atomic_overwrite_repeatedly() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test-repeat.txt");
        let writer = AtomicWriter::new(&target_path);

        for idx in 0..5 {
            writer.write_string(&format!("content-{idx}")).unwrap();
        }

        assert_eq!(fs::read_to_string(&target_path).unwrap(), "content-4");
    }

    #[test]
    fn test_atomic_write_creates_directory() {
        let temp_dir = tempdir().unwrap();
        let nested_path = temp_dir.path().join("nested").join("dir").join("test.txt");

        let writer = AtomicWriter::new(&nested_path);
        writer.write_string("Nested file").unwrap();

        // 验证目录和文件都被创建
        assert!(nested_path.exists());
        assert_eq!(fs::read_to_string(&nested_path).unwrap(), "Nested file");
    }

    #[tokio::test]
    async fn test_async_atomic_write() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test_async.txt");

        let writer = AsyncAtomicWriter::new(&target_path);
        writer.write_async(b"Hello, Async!").await.unwrap();

        assert!(target_path.exists());
        let content = fs::read_to_string(&target_path).unwrap();
        assert_eq!(content, "Hello, Async!");
    }

    #[tokio::test]
    async fn test_async_atomic_write_string() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test_async.txt");

        let writer = AsyncAtomicWriter::new(&target_path);
        writer
            .write_string_async("Hello, Async String!")
            .await
            .unwrap();

        let content = fs::read_to_string(&target_path).unwrap();
        assert_eq!(content, "Hello, Async String!");
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_async_secret_mode_ignores_process_umask() {
        unsafe extern "C" {
            fn umask(mask: u32) -> u32;
        }

        let temp_dir = tempdir().unwrap();
        for mask in [0o000, 0o022, 0o077] {
            let target_path = temp_dir.path().join(format!("secret-{mask:o}.json"));
            // SAFETY: This test is required to exercise the process umask and
            // restores it immediately. Run the documented target with one test
            // thread so unrelated file-creation tests cannot overlap.
            let previous = unsafe { umask(mask) };
            let result = AsyncAtomicWriter::new(&target_path)
                .secret(true)
                .preserve_mode(true)
                .write_string_async("secret")
                .await;
            // SAFETY: Restore the exact process umask returned above.
            unsafe { umask(previous) };

            result.unwrap();
            assert_eq!(
                fs::metadata(&target_path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_async_secret_preserves_only_stricter_existing_mode() {
        let temp_dir = tempdir().unwrap();
        for (existing_mode, expected_mode) in [(0o400, 0o400), (0o600, 0o600), (0o644, 0o600)] {
            let target_path = temp_dir
                .path()
                .join(format!("secret-{existing_mode:o}.json"));
            fs::write(&target_path, "old").unwrap();
            fs::set_permissions(&target_path, fs::Permissions::from_mode(existing_mode)).unwrap();

            AsyncAtomicWriter::new(&target_path)
                .secret(true)
                .preserve_mode(true)
                .write_string_async("new")
                .await
                .unwrap();

            assert_eq!(fs::read_to_string(&target_path).unwrap(), "new");
            assert_eq!(
                fs::metadata(&target_path).unwrap().permissions().mode() & 0o777,
                expected_mode
            );
        }
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn test_async_secret_preserves_windows_dacl() {
        fn dacl_bytes(descriptor: &[u8]) -> &[u8] {
            let offset = u32::from_le_bytes(descriptor[16..20].try_into().unwrap()) as usize;
            let length =
                u16::from_le_bytes(descriptor[offset + 2..offset + 4].try_into().unwrap()) as usize;
            &descriptor[offset..offset + length]
        }

        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("secret.json");
        fs::write(&target_path, "old").unwrap();
        let before = capture_windows_dacl(&target_path).unwrap();

        AsyncAtomicWriter::new(&target_path)
            .secret(true)
            .preserve_mode(true)
            .write_string_async("new")
            .await
            .unwrap();

        let after = capture_windows_dacl(&target_path).unwrap();
        assert_eq!(fs::read_to_string(&target_path).unwrap(), "new");
        assert_eq!(dacl_bytes(&after), dacl_bytes(&before));
        assert_eq!(after[3] & 0x10, before[3] & 0x10);
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn grok_auth_new_secret_windows_dacl_is_private_before_payload() {
        let directory = tempdir().unwrap();
        let target = directory.path().join("new-secret.json");
        let descriptor = secret_windows_dacl(&target).unwrap();
        let temporary = NamedTempFile::new_in(directory.path()).unwrap();
        apply_windows_dacl(temporary.path(), &descriptor).unwrap();
        assert_eq!(temporary.as_file().metadata().unwrap().len(), 0);
        let actual = capture_windows_dacl(temporary.path()).unwrap();
        let control = u16::from_le_bytes(actual[2..4].try_into().unwrap());
        assert_ne!(control & 0x1000, 0, "new secret DACL must be protected");
        let offset = u32::from_le_bytes(actual[16..20].try_into().unwrap()) as usize;
        let ace_count = u16::from_le_bytes(actual[offset + 4..offset + 6].try_into().unwrap());
        assert_eq!(ace_count, 1, "only the current user receives an ACE");
        let expected_offset = u32::from_le_bytes(descriptor[16..20].try_into().unwrap()) as usize;
        let ace_size =
            u16::from_le_bytes(actual[offset + 10..offset + 12].try_into().unwrap()) as usize;
        // An ACCESS_ALLOWED_ACE has an 8-byte header/mask followed by the SID.
        // The descriptor helper obtains this SID from the current process token.
        assert_eq!(
            &actual[offset + 16..offset + 8 + ace_size],
            &descriptor[expected_offset + 16..expected_offset + 8 + ace_size],
        );
        AtomicWriter::new(&target)
            .secret(true)
            .write(b"secret")
            .unwrap();
        let asynchronous = directory.path().join("async-secret.json");
        AsyncAtomicWriter::new(&asynchronous)
            .secret(true)
            .write_async(b"secret")
            .await
            .unwrap();
        for path in [&target, &asynchronous] {
            let dacl = capture_windows_dacl(path).unwrap();
            let control = u16::from_le_bytes(dacl[2..4].try_into().unwrap());
            assert_ne!(control & 0x1000, 0);
        }
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn grok_auth_windows_acl_failure_precedes_payload_and_cleans_temporary_files() {
        let directory = tempdir().unwrap();
        let existing = directory.path().join("existing.json");
        let new_target = directory.path().join("new.json");
        fs::write(&existing, b"original bytes").unwrap();
        *DACL_SETUP_FAILURE.lock().unwrap() = Some(DaclSetupFailure {
            directory: directory.path().to_path_buf(),
            empty_temporaries: Vec::new(),
        });

        let sync_result = AtomicWriter::new(&existing)
            .secret(true)
            .write(b"SECRET_PAYLOAD");
        let async_result = AsyncAtomicWriter::new(&new_target)
            .secret(true)
            .write_async(b"SECRET_PAYLOAD")
            .await;
        // Clear the failpoint before assertions so assertion failure cannot
        // affect another fixture. The directory match also isolates concurrency.
        let failure = DACL_SETUP_FAILURE.lock().unwrap().take().unwrap();
        assert!(sync_result.is_err());
        assert!(async_result.is_err());
        assert_eq!(failure.empty_temporaries.len(), 2);
        for temporary in failure.empty_temporaries {
            assert!(
                !temporary.exists(),
                "failed empty temporary was not cleaned up"
            );
        }
        assert_eq!(fs::read(&existing).unwrap(), b"original bytes");
        assert!(!new_target.exists());
        assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 1);
    }

    #[cfg(windows)]
    #[test]
    fn test_windows_atomic_replace_failure_keeps_original_file() {
        let temp_dir = tempdir().unwrap();
        let target_path = temp_dir.path().join("test.txt");
        fs::write(&target_path, "original").unwrap();

        let mut attempts = 0usize;
        let error = retry_windows_atomic_replace(&target_path, || {
            attempts += 1;
            Err(std::io::Error::from_raw_os_error(32))
        })
        .expect_err("retry exhaustion should fail");

        assert!(is_windows_atomic_retry_error(&error));
        assert_eq!(attempts, ATOMIC_WRITE_RETRY_LIMIT);
        assert_eq!(fs::read_to_string(&target_path).unwrap(), "original");
    }
}
