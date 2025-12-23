# 便携版制作说明

## 概述

本项目支持两种运行模式：

1. **传统模式**：数据保存在 `C:\Users\<用户>\AppData\Local\syj_require_list\`
2. **便携模式**：数据保存在程序目录的 `data` 文件夹中（推荐用于 U 盘或多台电脑）

## 便携模式触发条件

程序会在以下情况自动启用便携模式：

- 程序目录下存在 `portable.txt` 文件，或
- 程序目录下存在 `data` 文件夹

## 制作便携版

### 方法一：使用自动化脚本（推荐）

#### PowerShell 脚本（推荐）
```powershell
.\make-portable.ps1
```

#### 批处理脚本
```cmd
make-portable.bat
```

脚本会自动完成以下操作：
1. 打包应用（`npm run tauri build`）
2. 提取生成的 exe 文件
3. 创建便携版文件夹结构
4. 生成使用说明
5. 打包成 ZIP 文件

输出文件：
- `portable-release/` - 便携版文件夹
- `syj_require_list_portable_v0.1.0.zip` - 便携版压缩包

### 方法二：手动制作

1. **打包应用**
   ```bash
   npm run tauri build
   ```

2. **找到生成的 exe 文件**
   - 位置：`src-tauri\target\release\syj_require_list.exe`

3. **创建文件夹结构**
   ```
   my-portable-app/
   ├── syj_require_list.exe
   ├── portable.txt       (创建空文件)
   └── data/              (创建空文件夹)
   ```

4. **创建 portable.txt**
   - 在程序目录创建一个空的 `portable.txt` 文件
   - 这是便携模式的标记文件

5. **创建 data 文件夹**
   - 用于存储数据库、图片和配置文件

## 使用便携版

### 分发

将生成的 `syj_require_list_portable_v0.1.0.zip` 发送给用户。

### 安装（解压）

1. 解压 ZIP 文件到任意位置（例如：`D:\Apps\TodoList\`）
2. 确保文件夹包含：
   - `syj_require_list.exe`
   - `portable.txt`
   - `data/` 文件夹

### 首次运行

1. 双击 `syj_require_list.exe` 运行
2. 如果提示缺少 WebView2：
   - 访问：https://developer.microsoft.com/zh-cn/microsoft-edge/webview2/
   - 下载并安装 "Evergreen Bootstrapper"
   - 重新运行程序

### 数据存储

所有数据都保存在 `data` 文件夹中：
```
data/
├── todos.db          # SQLite 数据库
├── images/           # 图片附件
└── config.json       # 配置文件
```

### 迁移到其他电脑

1. 复制整个程序文件夹到新电脑
2. 确保新电脑已安装 WebView2
3. 直接运行，数据自动保留

## 技术实现

### 便携模式检测逻辑

程序启动时会执行以下检测（`src-tauri/src/config.rs`）：

```rust
fn get_default_root_dir() -> String {
    // 1. 获取程序所在目录
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let portable_dir = exe_dir.join("data");

            // 2. 检查是否存在 data 文件夹或 portable.txt
            if portable_dir.exists() || exe_dir.join("portable.txt").exists() {
                return portable_dir.to_string_lossy().to_string();
            }
        }
    }

    // 3. 否则使用 AppData 目录
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("syj_require_list")
        .to_string_lossy()
        .to_string()
}
```

### 数据路径

- **便携模式**: `<程序目录>/data/`
- **传统模式**: `C:\Users\<用户>\AppData\Local\syj_require_list\`

## 常见问题

### Q: 为什么不默认启用便携模式？

A: 为了兼容性和安全性：
- 传统模式符合 Windows 应用规范
- 便携模式需要手动创建标记文件，避免意外修改程序目录
- 用户可以根据需求选择模式

### Q: 如何从传统模式切换到便携模式？

A: 两种方法：

**方法 1：使用设置窗口迁移（推荐）**
1. 在程序中打开设置
2. 修改数据目录为 `<程序目录>\data`
3. 点击"迁移数据"按钮
4. 在程序目录创建 `portable.txt` 文件

**方法 2：手动迁移**
1. 找到 AppData 目录：`C:\Users\<用户>\AppData\Local\syj_require_list\`
2. 复制所有内容到程序目录的 `data` 文件夹
3. 在程序目录创建 `portable.txt` 文件
4. 重启程序

### Q: 便携版需要安装吗？

A: 不需要安装，解压即可使用。但需要确保：
- Windows 10 或更高版本
- 已安装 WebView2 运行时

### Q: 可以直接运行 exe 而不打包吗？

A: 可以。开发模式下：
1. 在程序目录创建 `portable.txt`
2. 运行 `npm run tauri dev`
3. 数据会保存到开发目录的 `data` 文件夹

### Q: 打包文件太大怎么办？

A: 当前配置使用 `downloadBootstrapper` 模式，安装包约 2-3MB。如果需要离线安装：
- 修改 `tauri.conf.json`
- 将 `webviewInstallMode.type` 改为 `fixedRuntime`
- 安装包会增大到约 150MB

## 版本更新

### 更新便携版

1. 下载新版本的便携版 ZIP
2. 解压到新位置
3. 复制旧版本的 `data` 文件夹到新版本
4. 运行新版本程序

### 保留旧版本

如果需要保留旧版本作为备份：
1. 重命名旧版本文件夹（例如：`syj_require_list_v0.1.0_backup`）
2. 解压新版本到新文件夹
3. 如果新版本有问题，可以回退到旧版本

## 安全提示

- 定期备份 `data` 文件夹
- 不要将程序放在系统关键目录（如 `C:\Windows\`）
- U 盘使用时注意安全弹出，避免数据损坏

## 许可证

便携版遵循项目原有许可证。
