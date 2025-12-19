# Claude 开发指南

本文档为 AI 助手提供项目开发的关键信息和最佳实践。

## 项目概览

这是一个基于 Tauri 2.x + Vue 3 + TypeScript 的桌面悬浮待办事项工具。

### 技术栈
- **前端**: Vue 3 + TypeScript + Vite + Pinia
- **桌面**: Tauri 2.x (Rust + WebView2)
- **数据库**: SQLite (通过 sqlx crate)
- **异步运行时**: Tokio

### 项目结构
```
src/                    # Vue 前端
  ├── components/       # UI 组件
  ├── stores/          # Pinia 状态管理
  └── types/           # TypeScript 类型
src-tauri/             # Rust 后端
  ├── src/
  │   ├── lib.rs       # 主逻辑和数据库初始化
  │   └── commands.rs  # Tauri 命令 (CRUD API)
  ├── Cargo.toml       # Rust 依赖
  └── tauri.conf.json  # Tauri 配置
```

## 关键技术决策

### 1. 数据库实现
- **不使用** `tauri-plugin-sql` (API 过于复杂)
- **直接使用** `sqlx` crate 操作 SQLite
- 数据库位置: `C:\Users\<用户>\AppData\Local\syj_require_list\todos.db`
- 连接池管理: `Arc<Pool<Sqlite>>` 通过 `tauri::State` 共享

### 2. Tauri 2.x API 变化
- **窗口获取**: 使用 `getCurrentWindow()` 而非直接导入
- **拖拽**: 使用 `startDragging()` API 而非 HTML 属性
- **权限**: 需要在 `tauri.conf.json` 的 `capabilities` 中显式声明

### 3. 前后端通信
- 前端通过 `invoke()` 调用 Rust 命令
- Rust 命令需要用 `#[tauri::command]` 标注
- 所有命令都在 `src-tauri/src/commands.rs` 中定义
- **数据序列化**: 使用 `#[serde(rename_all = "camelCase")]` 实现字段名自动转换（Rust 下划线命名 ↔ JS 驼峰命名）

