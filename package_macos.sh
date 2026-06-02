#!/bin/bash

set -e

echo "========================================"
echo "  Timer Assistant - macOS 打包脚本"
echo "========================================"
echo ""

# 检查是否在正确的目录
if [ ! -f "Cargo.toml" ]; then
    echo "[错误] 请在项目根目录运行此脚本"
    exit 1
fi

# 清理旧的构建
echo "[1/5] 清理旧的构建..."
cargo clean

# 编译 Release 版本
echo ""
echo "[2/5] 编译 Release 版本..."
cargo build --release

# 创建发布目录结构
echo ""
echo "[3/5] 创建应用包结构..."
APP_NAME="TimerAssistant.app"
RELEASE_DIR="dist/$APP_NAME"

# 删除旧目录
if [ -d "$RELEASE_DIR" ]; then
    rm -rf "$RELEASE_DIR"
fi

# 创建 macOS App Bundle 结构
mkdir -p "$RELEASE_DIR/Contents/MacOS"
mkdir -p "$RELEASE_DIR/Contents/Resources"

# 复制可执行文件
cp "target/release/timer-assistant" "$RELEASE_DIR/Contents/MacOS/"

# 复制资源文件
cp -r "fonts" "$RELEASE_DIR/Contents/Resources/" 2>/dev/null || true
cp "icons/app.png" "$RELEASE_DIR/Contents/Resources/" 2>/dev/null || true
cp "icons/app.icns" "$RELEASE_DIR/Contents/Resources/app.icns" 2>/dev/null || true

# 创建 Info.plist
cat > "$RELEASE_DIR/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Timer Assistant</string>
    <key>CFBundleDisplayName</key>
    <string>电脑定时助手</string>
    <key>CFBundleIdentifier</key>
    <string>com.timerassistant.app</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleExecutable</key>
    <string>timer-assistant</string>
    <key>CFBundleIconFile</key>
    <string>app</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
</dict>
</plist>
EOF

# 创建 PkgInfo
echo "APPL????" > "$RELEASE_DIR/Contents/PkgInfo"

# 创建 README
echo ""
echo "[4/5] 生成使用说明..."
cat > "dist/README.md" << 'EOF'
# 电脑定时助手 - macOS 版

## 快速开始

### 方式一：使用 App 包（推荐）
1. 双击 `TimerAssistant.app` 运行程序
2. 首次运行可能需要在「系统偏好设置」→「安全性与隐私」中允许运行

### 方式二：使用命令行
```bash
chmod +x timer-assistant
./timer-assistant
```

## 权限设置

macOS 系统对自动化脚本有严格的安全限制，使用前需要授权：

### 1. 辅助功能权限
- 打开「系统偏好设置」→「安全性与隐私」→「隐私」
- 选择「辅助功能」
- 点击左下角锁图标解锁
- 添加 Terminal 或你的终端应用
- 确保勾选了该应用

### 2. 自动化权限
- 在同一界面选择「自动化」
- 确保允许 AppleScript 控制计算机

## 功能说明

- **关机/重启**: 使用 AppleScript 调用系统命令
- **锁屏**: 使用 `pmset displaysleepnow`
- **弹窗提醒**: 使用 AppleScript 的 `display dialog` 命令

## 系统要求

- macOS 10.15 或更高版本
- Intel 或 Apple Silicon (M1/M2) 处理器
EOF

# 复制到发布目录
cp "dist/README.md" "$RELEASE_DIR/Contents/Resources/"

# 压缩成 DMG（如果 hdiutil 可用）
echo ""
echo "[5/5] 创建压缩包..."

# 先创建 tar.gz
cd dist
tar -czf "timer-assistant-macos.tar.gz" "$APP_NAME" README.md
cd ..

# 如果支持，也创建 DMG
if command -v hdiutil &> /dev/null; then
    echo "创建 DMG 镜像..."
    hdiutil create -volname "Timer Assistant" \
        -srcfolder "dist/$APP_NAME" \
        -ov -format UDZO \
        "timer-assistant-macos.dmg"
    
    echo ""
    echo "✓ DMG 文件已创建: timer-assistant-macos.dmg"
fi

echo ""
echo "========================================"
echo "  打包完成！"
echo "========================================"
echo ""
echo "发布文件:"
echo "  - dist/timer-assistant-macos.tar.gz (通用压缩包)"
if [ -f "timer-assistant-macos.dmg" ]; then
    echo "  - timer-assistant-macos.dmg (DMG 镜像，推荐)"
fi
echo "  - dist/$APP_NAME (App 包)"
echo ""
echo "建议分发 DMG 文件给用户（更友好的安装体验）"
echo ""
