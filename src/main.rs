#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

mod tray;
mod windows_api;
mod scheduler;

use std::sync::{Arc, Mutex};
use chrono::{Local, Timelike};
use scheduler::{TaskScheduler, ScheduledTask, TaskType};
#[cfg(target_os = "macos")]
use cocoa::base::{nil, id};

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建应用状态
    let scheduler = Arc::new(Mutex::new(TaskScheduler::new()));

    // 先创建 Slint UI
    let ui = TimerApp::new()?;

    // 拦截窗口关闭事件：隐藏整个应用到托盘，而非退出
    #[cfg(target_os = "macos")]
    ui.window().on_close_requested(|| unsafe {
        let app: id = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![app, hide:nil];
        slint::CloseRequestResponse::KeepWindowShown
    });

    #[cfg(not(target_os = "macos"))]
    ui.window().on_close_requested({
        let ui_weak = ui.as_weak();
        move || {
            println!("Windows: 点击关闭按钮，准备隐藏到托盘");
            if let Some(ui_ref) = ui_weak.upgrade() {
                let _ = ui_ref.window().hide();
                println!("Windows: 窗口已隐藏");
            }
            slint::CloseRequestResponse::KeepWindowShown
        }
    });

    // 创建 macOS 菜单栏托盘图标
    #[cfg(target_os = "macos")]
    let status_item = {
        use cocoa::appkit::{NSStatusBar, NSStatusItem, NSMenu, NSMenuItem, NSButton, NSImage};
        use cocoa::base::selector;
        use cocoa::foundation::NSString;
        use objc::runtime::{Object, Sel};
        use objc::declare::ClassDecl;

        unsafe {
            let status_item = NSStatusBar::systemStatusBar(nil).statusItemWithLength_(-1.0);
            let button = status_item.button();
            
            // 加载图标图片（支持开发模式和打包后两种路径）
            let icon_path = {
                // 先尝试 App Bundle 的资源路径
                let bundle_path = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|pp| pp.join("../Resources/app.png")))
                    .unwrap_or_default();
                
                if bundle_path.exists() {
                    bundle_path
                } else {
                    // 开发模式：直接使用项目目录的图标
                    let dev_path = std::env::current_dir()
                        .unwrap_or_default()
                        .join("icons/app.png");
                    
                    if dev_path.exists() {
                        dev_path
                    } else {
                        // 两个路径都找不到，返回空路径
                        std::path::PathBuf::new()
                    }
                }
            };
            
            // 尝试加载图片，如果失败则使用文字图标
            let ns_image = if !icon_path.as_os_str().is_empty() && icon_path.exists() {
                let icon_path_str = NSString::alloc(nil).init_str(&icon_path.to_string_lossy());
                let img = NSImage::alloc(nil).initWithContentsOfFile_(icon_path_str);
                let _: () = msg_send![icon_path_str, release];
                if !img.is_null() {
                    img
                } else {
                    // 图片加载失败，使用文字图标
                    let title = NSString::alloc(nil).init_str("\u{23F0}");
                    button.setTitle_(title);
                    let _: () = msg_send![title, release];
                    nil
                }
            } else {
                // 图片文件不存在，使用文字图标
                println!("警告: 图标文件不存在，使用默认图标");
                let title = NSString::alloc(nil).init_str("\u{23F0}");
                button.setTitle_(title);
                let _: () = msg_send![title, release];
                nil
            };
            
            // 如果图片加载成功，设置图标
            if !ns_image.is_null() {
                // 设置图标大小
                let _: () = msg_send![ns_image, setSize:cocoa::foundation::NSSize::new(18.0, 18.0)];
                button.setImage_(ns_image);
                
                // 不使用模板模式，保留原始颜色
                // let _: () = msg_send![ns_image, setTemplate: false];
            }

            let menu = NSMenu::new(nil);
            let show_item = NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(
                    NSString::alloc(nil).init_str("显示主窗口"),
                    selector("showWindow:"),
                    NSString::alloc(nil).init_str(""),
                );
            menu.addItem_(show_item);
            menu.addItem_(NSMenuItem::separatorItem(nil));
            let quit_item = NSMenuItem::alloc(nil)
                .initWithTitle_action_keyEquivalent_(
                    NSString::alloc(nil).init_str("退出"),
                    selector("terminate:"),
                    NSString::alloc(nil).init_str("q"),
                );
            menu.addItem_(quit_item);
            status_item.setMenu_(menu);

            // 菜单项"显示主窗口"：激活应用（自动恢复所有窗口）
            let mut cls = ClassDecl::new("TrayClickHandler", class!(NSObject)).unwrap();
            extern "C" fn show_window(_this: &Object, _sel: Sel, _sender: id) {
                unsafe {
                    let app: id = msg_send![class!(NSApplication), sharedApplication];
                    let _: () = msg_send![app, activateIgnoringOtherApps: true];
                }
            }
            cls.add_method(
                sel!(showWindow:),
                show_window as extern "C" fn(&Object, Sel, id),
            );
            let handler_cls = cls.register();
            let handler: id = msg_send![handler_cls, new];
            let _: () = msg_send![show_item, setTarget:handler];
            let _: () = msg_send![show_item, setAction:sel!(showWindow:)];

            status_item
        }
    };

    // 创建 Windows/Linux 系统托盘图标（局部变量，需保持存活直到程序退出）
    #[cfg(not(target_os = "macos"))]
    let _tray_icon = {
        let ui_weak_tray = ui.as_weak();
        tray::create_tray_icon(move || {
            if let Some(ui_ref) = ui_weak_tray.upgrade() {
                ui_ref.window().show().ok();
            }
        })
    };

    // 定时刷新任务列表 + 更新窗口标题和托盘 tooltip
    let scheduler_refresh = scheduler.clone();
    let ui_weak_refresh = ui.as_weak();
    let refresh_timer = slint::Timer::default();
    refresh_timer.start(slint::TimerMode::Repeated, std::time::Duration::from_secs(1), move || {
        if let Some(ui_ref) = ui_weak_refresh.upgrade() {
            update_task_list(&ui_ref, &scheduler_refresh);
            #[allow(unused_variables)]
            let (title, tray_text, tooltip) = if let Ok(sched) = scheduler_refresh.lock() {
                if let Some((task_type, remaining)) = sched.get_next_countdown() {
                    let name = match task_type {
                        TaskType::Shutdown => "关机",
                        TaskType::Reboot => "重启",
                        TaskType::LockScreen => "锁屏",
                        TaskType::Popup => "弹窗",
                    };
                    let h = remaining / 3600;
                    let m = (remaining % 3600) / 60;
                    let s = remaining % 60;
                    let time_str = if h > 0 {
                        format!("{:02}:{:02}:{:02}", h, m, s)
                    } else {
                        format!("{:02}:{:02}", m, s)
                    };
                    let title = format!("电脑定时助手 - {} {}", name, time_str);
                    let tray = format!(" {}", time_str);  // 去掉闹钟emoji，只显示倒计时
                    let tip = format!("{} {} 剩余", name, time_str);
                    (title, tray, tip)
                } else {
                    ("电脑定时助手".to_string(), "".to_string(), "暂无任务".to_string())
                }
            } else {
                ("电脑定时助手".to_string(), "".to_string(), "暂无任务".to_string())
            };
            ui_ref.set_window_title(title.into());
            // 更新托盘图标：倒计时文字 + tooltip
            #[cfg(target_os = "macos")]
            unsafe {
                use cocoa::appkit::NSStatusItem;
                use cocoa::base::nil;
                use cocoa::foundation::NSString;
                let button = status_item.button();
                
                // 更新 tooltip
                let tip = NSString::alloc(nil).init_str(&tooltip);
                let _: () = msg_send![button, setToolTip:tip];
                let _: () = msg_send![tip, release];
                
                // 如果有倒计时，显示倒计时文字；否则只显示蓝色钟表图标
                if !tray_text.is_empty() {
                    // 有倒计时：显示蓝色钟表 + 倒计时文字
                    let title = NSString::alloc(nil).init_str(&tray_text);
                    let _: () = msg_send![button, setTitle:title];
                    let _: () = msg_send![title, release];
                } else {
                    // 没有倒计时：清空文字，只显示蓝色钟表图标
                    let title = NSString::alloc(nil).init_str("");
                    let _: () = msg_send![button, setTitle:title];
                    let _: () = msg_send![title, release];
                }
            }
        }
    });
    
    // drop(countdown_tx); // 关闭原始发送端，只保留克隆的
    
    // 设置初始值
    ui.set_selected_hour(22);
    ui.set_selected_minute(0);
    ui.set_selected_task_type("弹窗提醒".into());
    ui.set_popup_message("这是您的定时提醒消息！".into());
    
    // 添加任务回调
    let scheduler_add = scheduler.clone();
    let ui_weak_add = ui.as_weak();
    ui.on_add_task(move || {
        let Some(ui_clone) = ui_weak_add.upgrade() else { return };
        let hour = ui_clone.get_selected_hour() as u32;
        let minute = ui_clone.get_selected_minute() as u32;
        let task_type_str = ui_clone.get_selected_task_type();
        let popup_msg = ui_clone.get_popup_message();
        
        let task_type = match task_type_str.as_str() {
            "关机" => TaskType::Shutdown,
            "重启" => TaskType::Reboot,
            "锁屏" => TaskType::LockScreen,
            "弹窗提醒" => TaskType::Popup,
            _ => TaskType::Shutdown,
        };
        
        let now = Local::now();
        
        // 创建预定时间
        let mut scheduled_time = now
            .with_hour(hour)
            .unwrap_or(now)
            .with_minute(minute)
            .unwrap_or(now)
            .with_second(0)
            .unwrap_or(now)
            .with_nanosecond(0)
            .unwrap_or(now);
        
        // 如果时间已过，设置为明天
        if scheduled_time <= now {
            scheduled_time = scheduled_time + chrono::Duration::days(1);
        }

        // 只为 Popup 任务设置消息
        let message = if task_type == TaskType::Popup {
            Some(popup_msg.to_string())
        } else {
            None
        };

        let task = ScheduledTask {
            id: uuid::Uuid::new_v4().to_string(),
            task_type: task_type.clone(),
            scheduled_time,
            enabled: true,
            message,
        };

        if let Ok(mut sched) = scheduler_add.lock() {
            sched.add_task(task);
        }
        
        // 刷新任务列表
        update_task_list(&ui_clone, &scheduler_add);
    });
    
    // 移除任务回调
    let scheduler_remove = scheduler.clone();
    let ui_weak_remove = ui.as_weak();
    ui.on_remove_task(move |task_id| {
        if let Ok(mut sched) = scheduler_remove.lock() {
            sched.remove_task(&task_id.as_str());
        }
        if let Some(ui_r) = ui_weak_remove.upgrade() {
            update_task_list(&ui_r, &scheduler_remove);
        }
    });
    
    // 切换任务状态回调
    let scheduler_toggle = scheduler.clone();
    let ui_weak_toggle = ui.as_weak();
    ui.on_toggle_task(move |task_id| {
        if let Ok(mut sched) = scheduler_toggle.lock() {
            sched.toggle_task(&task_id.as_str());
        }
        if let Some(ui_t) = ui_weak_toggle.upgrade() {
            update_task_list(&ui_t, &scheduler_toggle);
        }
    });
    
    // 清空所有任务回调
    let scheduler_clear = scheduler.clone();
    let ui_weak_clear = ui.as_weak();
    ui.on_clear_all(move || {
        if let Ok(mut sched) = scheduler_clear.lock() {
            sched.clear_all();
        }
        if let Some(ui_c) = ui_weak_clear.upgrade() {
            update_task_list(&ui_c, &scheduler_clear);
        }
    });
    
    // 退出应用回调
    ui.on_exit_app(|| {
        std::process::exit(0);
    });
    
    // 最小化到托盘回调（隐藏整个应用）- macOS专用
    #[cfg(target_os = "macos")]
    ui.on_minimize_to_tray(|| unsafe {
        let app: id = msg_send![class!(NSApplication), sharedApplication];
        let _: () = msg_send![app, hide:nil];
    });
    
    // 初始加载任务列表
    update_task_list(&ui, &scheduler);
    
    // 运行应用
    ui.run()?;
    
    Ok(())
}

