# 电脑定时助手 (Timer Assistant)

> 基于 Rust + Slint 的跨平台桌面定时任务管理工具

一个轻量级的桌面应用，让你在指定时间自动执行关机、重启、锁屏或弹窗提醒。支持 Windows 和 macOS，最小化到系统托盘后台运行，实时倒计时显示。


## 界面预览
![img_1.png](img.png)


## 功能特性

### 核心功能

| 功能 | 说明 |
|------|------|
| 定时关机 | 在指定时间自动关闭计算机 |
| 定时重启 | 在指定时间自动重启计算机 |
| 定时锁屏 | 在指定时间锁定屏幕 |
| 弹窗提醒 | 在指定时间显示自定义提醒消息 |
| 任务管理 | 添加、删除、启用/禁用定时任务 |
| 托盘后台 | 关闭窗口后最小化到系统托盘，后台运行 |
| 倒计时显示 | 窗口标题栏和托盘图标实时显示最近任务倒计时 |

### 交互设计

- **时间选择**：滑块选择小时 (0-23) 和分钟 (0-59)
- **任务类型**：下拉菜单选择任务类型
- **智能调度**：若选择的时间已过，自动调度到明天
- **批量操作**：支持清空所有任务
- **状态切换**：一键启用/禁用任务

## 技术栈

```
┌─────────────────────────────────────────────────────────────┐
│                      Slint UI (1.6)                         │
├─────────────────────────────────────────────────────────────┤
│                      Rust 应用层                             │
├──────────────────┬──────────────────┬───────────────────────┤
│   macOS 托盘      │ Windows/Linux    │    调度器              │
│   (cocoa/objc)   │  (tray-icon)     │    (std::thread)      │
├──────────────────┴──────────────────┴───────────────────────┤
│                    平台抽象层                                 │
├──────────────────┬──────────────────┬───────────────────────┤
│   macOS          │   Windows        │   (其他系统)           │
│   osascript      │   Win32 API      │   控制台输出            │
│   pmset          │   windows crate  │                       │
└──────────────────┴──────────────────┴───────────────────────┘
```

### 依赖概览

| 类别 | 依赖 | 用途 |
|------|------|------|
| GUI | `slint 1.6` | 跨平台 UI 框架 |
| 托盘 | `tray-icon 0.19` + `tao 0.31` | Windows/Linux 系统托盘 |
| 平台 | `cocoa 0.25` + `objc 0.2` | macOS 原生 API |
| 平台 | `windows 0.58` | Windows API 绑定 |
| 时间 | `chrono 0.4` | 日期时间处理 |
| ID | `uuid 1` | 任务唯一标识生成 |
| 图标 | `image 0.24` | 图标处理 |
| 构建 | `slint-build 1.6` | 编译 Slint UI 文件 |

## 项目架构

```
timer-assistant/
├── ui/
│   └── timer_app.slint      # Slint UI 定义 (550×650 窗口)
├── src/
│   ├── main.rs              # 入口、UI 回调、macOS 托盘 (414 行)
│   ├── scheduler.rs         # 任务模型 + 后台调度线程 (253 行)
│   ├── tray.rs              # Windows/Linux 托盘实现 (108 行)
│   └── windows_api.rs       # 平台抽象层 (203 行)
├── icons/                   # 应用图标资源
├── build.rs                 # 构建脚本 (Slint 编译 + Windows 资源嵌入)
└── Cargo.toml
```

### 核心模块

#### 1. UI 层 (`ui/timer_app.slint`)

Slint 声明式 UI，定义了：

- **TimerApp**：主窗口组件，包含时间选择器、任务类型选择、任务列表
- **TimerItem**：任务列表项组件，显示任务信息和操作按钮
- **数据绑定**：通过四个并行数组 (`task-ids`, `task-types`, `task-times`, `task-enabled`) 与 Rust 后端同步

#### 2. 调度器 (`scheduler.rs`)

```rust
// 核心数据结构
struct ScheduledTask {
    id: String,                    // UUID v4
    task_type: TaskType,           // Shutdown/Reboot/LockScreen/Popup
    scheduled_time: DateTime<Local>,
    enabled: bool,
    message: Option<String>,       // 仅 Popup 类型使用
}

struct TaskScheduler {
    tasks: Arc<Mutex<HashMap<String, ScheduledTask>>>,
}
```

- **后台线程**：每秒检查到期任务，独立线程执行
- **共享状态**：`Arc<Mutex<HashMap>>` 实现线程安全的任务管理
- **自动调度**：已过时间自动推迟到明天

#### 3. 平台抽象层 (`windows_api.rs`)

使用 `#[cfg(target_os)]` 条件编译，三个平台模块：

