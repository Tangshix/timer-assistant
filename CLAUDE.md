# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在此仓库中工作时提供指导。

## 项目

电脑定时助手 — 跨平台桌面应用，用于在指定时间执行系统操作（关机、重启、锁屏、弹窗提醒）。使用 Rust + Slint UI 构建。

## 构建与运行

```bash
# 构建（build.rs 通过 slint-build 编译 ui/timer_app.slint）
cargo build

# 运行
cargo run

# 运行图标生成示例
cargo run --example generate_icon
```

跨平台构建使用 `#[cfg]` 条件编译，无需 feature flag。macOS 使用 `cocoa` crate 进行原生集成，Windows 使用 `windows` crate 调用关机/重启/锁屏 API。

## 架构

`src/` 下四个源文件：

- **`main.rs`** — 入口文件。创建 Slint `TimerApp` 窗口，绑定 UI 回调（`on_add_task`、`on_remove_task`、`on_toggle_task`、`on_clear_all`、`on_exit_app`、`on_minimize_to_tray`）。使用 `Arc<Mutex<TaskScheduler>>` 共享状态，启动后台倒计时线程。

- **`scheduler.rs`** — 核心数据模型：`TaskType` 枚举（Shutdown/Reboot/LockScreen/Popup）、`ScheduledTask` 结构体（id、type、time、enabled、message）。`TaskScheduler` 封装 `Arc<Mutex<HashMap<String, ScheduledTask>>>`。`start_listener` 启动线程每秒检查到期任务并通过 `execute_task` 分发执行。

- **`tray.rs`** — 系统托盘图标，使用 `tray-icon` crate。程序化生成 32×32 像素时钟图标，内置 5×7 像素字体用于在托盘图标上渲染倒计时数字。**当前已禁用** — main.rs 中的 `create_tray_icon` 调用被注释掉了。

- **`windows_api.rs`** — 平台抽象层，三个 `#[cfg]` 模块：`windows`（通过 `windows` crate 调用 Win32 API）、`macos`（通过 `Command` 调用 osascript/pmset）、`non_windows`（仿真模式，仅打印输出）。

**UI**：`ui/timer_app.slint` 定义 550×650 窗口，包含时间滑块、任务类型选择器和可滚动任务列表。Slint 类型 `TimerApp` 通过 `slint::include_modules!()` 导入 Rust。

## 关键模式

- **任务 ID**：创建时生成 UUID v4 字符串，作为 HashMap 的键。
- **时间调度**：仅使用小时和分钟（无日期选择）。若所选时间已过，自动调度到明天。
- **UI 并行数组**：Slint 模型使用四个并行数组（`task-ids`、`task-types`、`task-times`、`task-enabled`），通过 `update_task_list()` 从 Rust 同步。
- **条件编译**：平台相关代码在 `windows_api.rs` 中通过 `cfg(target_os)` 区分。调度器的 `execute_task` 分发到对应平台模块。

## 备注

- README/PROJECT_SUMMARY.md 中提到 "egui"，但实际代码使用 **Slint 1.6** — 文档已过时。
- `tokio` 已声明为依赖但未使用 — 所有并发均使用 `std::thread`。
- `fonts/` 目录下的字体文件（共约 85MB）已入库但当前代码未引用。
- Windows 执行关机/重启操作需要 `requireAdministrator` 权限（见 `timer-assistant.exe.manifest`）。
