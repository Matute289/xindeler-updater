use crate::{Result, windows};
use std::{
    ffi::{OsStr, OsString},
    os::windows::ffi::OsStrExt,
    path::Path,
    ptr,
};
use windows_sys::Win32::{
    System::{Console::GetConsoleWindow, Threading::GetCurrentProcessId},
    UI::{
        Shell::ShellExecuteW,
        WindowsAndMessaging::{GetWindowThreadProcessId, SW_HIDE, SW_SHOW, ShowWindow},
    },
};

/// Runs a downloaded launcher installer (`.exe` or `.msi`) elevated (UAC prompt) and
/// exits the current process - Windows can't replace a running exe's own file the way
/// Unix can, so this hands off to the installer instead. See `launcher_update.rs`,
/// which downloads and checksum-verifies the installer before calling this.
pub(crate) fn run_installer_elevated(install_file_path: &Path) -> Result<()> {
    tracing::debug!("Starting installer...");
    let result = match install_file_path.extension().and_then(|f| f.to_str()) {
        Some("exe") => windows::execute_as_admin(install_file_path, ""),
        _ => windows::execute_as_admin(
            "msiexec",
            &format!(
                "/passive /i \"{}\" AUTOSTART=1",
                install_file_path.display(),
            ),
        ),
    };

    if result <= 32 {
        tracing::error!(
            "Failed to launch xindeler-updater installer! {}",
            std::io::Error::last_os_error()
        );
    }

    Ok(())
}

pub fn execute_as_admin<T, T2>(program: T, args: T2) -> i32
where
    T: Into<OsString>,
    T2: Into<OsString>,
{
    let operation: Vec<u16> = OsStr::new("runas\0").encode_wide().collect();
    let mut program = program.into();
    program.push("\0");
    let mut arguments = args.into();
    arguments.push("\0");

    let bin = program.encode_wide().collect::<Vec<u16>>();
    let arguments: Vec<u16> = arguments.encode_wide().collect();

    unsafe {
        ShellExecuteW(
            ptr::null_mut(),
            operation.as_ptr(),
            bin.as_ptr(),
            arguments.as_ptr(),
            ptr::null(),
            SW_SHOW,
        ) as i32
    }
}

/// Hides the console incase the process hasn't been started from one.
pub fn hide_non_inherited_console() {
    if !started_from_console() {
        let window = unsafe { GetConsoleWindow() };
        // https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-showwindow
        if !window.is_null() {
            unsafe {
                ShowWindow(window, SW_HIDE);
            }
        }
    }
}

/// Determines whether the process has been started from console.
fn started_from_console() -> bool {
    unsafe {
        let console_wnd = GetConsoleWindow();
        let process_id = GetCurrentProcessId();

        let mut parent_id = 0;
        GetWindowThreadProcessId(console_wnd, &mut parent_id);

        process_id != parent_id
    }
}
