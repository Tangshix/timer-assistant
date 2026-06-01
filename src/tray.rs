use tray_icon::{
    menu::{Menu, MenuItem, PredefinedMenuItem, MenuEvent},
    TrayIcon, TrayIconBuilder, Icon,
};

/// 创建系统托盘图标（Windows/Linux），返回 TrayIcon
/// 调用方必须持有返回值，否则图标会消失
pub fn create_tray_icon<F: Fn() + Send + 'static>(show_window: F) -> TrayIcon {
    let menu = Menu::new();

    menu.append(&MenuItem::with_id("countdown", "暂无任务", false, None)).unwrap();
    menu.append(&PredefinedMenuItem::separator()).unwrap();
    menu.append(&MenuItem::with_id("show", "显示主窗口", true, None)).unwrap();
    menu.append(&PredefinedMenuItem::separator()).unwrap();
    menu.append(&MenuItem::with_id("exit", "退出", true, None)).unwrap();

    std::thread::spawn(move || {
        let receiver = MenuEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                match event.id.0.as_str() {
                    "show" => show_window(),
                    "exit" => std::process::exit(0),
                    _ => {}
                }
            }
        }
    });

    TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("电脑定时助手")
        .with_icon(create_clock_icon())
        .build()
        .unwrap()
}

fn create_clock_icon() -> Icon {
    let size: usize = 32;
    let mut data = vec![0u8; size * size * 4];
    let center = size as f32 / 2.0;
    let radius = center - 2.0;

    for y in 0..size {
        for x in 0..size {
            let idx = (y * size + x) * 4;
            let cx = x as f32 - center;
            let cy = y as f32 - center;
            let dist = (cx * cx + cy * cy).sqrt();

            if dist <= radius {
                let g = dist / radius;
                data[idx]     = (41.0 + g * 30.0) as u8;
                data[idx + 1] = (121.0 + g * 40.0) as u8;
                data[idx + 2] = (230.0 - g * 50.0) as u8;
                data[idx + 3] = 255;

                let angle = cy.atan2(cx).to_degrees();
                let norm = if angle < 0.0 { angle + 360.0 } else { angle };
                if dist > radius - 4.0 && (norm % 30.0).abs() < 3.0 {
                    data[idx] = 255; data[idx+1] = 255; data[idx+2] = 255;
                }

                let ha = 150.0_f32.to_radians();
                if ((cx - ha.cos() * radius * 0.25).powi(2) + (cy - ha.sin() * radius * 0.25).powi(2)).sqrt() < 2.0 && dist < radius * 0.6 {
                    data[idx] = 255; data[idx+1] = 255; data[idx+2] = 255;
                }

                let ma = 60.0_f32.to_radians();
                if ((cx - ma.cos() * radius * 0.375).powi(2) + (cy - ma.sin() * radius * 0.375).powi(2)).sqrt() < 1.5 && dist < radius * 0.8 {
                    data[idx] = 255; data[idx+1] = 255; data[idx+2] = 255;
                }

                if dist < 2.0 {
                    data[idx] = 255; data[idx+1] = 255; data[idx+2] = 255;
                }
            } else if dist <= radius + 2.0 {
                data[idx] = 255; data[idx+1] = 255; data[idx+2] = 255; data[idx+3] = 255;
            } else {
                data[idx + 3] = 0;
            }
        }
    }

    Icon::from_rgba(data, 32, 32).unwrap()
}
