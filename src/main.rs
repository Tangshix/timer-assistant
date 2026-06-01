mod tray;
mod windows_api;
mod scheduler;

use eframe::egui;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use chrono::{Local, Timelike};
use scheduler::{TaskScheduler, ScheduledTask, TaskType};

/// 应用程序状态
struct TimerApp {
    /// 任务调度器
    scheduler: Arc<Mutex<TaskScheduler>>,
    /// 当前选中的时间
    selected_hour: u32,
    selected_minute: u32,
    /// 当前选中的任务类型
    selected_task: TaskType,
    /// 弹窗消息内容（仅对 Popup 任务有效）
    popup_message: String,
    /// 是否显示窗口（使用 Arc<AtomicBool> 以便跨线程访问）
    visible: Arc<AtomicBool>,
    /// 上一次检测到的可见状态（用于检测状态变化）
    last_visible: bool,
    /// 重绘通知接收器
    repaint_receiver: Mutex<mpsc::Receiver<bool>>,
    /// 倒计时接收器
    countdown_receiver: Option<mpsc::Receiver<(String, i64)>>,
    /// 托盘图标（需要在 update 中更新）
    tray_icon: Option<tray_icon::TrayIcon>,
}

impl TimerApp {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        
        // 启动一个线程来监听 visible 状态变化并通知重绘
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(50));
                if tx.send(true).is_err() {
                    break;
                }
            }
        });
        
        Self {
            scheduler: Arc::new(Mutex::new(TaskScheduler::new())),
            selected_hour: 22,
            selected_minute: 0,
            selected_task: TaskType::Shutdown,
            popup_message: String::from("这是您的定时提醒消息！"),
            visible: Arc::new(AtomicBool::new(true)),
            last_visible: true,
            repaint_receiver: Mutex::new(rx),
            countdown_receiver: None,
            tray_icon: None,
        }
    }
}

