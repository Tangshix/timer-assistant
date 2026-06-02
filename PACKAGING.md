# 打包分发指南

本文档说明如何将 Timer Assistant 打包分发给用户。

## 📦 快速打包

### Windows 系统

在项目根目录运行：

```bash
package_windows.bat
```

**输出文件：**
- `timer-assistant-windows.zip` - 可直接分发的压缩包
- `dist/timer-assistant-windows/` - 未压缩的发布目录

**包含内容：**
- timer-assistant.exe (主程序)
- fonts/ (中文字体)
- app.png (应用图标)
- README.md (使用说明)
- 使用说明.txt (快速入门)

---

### macOS 系统

在项目根目录运行：

```bash
./package_macos.sh
```

**输出文件：**
- `timer-assistant-macos.dmg` - DMG 镜像（推荐分发）
- `dist/timer-assistant-macos.tar.gz` - tar.gz 压缩包
- `dist/TimerAssistant.app` - macOS 应用包

**包含内容：**
- TimerAssistant.app (标准 macOS 应用包)
- Info.plist (应用配置)
- Resources/ (字体、图标等资源)
- README.md (使用说明)

---

## 🔧 编译优化

项目已配置了 Release 优化选项（在 Cargo.toml 中）：

```toml
[profile.release]
strip = true          # 去除调试符号，减小体积
lto = true            # 链接时优化
codegen-units = 1     # 优化编译单元
panic = 'abort'       # 减小二进制大小
```

这些优化可以：
- 减小可执行文件大小约 30-50%
- 提升运行性能
- 移除调试信息，保护代码

---

## 📋 分发建议

### 方式一：直接分发压缩包（最简单）

**适用场景：** 小范围分享、测试版本

**步骤：**
1. 运行对应平台的打包脚本
2. 将生成的 `.zip` (Windows) 或 `.dmg` (macOS) 文件发送给用户
3. 用户解压后即可使用

---

### 方式二：上传到 GitHub Releases（推荐）

**适用场景：** 开源项目、公开发布

**步骤：**

1. **创建 Git tag：**
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. **在各平台编译打包：**
   - Windows: 运行 `package_windows.bat`
   - macOS: 运行 `./package_macos.sh`

3. **上传到 GitHub：**
   - 进入项目的 Releases 页面
   - 点击 "Create a new release"
   - 选择刚创建的 tag
   - 上传打包文件：
     - `timer-assistant-windows.zip`
     - `timer-assistant-macos.dmg`
   - 填写版本说明
   - 发布

---

### 方式三：使用安装包工具（专业）

#### Windows - 使用 Inno Setup

1. 下载并安装 [Inno Setup](https://jrsoftware.org/isdl.php)
2. 创建 `.iss` 脚本文件
3. 编译生成专业的 `.exe` 安装程序

**优点：**
- 支持自定义安装路径
- 可创建开始菜单快捷方式
- 支持卸载功能
- 更专业的用户体验

#### macOS - 使用 create-dmg

1. 安装 create-dmg：
   ```bash
   brew install create-dmg
   ```

2. 创建精美的 DMG：
   ```bash
   create-dmg \
     --volname "Timer Assistant" \
     --window-pos 200 120 \
     --window-size 800 400 \
     --icon-size 100 \
     --app-drop-link 600 185 \
     "Timer-Assistant.dmg" \
     "dist/TimerAssistant.app"
   ```

---

## 🎯 跨平台编译（高级）

如果你想在单个平台上编译所有平台版本，可以使用交叉编译：

### 从 macOS 编译 Windows 版本

```bash
# 安装 cross 工具
cargo install cross

# 交叉编译 Windows 版本
cross build --release --target x86_64-pc-windows-gnu
```

### 从 Linux 编译所有平台

需要安装对应的目标工具链和交叉编译器。

---

## 📊 文件大小参考

经过优化后，典型文件大小：

| 平台 | 可执行文件 | 压缩包 | 包含资源 |
|------|-----------|--------|---------|
| Windows | ~8-12 MB | ~3-5 MB | +20 MB (字体) |
| macOS | ~10-15 MB | ~4-6 MB | +20 MB (字体) |

**注意：** 字体文件较大，如果用户系统已有中文字体，可以考虑不打包字体。

---

## ✅ 发布前检查清单

- [ ] 在 Release 模式下编译 (`cargo build --release`)
- [ ] 测试可执行文件能否正常运行
- [ ] 测试所有功能（关机、重启、锁屏、弹窗）
- [ ] 确认托盘功能正常
- [ ] 检查中文字体显示是否正常
- [ ] 准备 README 和使用说明
- [ ] 更新版本号
- [ ] 编写版本更新日志
- [ ] 在多个系统上测试（如果可能）

---

## 🚀 自动化打包（CI/CD）

可以配置 GitHub Actions 自动打包：

创建 `.github/workflows/release.yml`：

```yaml
name: Release Build

on:
  push:
    tags:
      - 'v*'

jobs:
  build-windows:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
      - uses: actions/upload-artifact@v3
        with:
          name: timer-assistant-windows
          path: target/release/timer-assistant.exe

  build-macos:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - run: cargo build --release
      - uses: actions/upload-artifact@v3
        with:
          name: timer-assistant-macos
          path: target/release/timer-assistant
```

---

## 💡 常见问题

### Q: 为什么可执行文件这么大？
A: Rust 静态链接所有依赖，且包含调试信息。Release 模式已启用 strip 和 LTO 优化来减小体积。

### Q: 用户可以不安装 Rust 直接运行吗？
A: 可以！编译后的可执行文件是独立的，不需要用户安装 Rust 或任何依赖。

### Q: 如何进一步减小文件体积？
A: 
- 移除不必要的字体文件
- 使用 `cargo bloat` 分析体积来源
- 考虑动态链接系统库（但会降低便携性）

### Q: 是否需要数字签名？
A: 
- **Windows**: 非必需，但没有签名的程序会显示"未知发布者"警告
- **macOS**: 从 App Store 外分发的应用可能需要公证（Notarization）

---

## 📞 获取帮助

如有问题，请查看：
- 项目 README.md
- GitHub Issues
- 相关文档
