@echo off
setlocal enabledelayedexpansion

echo ========================================
echo   Timer Assistant - Windows 打包脚本
echo ========================================
echo.

:: 检查是否在正确的目录
if not exist "Cargo.toml" (
    echo [错误] 请在项目根目录运行此脚本
    pause
    exit /b 1
)

:: 清理旧的构建
echo [1/5] 清理旧的构建...
cargo clean
if errorlevel 1 (
    echo [错误] 清理失败
    pause
    exit /b 1
)

:: 编译 Release 版本
echo.
echo [2/5] 编译 Release 版本...
cargo build --release
if errorlevel 1 (
    echo [错误] 编译失败
    pause
    exit /b 1
)

:: 创建发布目录
echo.
echo [3/5] 创建发布目录结构...
set RELEASE_DIR=dist\timer-assistant-windows
if exist "%RELEASE_DIR%" rmdir /s /q "%RELEASE_DIR%"
mkdir "%RELEASE_DIR%"
mkdir "%RELEASE_DIR%\fonts"

:: 复制可执行文件
echo.
echo [4/5] 复制文件到发布目录...
copy "target\release\timer-assistant.exe" "%RELEASE_DIR%\" >nul
copy "README.md" "%RELEASE_DIR%\" >nul
copy "fonts\PingFang.ttc" "%RELEASE_DIR%\fonts\" >nul 2>&1
copy "icons\app.png" "%RELEASE_DIR%\" >nul 2>&1

:: 创建使用说明
echo.
echo [5/5] 生成使用说明...
(
echo # 电脑定时助手 - Windows 版
echo.
echo ## 快速开始
echo.
echo 1. 双击 `timer-assistant.exe` 运行程序
echo 2. 首次运行可能需要授予管理员权限（用于关机/重启功能）
echo 3. 如需开机自启，可将快捷方式放入启动文件夹
echo.
echo ## 注意事项
echo.
echo - 关机和重启功能需要管理员权限
echo - 程序会最小化到系统托盘后台运行
echo - 点击托盘图标可恢复主窗口
echo.
echo ## 系统要求
echo.
echo - Windows 10/11 64位
echo - 无需安装额外依赖
echo.
) > "%RELEASE_DIR%\使用说明.txt"

:: 压缩成 zip
echo.
echo 创建压缩包...
powershell -Command "Compress-Archive -Path '%RELEASE_DIR%\*' -DestinationPath 'timer-assistant-windows.zip' -Force"

echo.
echo ========================================
echo   打包完成！
echo ========================================
echo.
echo 发布文件位置: timer-assistant-windows.zip
echo 发布目录: %RELEASE_DIR%
echo.
echo 可以直接分发 zip 文件给用户
echo.

pause
