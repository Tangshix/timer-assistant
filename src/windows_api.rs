#[cfg(windows)]
pub mod windows {
    use windows::{
        core::w,
        Win32::Foundation::*,
        Win32::System::Shutdown::*,
        Win32::UI::WindowsAndMessaging::*,
        Win32::Security::*,
        Win32::System::Threading::*,
    };
    use windows::core::PCWSTR;

    /// 启用关机特权
    pub fn enable_shutdown_privilege() -> Result<(), String> {
        unsafe {
            let mut token_handle = HANDLE::default();
            
            // 打开当前进程令牌
            OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                &mut token_handle,
            ).map_err(|e| format!("无法打开进程令牌: {:?}", e))?;

            // 查找 SE_SHUTDOWN_NAME 特权的 LUID
            let mut luid = LUID::default();
            LookupPrivilegeValueW(
                None,
                w!("SeShutdownPrivilege"),
                &mut luid,
            ).map_err(|e| format!("无法查找特权值: {:?}", e))?;

            // 设置特权
            let tp = TOKEN_PRIVILEGES {
                PrivilegeCount: 1,
                Privileges: [LUID_AND_ATTRIBUTES {
                    Luid: luid,
                    Attributes: SE_PRIVILEGE_ENABLED,
                }],
            };

            // 调整令牌特权
            AdjustTokenPrivileges(
                token_handle,
                false,
                Some(&tp),
                std::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
                None,
                None,
            ).map_err(|e| format!("无法调整令牌特权: {:?}", e))?;

            Ok(())
        }
    }

    /// 关机
    pub fn shutdown() -> Result<(), String> {
        enable_shutdown_privilege()?;
        
        unsafe {
            ExitWindowsEx(EWX_POWEROFF | EWX_FORCE, SHUTDOWN_REASON(0))
                .map_err(|e| format!("关机失败: {:?}", e))?;
        }
        
        Ok(())
    }

    /// 重启
    pub fn reboot() -> Result<(), String> {
        enable_shutdown_privilege()?;
        
        unsafe {
            ExitWindowsEx(EWX_REBOOT | EWX_FORCE, SHUTDOWN_REASON(0))
                .map_err(|e| format!("重启失败: {:?}", e))?;
        }
        
        Ok(())
    }

    /// 锁屏
    pub fn lock_screen() -> Result<(), String> {
        unsafe {
            LockWorkStation()
                .map_err(|e| format!("锁屏失败: {:?}", e))?;
        }
        
        Ok(())
    }

    /// 弹窗提醒
    pub fn show_popup(title: &str, message: &str) {
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;
        
        println!("准备显示弹窗: {} - {}", title, message);
        
        let title_wide: Vec<u16> = OsStr::new(title)
            .encode_wide()
            .chain(Some(0))
            .collect();
        
        let message_wide: Vec<u16> = OsStr::new(message)
            .encode_wide()
            .chain(Some(0))
            .collect();
        
        unsafe {
            let result = MessageBoxW(
                HWND::default(),
                PCWSTR(message_wide.as_ptr()),
                PCWSTR(title_wide.as_ptr()),
                MB_OK | MB_ICONINFORMATION | MB_SYSTEMMODAL,
            );
            println!("弹窗关闭，返回码: {:?}", result);
        }
    }
}

#[cfg(target_os = "macos")]
pub mod macos {
    use std::process::Command;

    /// 关机 (macOS)
    pub fn shutdown() -> Result<(), String> {
        println!("执行 macOS 关机");
        Command::new("osascript")
            .args(&["-e", "tell application \"System Events\" to shut down"])
            .output()
            .map_err(|e| format!("关机失败: {}", e))?;
        Ok(())
    }

    /// 重启 (macOS)
    pub fn reboot() -> Result<(), String> {
        println!("执行 macOS 重启");
        Command::new("osascript")
            .args(&["-e", "tell application \"System Events\" to restart"])
            .output()
            .map_err(|e| format!("重启失败: {}", e))?;
        Ok(())
    }

    /// 锁屏 (macOS)
    pub fn lock_screen() -> Result<(), String> {
        println!("执行 macOS 锁屏");
        // macOS Monterey 及更高版本使用以下命令
        Command::new("pmset")
            .args(&["displaysleepnow"])
            .output()
            .or_else(|_| {
                // 降级方案：使用 AppleScript
                Command::new("osascript")
                    .args(&["-e", "tell application \"System Events\" to keystroke \"q\" using {control down, command down}"])
                    .output()
            })
            .map_err(|e| format!("锁屏失败: {}", e))?;
        Ok(())
    }

    /// 弹窗提醒 (macOS)
    pub fn show_popup(title: &str, message: &str) {
        println!("准备显示 macOS 弹窗: {} - {}", title, message);
        
        let script = format!(
            "display dialog \"{}\" with title \"{}\" buttons {{\"确定\"}} default button \"确定\"",
            message.replace("\"", "\\\""),
            title.replace("\"", "\\\"")
        );
        
        match Command::new("osascript")
            .args(&["-e", &script])
            .output() {
            Ok(_) => {},
            Err(e) => {
                eprintln!("弹窗失败: {}", e);
            }
        }
    }
}

#[cfg(not(windows))]
#[cfg(not(target_os = "macos"))]
pub mod non_windows {
    pub fn shutdown() -> Result<(), String> {
        println!("[模拟] 关机");
        Ok(())
    }

    pub fn reboot() -> Result<(), String> {
        println!("[模拟] 重启");
        Ok(())
    }

    pub fn lock_screen() -> Result<(), String> {
        println!("[模拟] 锁屏");
        Ok(())
    }

    pub fn show_popup(title: &str, message: &str) {
        println!("[模拟] 弹窗: {} - {}", title, message);
    }
}