| 平台 | 关机 | 重启 | 锁屏 | 弹窗 |
|------|------|------|------|------|
| Windows | `ExitWindowsEx` | `ExitWindowsEx` | `LockWorkStation` | `MessageBoxW` |
| macOS | `osascript` | `osascript` | `pmset displaysleepnow` | `display dialog` |
| Linux | 模拟输出 | 模拟输出 | 模拟输出 | 模拟输出 |

#### 4. 系统托盘

**macOS** (内置于 `main.rs`)：
- 使用 `cocoa` + `objc` crate 直接调用 Cocoa API
- 创建 `NSStatusItem`，加载 PNG 图标
- 注册自定义 Objective-C 类处理菜单事件

**Windows/Linux** (`tray.rs`)：
- 使用 `tray-icon` crate
- 编译时嵌入图标 (`include_bytes!`)
- 通过 channel 与主线程通信

## 关键技术实现

### 1. Slint UI 与 Rust 的数据同步

```rust
// Rust -> Slint: 使用 VecModel 同步任务列表
fn update_task_list(ui: &TimerApp, scheduler: &TaskScheduler) {
    let tasks = scheduler.get_tasks();
    let ids: Vec<SharedString> = tasks.iter().map(|t| t.id.clone().into()).collect();
    let types: Vec<SharedString> = tasks.iter().map(|t| t.task_type.to_string().into()).collect();
    // ...
    ui.set_task_ids(Rc::new(VecModel::from(ids)).into());
}
```

### 2. 跨平台条件编译

```rust
// windows_api.rs 中的平台分发
#[cfg(target_os = "windows")]
pub mod windows { /* Win32 API 调用 */ }

#[cfg(target_os = "macos")]
pub mod macos { /* osascript + pmset */ }

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
pub mod non_windows { /* 仿真模式 */ }
```

### 3. macOS 原生集成

```rust
// 创建系统托盘
unsafe {
    let status_item = ns_status_bar.statusItemWithLength(-1.0);
    let button = status_item.button();
    // 设置图标、标题、菜单...
    
    // 注册自定义 Objective-C 类处理点击事件
    let cls = ClassDecl::new("TrayClickHandler", class!(NSObject)).unwrap();
    cls.add_method(sel!(showWindow:), show_window as extern "C" fn(&Object, Sel, id));
}
```

### 4. Windows 权限提升

```rust
// 启用关机权限
fn enable_shutdown_privilege() -> Result<()> {
    unsafe {
        let mut token = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token)?;
        // 查找并启用 SeShutdownPrivilege...
    }
}
```

### 5. 构建时资源处理

```rust
// build.rs
fn main() {
    // 编译 Slint UI
    slint_build::compile("ui/timer_app.slint").unwrap();
    
    // Windows: 转换图标并嵌入资源
    #[cfg(target_os = "windows")]
    {
        // PNG -> ICO 转换
        // 生成 .rc 文件
        // embed_resource::compile()
    }
}
```

## 构建与运行

### 前置要求

| 平台 | 要求 |
|------|------|
| Windows | Rust 工具链、Visual Studio Build Tools |
| macOS | Rust 工具链、Xcode Command Line Tools |

### 编译运行

```bash
# 开发模式运行
cargo run

# Release 构建
cargo build --release

# 运行图标生成示例
cargo run --example generate_icon
```

### 打包分发

```bash
# macOS: 生成 App Bundle + DMG + tar.gz
./package_macos.sh

# Windows: 生成 zip 压缩包
package_windows.bat
```

## 使用方法

1. **设置时间** - 拖动滑块选择小时和分钟
2. **选择任务** - 从下拉菜单选择：弹窗提醒 / 锁屏 / 重启 / 关机
3. **自定义消息** - 选择"弹窗提醒"时可输入提示内容
4. **添加任务** - 点击"添加定时任务"
5. **管理任务** - 在列表中启用/禁用或删除任务
6. **托盘运行** - 关闭窗口自动最小化到托盘
7. **恢复窗口** - 点击托盘图标菜单"显示主窗口"

## 权限说明

### Windows

- 关机/重启需要管理员权限
- 程序通过 `requireAdministrator` manifest 自动请求提权
- 首次运行可能触发 UAC 提示

### macOS

- 关机/重启：通过 AppleScript 调用
- 锁屏：使用 `pmset displaysleepnow`（休眠显示屏）
- 弹窗：AppleScript `display dialog`
- 需要在「系统偏好设置 → 安全性与隐私」中授权终端应用

## Release 优化

```toml
[profile.release]
strip = true        # 去除调试符号
lto = true          # 链接时优化
codegen-units = 1   # 单编译单元，更好优化
panic = 'abort'     # panic 时直接终止
```

## License

MIT License
