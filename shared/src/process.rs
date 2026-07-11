//! Helpers for spawning child processes without flashing a console on Windows.

use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Applies Windows-specific flags so child processes do not open a console window.
pub fn hide_console(cmd: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
}

/// Creates a [`Command`] with console hiding applied on Windows.
pub fn hidden_command(program: &str) -> Command {
    let mut cmd = Command::new(program);
    hide_console(&mut cmd);
    cmd
}
