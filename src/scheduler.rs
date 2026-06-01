use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskType {
    Shutdown,
    Reboot,
    LockScreen,
    Popup,
}

/// 定时任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledTask {
    pub id: String,
    pub task_type: TaskType,
    pub scheduled_time: DateTime<Local>,
    pub enabled: bool,
    /// 弹窗消息内容（仅对 Popup 任务有效）
    pub message: Option<String>,
}

/// 任务调度器
pub struct TaskScheduler {
    tasks: Arc<Mutex<HashMap<String, ScheduledTask>>>,
}

impl TaskScheduler {
    pub fn new() -> Self {
        let scheduler = Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
        };
        
        // 启动后台监听线程
        scheduler.start_listener();
        
        scheduler
    }

    /// 添加任务
    pub fn add_task(&mut self, task: ScheduledTask) {
        println!("添加任务: {:?}", task);
        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(task.id.clone(), task);
    }

    /// 移除任务
    pub fn remove_task(&mut self, id: &str) {
        println!("移除任务: {}", id);
        let mut tasks = self.tasks.lock().unwrap();
        tasks.remove(id);
    }

    /// 切换任务启用状态
    pub fn toggle_task(&mut self, id: &str) {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(id) {
            task.enabled = !task.enabled;
            println!("切换任务 {} 状态: {}", id, task.enabled);
        }
    }

    /// 获取所有任务
    pub fn get_tasks(&self) -> Vec<ScheduledTask> {
        let tasks = self.tasks.lock().unwrap();
        tasks.values().cloned().collect()
    }

    /// 获取下一个任务的倒计时信息
    /// 返回格式：Some((任务类型, 剩余秒数)) 或 None
    pub fn get_next_countdown(&self) -> Option<(TaskType, i64)> {
        let tasks = self.tasks.lock().unwrap();
        let now = Local::now();
        
        let mut next_task: Option<(TaskType, i64)> = None;
        
        for task in tasks.values() {
            if task.enabled && task.scheduled_time > now {
                let remaining = (task.scheduled_time - now).num_seconds();
                
                // 如果是第一个任务，或者时间更早
                if next_task.is_none() || remaining < next_task.as_ref().unwrap().1 {
                    next_task = Some((task.task_type.clone(), remaining));
                }
            }
        }
        
        next_task
    }

    /// 清空所有任务
    pub fn clear_all(&mut self) {
        let mut tasks = self.tasks.lock().unwrap();
        tasks.clear();
        println!("清空所有任务");
    }

    /// 启动后台监听线程
    fn start_listener(&self) {
        // 使用 Arc 克隆，确保调度器线程和主线程共享同一个任务列表
        let tasks = self.tasks.clone();
        
        thread::spawn(move || {
            println!("调度器监听线程已启动");
            loop {
                let now = Local::now();
                let mut executed_ids = Vec::new();
                let mut tasks_to_execute = Vec::new();
                
                // 检查所有任务
                {
                    let tasks_lock = tasks.lock().unwrap();
                    println!("当前任务数量: {}", tasks_lock.len());
                    for (id, task) in tasks_lock.iter() {
                        println!("检查任务: {} - {} - 启用: {} - 预定时间: {} - 当前时间: {}", 
                            id, 
                            format!("{:?}", task.task_type),
                            task.enabled,
                            task.scheduled_time.format("%Y-%m-%d %H:%M:%S"),
                            now.format("%Y-%m-%d %H:%M:%S")
                        );
                        if task.enabled && task.scheduled_time <= now {
                            println!("✓ 触发任务: {:?}", task);
                            tasks_to_execute.push(task.clone());
                            executed_ids.push(id.clone());
                        }
                    }
                }
                
                // 在独立线程中执行任务，避免阻塞调度器
                if !tasks_to_execute.is_empty() {
                    println!("准备执行 {} 个任务", tasks_to_execute.len());
                }
                for task in tasks_to_execute {
                    thread::spawn(move || {
                        println!("在独立线程中执行任务: {:?}", task.task_type);
                        execute_task(&task);
                    });
                }
                
                // 移除已执行的任务
                if !executed_ids.is_empty() {
                    let mut tasks_lock = tasks.lock().unwrap();
                    for id in &executed_ids {
                        tasks_lock.remove(id);
                    }
                    println!("已移除 {} 个已执行的任务", executed_ids.len());
                }
                
                // 每秒检查一次
                thread::sleep(Duration::from_secs(1));
            }
        });
    }
}

/// 执行任务
fn execute_task(task: &ScheduledTask) {
    match task.task_type {
        TaskType::Shutdown => {
            println!("执行关机");
            #[cfg(windows)]
            {
                if let Err(e) = crate::windows_api::windows::shutdown() {
                    eprintln!("关机失败: {}", e);
                }
            }
            #[cfg(target_os = "macos")]
            {
                if let Err(e) = crate::windows_api::macos::shutdown() {
                    eprintln!("关机失败: {}", e);
                }
            }
            #[cfg(not(windows))]
            #[cfg(not(target_os = "macos"))]
            {
                crate::windows_api::non_windows::shutdown().ok();
            }
        }
        TaskType::Reboot => {
            println!("执行重启");
            #[cfg(windows)]
            {
                if let Err(e) = crate::windows_api::windows::reboot() {
                    eprintln!("重启失败: {}", e);
                }
            }
            #[cfg(target_os = "macos")]
            {
                if let Err(e) = crate::windows_api::macos::reboot() {
                    eprintln!("重启失败: {}", e);
                }
            }
            #[cfg(not(windows))]
            #[cfg(not(target_os = "macos"))]
            {
                crate::windows_api::non_windows::reboot().ok();
            }
        }
        TaskType::LockScreen => {
            println!("执行锁屏");
            #[cfg(windows)]
            {
                if let Err(e) = crate::windows_api::windows::lock_screen() {
                    eprintln!("锁屏失败: {}", e);
                }
            }
            #[cfg(target_os = "macos")]
            {
                if let Err(e) = crate::windows_api::macos::lock_screen() {
                    eprintln!("锁屏失败: {}", e);
                }
            }
            #[cfg(not(windows))]
            #[cfg(not(target_os = "macos"))]
            {
                crate::windows_api::non_windows::lock_screen().ok();
            }
        }
        TaskType::Popup => {
            println!("执行弹窗");
            let message = task.message.as_deref().unwrap_or("这是您的定时提醒消息！");
            #[cfg(windows)]
            {
                crate::windows_api::windows::show_popup(
                    "定时提醒",
                    message,
                );
            }
            #[cfg(target_os = "macos")]
            {
                crate::windows_api::macos::show_popup(
                    "定时提醒",
                    message,
                );
            }
            #[cfg(not(windows))]
            #[cfg(not(target_os = "macos"))]
            {
                crate::windows_api::non_windows::show_popup(
                    "定时提醒",
                    message,
                );
            }
        }
    }
}
