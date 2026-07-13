//! Helpers for spawning child processes without flashing a console on Windows.

use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Applies Windows-specific flags so child processes do not open a console window.
pub fn hide_console(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    // 非 Windows 平台不需要额外进程标志，但保留统一调用入口。
    #[cfg(not(windows))]
    let _ = cmd;
}

/// Creates a [`Command`] with console hiding applied on Windows.
pub fn hidden_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    hide_console(&mut cmd);
    cmd
}
