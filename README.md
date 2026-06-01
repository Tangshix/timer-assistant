# ⏰ 电脑定时助手

一个基于 Rust + Slint 的跨平台定时任务管理工具，支持 Windows 和 macOS，提供关机、重启、锁屏和弹窗提醒功能。

## ✨ 功能特性

- 🔴 **定时关机** - 在指定时间自动关闭计算机
- 🔄 **定时重启** - 在指定时间自动重启计算机
- 🔒 **定时锁屏** - 在指定时间锁定屏幕
- 💬 **弹窗提醒** - 在指定时间显示自定义提醒消息
- 📋 **任务管理** - 添加、删除、启用/禁用定时任务
- 🗂️ **托盘后台** - 关闭窗口后最小化到系统托盘，后台运行
- ⏱️ **倒计时显示** - 窗口标题栏和托盘图标实时显示最近任务倒计时
- 🌐 **跨平台支持** - 支持 Windows 10/11 和 macOS

## 🛠️ 技术栈

- **GUI 框架**: Slint 1.6
- **系统托盘**: macOS 原生 cocoa API / Windows tray-icon crate
- **Windows API**: windows crate
- **macOS API**: osascript (AppleScript) + cocoa
- **时间处理**: chrono
- **构建**: slint-build（编译 `ui/timer_app.slint`）

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

## 🚀 使用方法

1. **启动程序** - 运行编译后的可执行文件
2. **设置时间** - 使用滑块选择小时和分钟
3. **选择任务** - 从下拉菜单选择任务类型（关机/重启/锁屏/弹窗提醒）
4. **自定义消息** - 选择弹窗提醒时可输入自定义提示内容
5. **添加任务** - 点击"添加定时任务"按钮
6. **管理任务** - 在任务列表中可以：
   - 点击"启用/禁用"切换任务状态
   - 点击"删除"移除任务
7. **托盘运行** - 点击关闭按钮或"最小化到托盘"，程序在后台运行
8. **恢复窗口** - 点击系统托盘图标菜单中的"显示主窗口"

## 📝 注意事项

### Windows 权限要求

关机和重启操作需要管理员权限。程序会自动请求必要的权限，但你可能需要：

1. **以管理员身份运行** - 右键点击程序，选择"以管理员身份运行"
2. **UAC 提示** - 首次执行时可能会弹出 UAC 提示，请点击"是"

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

### macOS 功能说明

- **关机/重启**: 使用 AppleScript 调用系统命令，可能需要确认对话框
- **锁屏**: 使用 `pmset displaysleepnow`
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
├── ui/
│   └── timer_app.slint   # Slint UI 定义（550×650 窗口）
├── src/
│   ├── main.rs           # 主程序入口、UI 回调绑定、macOS 托盘
│   ├── tray.rs           # Windows/Linux 系统托盘（tray-icon crate）
│   ├── windows_api.rs    # 跨平台 API 封装（Windows/macOS/仿真）
│   └── scheduler.rs      # 任务调度器（后台线程每秒检查）
├── fonts/
│   └── PingFang.ttc      # 中文字体
├── build.rs              # 构建脚本（slint-build 编译 UI）
└── Cargo.toml            # 项目配置和依赖
```

## 📄 许可证

MIT License
