use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem, MenuEvent},
    TrayIconBuilder, Icon,
};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOW};

/// 托盘菜单项 ID
pub const MENU_SHOW_WINDOW: &str = "show_window";
pub const MENU_EXIT: &str = "exit";

/// 创建系统托盘图标
pub fn create_tray_icon(
    visible: Arc<AtomicBool>,
) -> (tray_icon::TrayIcon, mpsc::Receiver<()>) {
    let (_tx, rx) = mpsc::channel();
    
    // 创建菜单（使用 ID 构造函数）
    let menu = Menu::new();
    
    menu.append(&MenuItem::with_id(
        MENU_SHOW_WINDOW,
        "显示主窗口",
        true,
        None,
    )).unwrap();
    
    menu.append(&PredefinedMenuItem::separator()).unwrap();
    
    menu.append(&MenuItem::with_id(
        MENU_EXIT,
        "退出",
        true,
        None,
    )).unwrap();
    
    // 监听菜单事件并转发
    let visible_clone = visible.clone();
    std::thread::spawn(move || {
        let menu_channel = MenuEvent::receiver();
        loop {
            if let Ok(event) = menu_channel.recv() {
                // 比较字符串 ID
                if event.id.0 == MENU_SHOW_WINDOW {
                    visible_clone.store(true, Ordering::SeqCst);
                    println!("通过菜单显示窗口");
                    
                    // 直接使用 Windows API 显示窗口
                    #[cfg(windows)]
                    unsafe {
                        use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, SetForegroundWindow};
                        use windows::core::w;
                        
                        // 尝试多种方式查找窗口
                        let mut hwnd = FindWindowW(None, w!("电脑定时助手"));
                        
                        // 如果没找到，尝试使用窗口类名
                        if hwnd.is_err() {
                            hwnd = FindWindowW(w!("egui_window"), None);
                        }
                        
                        if let Ok(hwnd) = hwnd {
                            let _ = ShowWindow(hwnd, SW_SHOW);
                            let _ = SetForegroundWindow(hwnd);
                            println!("已通过 Windows API 显示窗口");
                        } else {
                            println!("未找到窗口句柄，尝试所有方法失败");
                        }
                    }
                } else if event.id.0 == MENU_EXIT {
                    println!("退出程序");
                    std::process::exit(0);
                }
            }
        }
    });
    
    // 创建图标（使用简单的 RGBA 图标 - 一个蓝色的时钟图标）
    let icon_data = create_clock_icon();
    let icon = Icon::from_rgba(icon_data, 32, 32).unwrap();
    
    // 创建托盘图标
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("电脑定时助手")
        .with_icon(icon)
        .build()
        .unwrap();
    
    (tray_icon, rx)
}

/// 创建时钟图标 - 更美观的设计
fn create_clock_icon() -> Vec<u8> {
    let size = 32;
    let mut data = vec![0u8; size * size * 4];
    
    let center = size as f32 / 2.0;
    let radius = size as f32 / 2.0 - 2.0;
    
    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            
            let cx = x as f32 - center;
            let cy = y as f32 - center;
            let distance = (cx * cx + cy * cy).sqrt();
            
            if distance <= radius {
                // 在圆内 - 创建渐变背景
                let gradient = (distance / radius) as f32;
                
                // 蓝色渐变背景（从中心到边缘）
                let r = (41.0 + gradient * 30.0) as u8;  // 65-71
                let g = (121.0 + gradient * 40.0) as u8; // 161-161
                let b = (230.0 - gradient * 50.0) as u8; // 180-230
                
                data[idx] = r;
                data[idx + 1] = g;
                data[idx + 2] = b;
                data[idx + 3] = 255;
                
                // 绘制时钟刻度
                let angle = (cy).atan2(cx);
                let degrees = angle.to_degrees();
                let normalized_angle = if degrees < 0.0 { degrees + 360.0 } else { degrees };
                
                // 每 30 度一个刻度（12个小时刻度）
                if distance > radius - 4.0 && (normalized_angle % 30.0).abs() < 3.0 {
                    data[idx] = 255;
                    data[idx + 1] = 255;
                    data[idx + 2] = 255;
                }
                
                // 绘制时针（指向 10 点位置）
                let hour_angle = (150.0_f32).to_radians(); // 10点钟方向
                let hour_x = (hour_angle.cos() * radius * 0.5);
                let hour_y = (hour_angle.sin() * radius * 0.5);
                
                // 简单的线段绘制
                let dist_to_hour = ((cx - hour_x * 0.5).powi(2) + (cy - hour_y * 0.5).powi(2)).sqrt();
                if dist_to_hour < 2.0 && distance < radius * 0.6 {
                    data[idx] = 255;
                    data[idx + 1] = 255;
                    data[idx + 2] = 255;
                    data[idx + 3] = 255;
                }
                
                // 绘制分针（指向 2 点位置）
                let minute_angle = (60.0_f32).to_radians(); // 2点钟方向
                let minute_x = (minute_angle.cos() * radius * 0.75);
                let minute_y = (minute_angle.sin() * radius * 0.75);
                
                let dist_to_minute = ((cx - minute_x * 0.5).powi(2) + (cy - minute_y * 0.5).powi(2)).sqrt();
                if dist_to_minute < 1.5 && distance < radius * 0.8 {
                    data[idx] = 255;
                    data[idx + 1] = 255;
                    data[idx + 2] = 255;
                    data[idx + 3] = 230;
                }
                
                // 中心点
                if distance < 2.0 {
                    data[idx] = 255;
                    data[idx + 1] = 255;
                    data[idx + 2] = 255;
                }
            } else if distance <= radius + 2.0 {
                // 圆边框 - 白色边框
                data[idx] = 255;
                data[idx + 1] = 255;
                data[idx + 2] = 255;
                data[idx + 3] = 255;
            } else {
                // 圆外，透明
                data[idx + 3] = 0;
            }
        }
    }
    
    data
}

/// 托盘点击事件处理（当前未使用，菜单事件在 create_tray_icon 中处理）
#[allow(dead_code)]
pub fn handle_tray_event(
    visible: &Arc<AtomicBool>,
) {
    visible.store(true, Ordering::SeqCst);
    println!("托盘图标被点击，显示窗口");
}
