# 🚀 快速打包指南

## 一键打包

### 在 Windows 上
双击运行 `package_windows.bat` 或在命令行执行：
```bash
package_windows.bat
```

**输出：** `timer-assistant-windows.zip`

---

### 在 macOS 上
在终端执行：
```bash
./package_macos.sh
```

**输出：** `timer-assistant-macos.dmg` （推荐）或 `dist/timer-assistant-macos.tar.gz`

---

## 📦 分发方式

### 方式一：直接发送文件（最简单）
将生成的 `.zip` 或 `.dmg` 文件通过以下方式发送给用户：
- 微信/QQ 文件传输
- 邮件附件
- 网盘分享
- U盘拷贝

用户收到后：
- **Windows**: 解压 zip，双击 `timer-assistant.exe`
- **macOS**: 打开 dmg，拖拽 App 到应用程序文件夹

---

### 方式二：GitHub Releases（推荐开源项目）

1. **打标签：**
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. **在各平台打包：**
   - Windows: 运行 `package_windows.bat`
   - macOS: 运行 `./package_macos.sh`

3. **上传到 GitHub：**
   - 访问 https://github.com/你的用户名/timer-assistant/releases
   - 点击 "Create a new release"
   - 选择标签 `v0.1.0`
   - 上传打包文件
   - 填写版本说明
   - 点击 "Publish release"

用户可以从 Releases 页面下载。

---

## ✅ 打包前检查

确保以下内容已完成：

- [ ] 代码已提交到 Git
- [ ] 版本号已更新（在 Cargo.toml 中）
- [ ] 在本地测试过所有功能
- [ ] README.md 已更新
- [ ] 清理了不必要的临时文件

---

## 💡 提示

1. **文件大小：** 
   - Windows: 约 5-10 MB（压缩包）
   - macOS: 约 8-15 MB（DMG）

2. **字体文件：** 
   - 如果用户系统已有中文字体，可以移除 fonts 目录以减小体积

3. **首次运行：**
   - Windows 可能显示"未知发布者"警告，点击"仍要运行"即可
   - macOS 需要在"安全性与隐私"中允许运行

4. **管理员权限：**
   - 关机/重启功能在 Windows 上需要管理员权限
   - macOS 需要辅助功能和自动化权限

---

## 🔍 验证打包结果

打包完成后，建议：

1. 在干净的系统中测试（虚拟机最佳）
2. 确认程序能正常启动
3. 测试所有功能（添加任务、托盘、关机等）
4. 检查中文显示是否正常

---

## 📞 需要帮助？

查看详细文档：[PACKAGING.md](PACKAGING.md)
