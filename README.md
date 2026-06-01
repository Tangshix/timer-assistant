# ⏰ 电脑定时助手

一个基于 Rust + egui 的跨平台定时任务管理工具，支持 Windows 和 macOS，提供关机、重启、锁屏和弹窗提醒功能。

## ✨ 功能特性

- 🔴 **定时关机** - 在指定时间自动关闭计算机
- 🔄 **定时重启** - 在指定时间自动重启计算机  
- 🔒 **定时锁屏** - 在指定时间锁定屏幕
- 💬 **弹窗提醒** - 在指定时间显示提醒消息
- 📋 **任务管理** - 添加、删除、启用/禁用定时任务
- 🗂️ **托盘后台** - 最小化到系统托盘，后台运行
- 🎨 **简洁界面** - 基于 egui 的现代化 UI
- 🌐 **跨平台支持** - 支持 Windows 10/11 和 macOS

## 🛠️ 技术栈

- **GUI 框架**: egui + eframe
- **系统托盘**: tray-icon + tao
- **异步运行时**: tokio
- **Windows API**: windows crate
- **macOS API**: osascript (AppleScript)
- **时间处理**: chrono
- **配置管理**: serde + toml

## 📦 安装与编译

### 前置要求

#### Windows
- Rust 工具链（最新稳定版）
- Windows 10/11
- Visual Studio Build Tools（包含 C++ 构建工具）

#### macOS
- Rust 工具链（最新稳定版）
- macOS 10.15 或更高版本
- Xcode Command Line Tools

### 编译步骤

```bash
# 进入项目目录
cd timer-assistant

# 编译项目
cargo build --release

# 运行程序
cargo run
```

编译后的可执行文件位于 `target/release/timer-assistant.exe`

## 🚀 使用方法

1. **启动程序** - 运行 `timer-assistant.exe`
2. **设置时间** - 使用滑块选择小时和分钟
3. **选择任务** - 从下拉菜单选择任务类型（关机/重启/锁屏/弹窗）
4. **添加任务** - 点击"添加定时任务"按钮
5. **管理任务** - 在任务列表中可以：
   - 点击 ✅/⏸️ 启用或禁用任务
   - 点击 ❌ 删除任务
6. **托盘运行** - 点击"最小化到托盘"让程序在后台运行
7. **恢复窗口** - 点击系统托盘图标恢复主窗口

## 📝 注意事项

### Windows 权限要求

关机和重启操作需要管理员权限。程序会自动请求必要的权限，但你可能需要：

1. **以管理员身份运行** - 右键点击程序，选择“以管理员身份运行”
2. **UAC 提示** - 首次执行时可能会弹出 UAC 提示，请点击“是”

### macOS 权限要求

macOS 系统对自动化脚本有严格的安全限制，使用前需要授权：

1. **辅助功能权限**
   - 打开「系统偏好设置」→「安全性与隐私」→「隐私」
   - 选择「辅助功能」
   - 点击左下角锁图标解锁
   - 添加 Terminal、iTerm2 或你的终端应用
   - 确保勾选了该应用

2. **自动化权限**
   - 在同一界面选择「自动化」
   - 确保允许 AppleScript 控制计算机

3. **完全磁盘访问权限**（可选）
   - 如果遇到权限问题，可以授予「完全磁盘访问权限」

### macOS 功能说明

- **关机/重启**: 使用 AppleScript 调用系统命令，可能需要确认对话框
- **锁屏**: 
  - macOS Monterey (12.0) 及以上：使用 `pmset displaysleepnow`
  - 旧版本：使用快捷键 Control+Command+Q
- **弹窗**: 使用 AppleScript 的 `display dialog` 命令

### 混合关机（Windows 8+）

Windows 8 及更高版本默认启用"快速启动"功能，这可能导致关机操作执行的是混合关机而非完全断电。如需完全关机：

```powershell
# 禁用快速启动（需要管理员权限）
powercfg /h off
```

## 🏗️ 项目结构

```
timer-assistant/
├── src/
│   ├── main.rs          # 主程序入口和 UI
│   ├── tray.rs          # 系统托盘功能
│   ├── windows_api.rs   # Windows API 封装
│   ├── macos_api.rs     # macOS API 封装（集成在 windows_api.rs）
│   └── scheduler.rs     # 任务调度器
├── fonts/
│   └── simsun.ttc       # Windows 中文字体
├── icons/
│   └── app.png          # 应用图标
├── Cargo.toml           # 项目配置和依赖
├── README.md            # 说明文档
├── QUICKSTART.md        # 快速开始指南
└── PROJECT_SUMMARY.md   # 项目总结
```

## 🔧 开发说明

### 添加新功能

1. 在 `scheduler.rs` 中添加新的 `TaskType` 枚举值
2. 在 `execute_task` 函数中实现对应的执行逻辑
3. 在 `main.rs` 的 UI 中添加新选项

### Windows API 调用

所有 Windows API 调用都封装在 `windows_api.rs` 中：

- `shutdown()` - 关机
- `reboot()` - 重启
- `lock_screen()` - 锁屏
- `show_popup()` - 弹窗

### 任务调度

任务调度器使用 tokio 异步运行时，每秒检查一次是否有需要执行的任务。

## 📄 许可证

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📮 联系方式

如有问题或建议，请提交 Issue。
# timer-assistant