/// 更新任务列表显示
fn update_task_list(ui: &TimerApp, scheduler: &Arc<Mutex<TaskScheduler>>) {
    let tasks = if let Ok(sched) = scheduler.lock() {
        sched.get_tasks()
    } else {
        return;
    };
    
    let mut ids = Vec::new();
    let mut types_vec = Vec::new();
    let mut times = Vec::new();
    let mut enabled = Vec::new();
    
    for task in tasks {
        let task_type_str = match task.task_type {
            TaskType::Shutdown => "关机",
            TaskType::Reboot => "重启",
            TaskType::LockScreen => "锁屏",
            TaskType::Popup => "弹窗提醒",
        };
        
        let time_str = task.scheduled_time.format("%Y-%m-%d %H:%M").to_string();
        
        ids.push(task.id.into());
        types_vec.push(task_type_str.into());
        times.push(time_str.into());
        enabled.push(task.enabled);
    }
    
    ui.set_task_ids(std::rc::Rc::new(slint::VecModel::from(ids)).into());
    ui.set_task_types(std::rc::Rc::new(slint::VecModel::from(types_vec)).into());
    ui.set_task_times(std::rc::Rc::new(slint::VecModel::from(times)).into());
    ui.set_task_enabled(std::rc::Rc::new(slint::VecModel::from(enabled)).into());
}