### 4. 图片附件功能
- **存储位置**: `C:\Users\<用户>\AppData\Local\syj_require_list\images\`
- **上传方式**:
  - 点击按钮选择文件 (`@tauri-apps/plugin-dialog`)
  - Ctrl+V 粘贴剪贴板图片 (监听 `paste` 事件)
  - 拖拽文件到输入区域 (监听 `drop` 事件)
- **显示方式**:
  - 前端调用 `read_image` 命令获取图片字节数组
  - 转换为 Blob URL 显示（避免 `convertFileSrc` 的权限问题）
- **清理机制**: 删除待办时自动删除关联的图片文件

## 重要文件说明

### src-tauri/src/lib.rs
```rust
// 主逻辑文件，包含:
// 1. 数据库初始化 (init_database 函数)
// 2. Tauri 应用启动配置
// 3. 状态管理 (DbState)
```

### src-tauri/src/commands.rs
```rust
// 定义所有 Tauri 命令:
// - get_todos: 获取所有待办
// - add_todo: 添加新待办
// - update_todo: 更新待办
// - delete_todo: 删除待办 (自动删除关联的图片文件)
// - toggle_todo: 切换完成状态
// - save_image: 保存图片到本地目录
// - read_image: 读取图片文件返回字节数组
```

### src-tauri/tauri.conf.json
```json
// 配置文件，包含:
// - 窗口配置 (大小、置顶、无边框等)
// - 权限配置 (capabilities)
//   * dialog:allow-open - 文件选择对话框
//   * dialog:allow-confirm - 确认对话框
//   * fs:allow-read-file - 文件读取
//   * fs:scope - 文件访问范围限制
```

## 开发常用命令

### 启动开发服务器
```bash
npm run tauri dev
```

### 内存不足时
```powershell
$env:CARGO_BUILD_JOBS=1
npm run tauri dev
```

### 打包发布
```bash
npm run tauri build
```

### 清理编译缓存
```bash
cd src-tauri
cargo clean
```

## 常见问题和解决方案

### 1. 编译错误
- **类型推断失败**: 需要显式指定泛型参数，如 `map_err(|e: sqlx::Error| ...)`
- **生命周期错误**: 使用 `Arc` 和 `State` 管理共享状态
- **权限错误**: 在 `tauri.conf.json` 添加对应权限

### 2. 数据库问题
- **无法打开数据库**: 确保路径存在，使用 `dirs::data_local_dir()` 获取正确路径
- **SQL 语法错误**: SQLite 使用 `INTEGER` 而非 `INT`，布尔值用 0/1

### 3. 前端问题
- **invoke 调用失败**: 检查命令名称是否正确，参数是否匹配
- **状态不更新**: 确保使用 Pinia store 管理状态
- **窗口API不工作**: 确认权限配置是否正确

### 4. 图片显示问题
- **缩略图不显示**: 检查字段名映射是否正确（`imagePath` vs `image_path`）
- **convertFileSrc 失败**: 使用后端 `read_image` 命令读取图片，转换为 Blob URL
- **图片加载慢**: 考虑添加缓存机制或懒加载

## 代码规范

### Rust 代码
- 使用 `async/await` 处理异步操作
- 错误处理统一使用 `Result<T, String>`
- 数据库操作使用参数绑定防止 SQL 注入
- 命令函数保持简洁，复杂逻辑抽取到单独函数

### TypeScript 代码
- 使用接口定义数据结构
- Pinia store 使用 Composition API 风格
- 组件使用 `<script setup>` 语法
- 错误处理要有用户友好的提示

## 开发流程

### 添加新功能的步骤

1. **定义数据结构** (如果需要)
   - TypeScript: `src/types/*.ts`
   - Rust: `src-tauri/src/commands.rs` 中的 struct

2. **实现 Rust 后端**
   - 在 `commands.rs` 添加新命令
   - 在 `lib.rs` 的 `invoke_handler!` 中注册

3. **实现前端逻辑**
   - 在 Pinia store 中添加action
   - 创建或修改 Vue 组件

4. **添加权限** (如果需要)
   - 在 `tauri.conf.json` 的 `capabilities` 中添加

5. **测试**
   - 启动开发服务器测试
   - 检查控制台是否有错误

## 性能优化建议

### 编译优化
- 使用 `CARGO_BUILD_JOBS=1` 限制并行任务(内存不足时)
- 开启增量编译(默认已开启)

### 运行时优化
- 数据库连接池限制为 5 个连接
- 避免频繁的数据库查询
- 大列表使用虚拟滚动(未来实现)

## 调试技巧

### Rust 调试
```rust
// 打印调试信息
println!("Debug: {:?}", value);

// 开启 backtrace
$env:RUST_BACKTRACE=1
```

### 前端调试
```typescript
// 浏览器开发者工具
// 按 F12 或右键 -> 检查

// Console 输出
console.log('Debug:', data);
```

## 数据库结构

```sql
CREATE TABLE todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    source TEXT,
    image_path TEXT,
    ocr_text TEXT,
    created_at TEXT NOT NULL,
    expected_finish_at TEXT,
    finished_at TEXT,
    is_completed INTEGER DEFAULT 0,
    order_index INTEGER
);
```

## 环境配置

### Rust 环境
- 安装位置: `E:\DevTools\Rust`
- 环境变量: `RUSTUP_HOME`, `CARGO_HOME`

### Node.js 环境
- 安装位置: `J:\software\nodejs_new_nvm\nodejs`
- 包管理器: npm

### 项目路径
- 工作目录: `E:\IntelliJ_workspace\syj_require_list_project`

## 参考文档

- [Tauri 官方文档](https://tauri.app/)
- [Vue 3 官方文档](https://vuejs.org/)
- [sqlx 文档](https://github.com/launchbadge/sqlx)
- [Pinia 文档](https://pinia.vuejs.org/)

## 开发日志

详细的开发过程记录在 `document/开发日志-*.md` 文件中，包括:
- 环境搭建过程
- 遇到的问题和解决方案
- 功能实现记录

## 已完成功能

### 第一阶段：基础功能 ✅
- 待办事项的 CRUD 操作
- 完成状态切换
- 悬浮窗口（置顶、无边框）
- 数据持久化（SQLite）

### 第二阶段：图片附件功能 ✅
- 图片上传（点击按钮、Ctrl+V 粘贴、拖拽文件）
- 图片预览（缩略图和大图查看）
- 图片存储管理（自动保存、删除清理）
- 支持多种图片格式（PNG, JPG, JPEG, GIF, BMP, WEBP）

## 下一步计划

1. **OCR 功能**: 图片文字识别
2. **边缘吸附**: 窗口靠近屏幕边缘时自动吸附
3. **侧边隐藏**: 窗口在屏幕边缘时自动隐藏，鼠标悬停显示
4. **快捷键支持**: 全局快捷键唤醒窗口
5. **主题切换**: 深色/浅色主题

详细计划请参考 `document/项目规划.md`