impl eframe::App for TimerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检查是否有重绘通知
        if let Ok(rx) = self.repaint_receiver.lock() {
            while rx.try_recv().is_ok() {
                // 收到通知，继续执行
            }
        }
        
        // 检查并更新倒计时
        if let Some(rx) = &self.countdown_receiver {
            while let Ok((text, remaining_secs)) = rx.try_recv() {
                // 更新托盘图标
                if let Some(ref tray_icon) = self.tray_icon {
                    tray::update_tray_countdown(tray_icon, &text, remaining_secs);
                }
            }
        }
        
        // 从 Arc<AtomicBool> 读取可见状态
        let is_visible = self.visible.load(Ordering::SeqCst);
        
        // 检测可见状态变化（即使在不可见状态下也要检测）
        if is_visible != self.last_visible {
            self.last_visible = is_visible;
            println!("窗口可见性变化: {} -> {}", if self.last_visible { "false" } else { "true" }, is_visible);
            
            // 发送窗口可见性命令
            println!("发送 ViewportCommand::Visible({})", is_visible);
            ctx.send_viewport_cmd(egui::ViewportCommand::Visible(is_visible));
            
            // 同时强制请求重绘
            ctx.request_repaint();
            
            // 如果窗口变为可见，确保它获得焦点
            if is_visible {
                println!("窗口变为可见，发送聚焦命令");
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
        }
        
        // 如果不可见，不渲染UI内容，但要持续请求重绘以检测状态变化
        if !is_visible {
            // 持续请求重绘，确保能检测到 visible 状态的变化
            ctx.request_repaint_after(std::time::Duration::from_millis(100));
            return;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // 标题
            ui.vertical_centered(|ui| {
                ui.add_space(10.0);
                ui.heading("⏰ 电脑定时助手");
                ui.add_space(5.0);
            });
            
            ui.separator();
            ui.add_space(10.0);

            // 时间选择区域
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("⏰ 设置定时时间:");
                    ui.add_space(10.0);
                    ui.label(format!(
                        "{:02}:{:02}",
                        self.selected_hour, self.selected_minute
                    ));
                });
                
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    ui.label("小时:");
                    ui.add(egui::Slider::new(&mut self.selected_hour, 0..=23));
                    
                    ui.label("分钟:");
                    ui.add(egui::Slider::new(&mut self.selected_minute, 0..=59));
                });
            });

            ui.add_space(10.0);

            // 任务类型选择
            ui.group(|ui| {
                ui.label("🎯 选择任务类型:");
                ui.add_space(5.0);
                
                egui::ComboBox::from_label("")
                    .selected_text(match self.selected_task {
                        TaskType::Shutdown => "关机",
                        TaskType::Reboot => "重启",
                        TaskType::LockScreen => "锁屏",
                        TaskType::Popup => "弹窗提醒",
                    })
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_task, TaskType::Shutdown, "关机");
                        ui.selectable_value(&mut self.selected_task, TaskType::Reboot, "重启");
                        ui.selectable_value(&mut self.selected_task, TaskType::LockScreen, "锁屏");
                        ui.selectable_value(&mut self.selected_task, TaskType::Popup, "弹窗提醒");
                    });
                
                // 如果选择了弹窗任务，显示消息输入框
                if self.selected_task == TaskType::Popup {
                    ui.add_space(10.0);
                    ui.label("💬 弹窗消息内容:");
                    ui.add_space(5.0);
                    
                    // 使用多行文本编辑器，新版本 egui 对中文支持更好
                    ui.add(
                        egui::TextEdit::multiline(&mut self.popup_message)
                            .desired_rows(4)
                            .desired_width(f32::INFINITY)
                            .hint_text("请输入弹窗提示内容..."),
                    );
                }
            });

            ui.add_space(15.0);

            // 添加任务按钮
            ui.horizontal_centered(|ui| {
                let button = ui.add_sized(
                    [200.0, 35.0],
                    egui::Button::new("➕ 添加定时任务")
                );
                if button.clicked() {
                    let now = Local::now();
                    
                    // 创建预定时间
                    let mut scheduled_time = now
                        .with_hour(self.selected_hour)
                        .unwrap_or(now)
                        .with_minute(self.selected_minute)
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
                    let message = if self.selected_task == TaskType::Popup {
                        Some(self.popup_message.clone())
                    } else {
                        None
                    };

                    let task = ScheduledTask {
                        id: uuid::Uuid::new_v4().to_string(),
                        task_type: self.selected_task.clone(),
                        scheduled_time,
                        enabled: true,
                        message,
                    };

                    if let Ok(mut scheduler) = self.scheduler.lock() {
                        scheduler.add_task(task);
                    }
                }
            });

            ui.add_space(15.0);
            ui.separator();
            ui.add_space(10.0);
            ui.label("📋 任务列表:");
            ui.add_space(5.0);

            // 任务列表 - 使用 ScrollArea
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    let tasks = {
                        let scheduler = self.scheduler.lock().unwrap();
                        scheduler.get_tasks()
                    };
                                
                    if tasks.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label("暂无任务");
                        });
                    } else {
                        for task in tasks {
                            ui.group(|ui| {
                                ui.horizontal(|ui| {
                                    let icon = match task.task_type {
                                        TaskType::Shutdown => "",
                                        TaskType::Reboot => "",
                                        TaskType::LockScreen => "",
                                        TaskType::Popup => "",
                                    };
                                    
                                    ui.label(icon);
                                    ui.label(match task.task_type {
                                        TaskType::Shutdown => "关机",
                                        TaskType::Reboot => "重启",
                                        TaskType::LockScreen => "锁屏",
                                        TaskType::Popup => "弹窗提醒",
                                    });
                                    ui.label(format!(
                                        " - {}",
                                        task.scheduled_time.format("%Y-%m-%d %H:%M")
                                    ));
                                    
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button("❌").clicked() {
                                            let scheduler = self.scheduler.clone();
                                            let task_id = task.id.clone();
                                            std::thread::spawn(move || {
                                                if let Ok(mut s) = scheduler.lock() {
                                                    s.remove_task(&task_id);
                                                }
                                            });
                                        }
                                        
                                        let status = if task.enabled { "✅" } else { "⏸️" };
                                        if ui.small_button(status).clicked() {
                                            let scheduler = self.scheduler.clone();
                                            let task_id = task.id.clone();
                                            std::thread::spawn(move || {
                                                if let Ok(mut s) = scheduler.lock() {
                                                    s.toggle_task(&task_id);
                                                }
                                            });
                                        }
                                    });
                                });
                            });
                        }
                    }
                });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            // 底部按钮
            ui.horizontal(|ui| {
                if ui.button("🗑️ 清空所有任务").clicked() {
                    if let Ok(mut scheduler) = self.scheduler.lock() {
                        scheduler.clear_all();
                    }
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("❌ 退出").clicked() {
                        std::process::exit(0);
                    }
                    
                    if ui.button("➖ 最小化到托盘").clicked() {
                        self.visible.store(false, Ordering::SeqCst);
                    }
                });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    // 创建应用
    let mut app = TimerApp::new();
    let visible_clone = app.visible.clone();
    let _scheduler_clone = app.scheduler.clone();
    
    // 初始化托盘（传入 visible 状态）
    let (tray_icon, countdown_rx, countdown_tx) = tray::create_tray_icon(visible_clone.clone());
    
    // 启动倒计时更新线程
    let scheduler_clone = app.scheduler.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(1));
            
            // 获取下一个任务的倒计时
            if let Ok(scheduler) = scheduler_clone.lock() {
                if let Some((task_type, remaining_secs)) = scheduler.get_next_countdown() {
                    // 格式化倒计时文本
                    let task_name = match task_type {
                        TaskType::Shutdown => "关机",
                        TaskType::Reboot => "重启",
                        TaskType::LockScreen => "锁屏",
                        TaskType::Popup => "弹窗",
                    };
                    
                    let hours = remaining_secs / 3600;
                    let minutes = (remaining_secs % 3600) / 60;
                    let seconds = remaining_secs % 60;
                    
                    let countdown_text = if hours > 0 {
                        format!("⏰ {} - {:02}:{:02}:{:02}", task_name, hours, minutes, seconds)
                    } else {
                        format!(" {} - {:02}:{:02}", task_name, minutes, seconds)
                    };
                    
                    // 发送倒计时文本和剩余秒数到主线程
                    let _ = countdown_tx.send((countdown_text, remaining_secs));
                } else {
                    // 没有任务
                    let _ = countdown_tx.send(("暂无任务".to_string(), 0));
                }
            }
        }
    });
    
    // 在 TimerApp 中添加倒计时接收器
    let mut app_with_countdown = {
        app.countdown_receiver = Some(countdown_rx);
        app
    };

    // 窗口选项
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("电脑定时助手")
            .with_inner_size([550.0, 650.0])
            .with_min_inner_size([450.0, 500.0])
            .with_icon(eframe::icon_data::from_png_bytes(include_bytes!("../icons/app.png")).unwrap()),
        ..Default::default()
    };

    // 运行应用，配置中文字体
    eframe::run_native(
        "电脑定时助手",
        native_options,
        Box::new(|cc| {
            // 配置中文字体支持
            let mut fonts = egui::FontDefinitions::default();
            
            // 根据平台添加中文字体
            #[cfg(windows)]
            {
                // Windows: 使用宋体
                fonts.font_data.insert(
                    "chinese_font".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../fonts/simsun.ttc"))),
                );
            }
            
            #[cfg(target_os = "macos")]
            {
                // macOS: 使用系统自带中文字体（苹方）
                // 苹方字体在 macOS 上的路径
                fonts.font_data.insert(
                    "chinese_font".to_owned(),
                    std::sync::Arc::new(egui::FontData::from_static(include_bytes!("../fonts/PingFang.ttc"))),
                );
            }
            
            // 将中文字体添加到所有字体家族的最前面
            for family in fonts.families.values_mut() {
                family.insert(0, "chinese_font".to_owned());
            }
            
            cc.egui_ctx.set_fonts(fonts);
            
            // 设置托盘图标
            app_with_countdown.tray_icon = Some(tray_icon);
            
            Ok(Box::new(app_with_countdown))
        }),
    )
}
