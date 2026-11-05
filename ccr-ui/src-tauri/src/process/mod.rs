//! 进程辅助模块 — 统一封装外部命令构造，并在 Windows release 下隐藏子进程控制台窗口。

use std::ffi::OsStr;

pub(crate) mod gateway;

pub use ccr_core::core::process_gateway::read_bounded_line;
pub use gateway::{ManagedChild, ProcessDescriptor, ProcessGateway};

#[cfg(all(target_os = "windows", not(debug_assertions)))]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[cfg(all(target_os = "windows", not(debug_assertions)))]
fn apply_no_window_to_std(cmd: &mut std::process::Command) {
    use std::os::windows::process::CommandExt;

    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(all(target_os = "windows", not(debug_assertions))))]
#[allow(dead_code)]
fn apply_no_window_to_std(_cmd: &mut std::process::Command) {}

/// 创建并配置标准库进程命令。
#[allow(dead_code)]
pub fn std_command(program: impl AsRef<OsStr>) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    configure_std_command(&mut cmd);
    cmd
}

/// 为现有标准库进程命令应用统一配置。
#[allow(dead_code)]
pub fn configure_std_command(cmd: &mut std::process::Command) -> &mut std::process::Command {
    apply_no_window_to_std(cmd);
    cmd
}
