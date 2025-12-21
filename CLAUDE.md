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
  ├── composables/      # Vue composables
  ├── stores/          # Pinia 状态管理
  └── types/           # TypeScript 类型
src-tauri/             # Rust 后端
  ├── src/
  │   ├── lib.rs       # 主逻辑和数据库初始化
  │   ├── commands.rs  # Tauri 命令 (CRUD API)
  │   └── window_snap.rs  # 窗口吸附逻辑
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
- **拖拽**: 不使用 `startDragging()` API，而是自定义拖动实现（mousedown/mousemove/mouseup）
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

### 5. 窗口吸附功能
- **自定义拖动**: 使用 mousedown/mousemove/mouseup 实现，60fps 节流
- **吸附触发**: 窗口边缘距离屏幕边缘 10px 内自动吸附
- **吸附位置**:
  - 左/右边缘：保持原宽度，高度等于屏幕高度
  - 顶部边缘：保持原尺寸
- **预览窗口**: 独立窗口（label: "preview"），拖动时显示吸附预览
- **自动隐藏**: 吸附后立即隐藏，保留 10px 可见区域
- **动画优化**: 150ms 动画时长，15fps 帧率，使用 easeInOutCubic 缓动
- **鼠标交互**:
  - 鼠标悬停在 10px 可见区域时显示窗口
  - 鼠标离开窗口时立即隐藏
- **防抖机制**: 吸附后 500ms 内不允许拖动，防止重复调用

### 6. 独立图片查看器窗口
- **窗口管理策略**: 使用预配置窗口（tauri.conf.json），而非动态创建
- **窗口生命周期**: 使用 `hide()` 而非 `close()` 关闭窗口，避免窗口被销毁
- **窗口通信**: 使用 Tauri 事件系统（`emit`/`listen`）传递图片数据
- **窗口尺寸**: 根据图片尺寸自动调整（最大屏幕的 80%）
- **图片交互**:
  - 鼠标滚轮缩放（25% - 300%），以鼠标位置为中心
  - 鼠标左键拖动平移图片
  - 禁用浏览器默认拖拽行为（`draggable="false"`）
- **Transform 顺序**: 使用 `translate scale` 而非 `scale translate`，确保缩放中心点正确
- **多图片支持**: 左右箭头切换，显示图片计数
- **快捷键**: ESC 关闭，方向键切换，+/- 缩放，0 重置
- **关键 API 注意事项**:
  - `Window.getByLabel()` 返回 `Promise<Window>`，必须 `await`
  - 从主窗口调用 `setFocus()` 需要权限，建议省略（`show()` 会自动获取焦点）

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

### src-tauri/src/window_snap.rs
```rust
// 窗口吸附功能的 Rust 实现:
// - get_current_screen_info: 获取当前显示器信息
// - determine_snap_edge: 判断应该吸附到哪个边缘
// - calculate_snap_position: 计算吸附后的位置和尺寸
// - snap_to_edge: 执行吸附操作（先设置位置，再设置尺寸）
// - check_should_unsnap: 检查是否应该取消吸附
```

### src/types/window.ts
```typescript
// 窗口吸附相关的 TypeScript 类型定义:
// - SnapEdge: 吸附边缘类型 ('left' | 'right' | 'top' | 'none')
// - ScreenInfo: 屏幕信息
// - WindowSize: 窗口尺寸
// - WindowPosition: 窗口位置
// - SnapConfig: 吸附配置
```

### src/stores/windowStore.ts
```typescript
// 窗口状态管理 (Pinia):
// - isSnapped: 是否处于吸附状态
// - snapEdge: 吸附的边缘
// - isHidden: 是否处于隐藏状态
// - snappedPosition: 吸附位置（用于从隐藏状态恢复）
// - originalSize/Position: 原始尺寸和位置（用于还原）
```

### src/composables/useWindowSnap.ts
```typescript
// 窗口吸附功能的前端实现:
// - getCurrentScreenInfo: 获取屏幕信息
// - determineSnapEdge: 判断吸附边缘
// - calculateSnapPosition: 计算吸附位置
// - snapToEdge: 执行吸附
// - restoreOriginalState: 还原窗口状态
// - hideWindow: 隐藏窗口（150ms/15fps 动画）
// - showWindow: 显示窗口（150ms/15fps 动画）
```

### src/components/SnapPreview.vue
```vue
// 吸附预览框组件:
// - 半透明蓝色背景
// - 显示吸附后的位置和尺寸
// - 拖动时自动显示/隐藏
```

### src/types/imageViewer.ts
```typescript
// 图片查看器相关的 TypeScript 类型定义:
// - ImageData: 图片数据（路径和 Blob URL）
// - ImageViewerData: 图片查看器数据（图片列表和当前索引）
```

### src/composables/useImageViewer.ts
```typescript
// 图片查看器窗口管理逻辑:
// - openImageViewer: 打开图片查看器窗口
// - 使用 Window.getByLabel('image-viewer') 获取预配置窗口
// - 通过事件系统发送图片数据到窗口
// 注意: Window.getByLabel() 返回 Promise，必须 await
```

### src/components/ImageViewer.vue
```vue
// 图片查看器组件:
// - 独立窗口显示，支持缩放、平移、多图切换
// - 接收 'image-viewer-data' 事件获取图片数据
// - 根据图片尺寸自动调整窗口大小并居中
// - 关闭时使用 hide() 而非 close()，保持窗口可复用
// - Transform 顺序: translate(x, y) scale(s)，确保缩放中心点正确
// - 禁用图片默认拖拽: draggable="false" + @dragstart.prevent
```

### src-tauri/tauri.conf.json
```json
// 配置文件，包含:
// - 窗口配置 (大小、置顶、无边框等)
//   * main: 主窗口
//   * preview: 吸附预览窗口 (visible: false)
//   * image-viewer: 图片查看器窗口 (visible: false)
// - 权限配置 (capabilities)
//   * dialog:allow-open - 文件选择对话框
//   * dialog:allow-confirm - 确认对话框
//   * fs:allow-read-file - 文件读取
//   * fs:scope - 文件访问范围限制
//   * core:event:* - 事件系统权限（窗口间通信）
//   * core:window:allow-set-size/center - 窗口尺寸和位置调整
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

### 5. 窗口吸附问题
- **吸附后窗口超出屏幕**: Rust 端先设置位置再设置尺寸，避免尺寸改变导致位置偏移
- **动画卡顿**: 降低帧率（15fps）和动画时长（150ms），使用 await 等待更新完成
- **隐藏后无法触发显示**: 增加可见区域到 10px，确保鼠标容易触发
- **重复隐藏导致闪烁**: 添加 `isHiding`/`isShowing` 标志防止重复调用
- **编译错误 "failed to remove file"**: 旧进程未退出，使用 `taskkill /F /IM syj_require_list.exe` 终止

### 6. 图片查看器窗口问题
- **`Window.getByLabel()` 返回 Promise**: 必须使用 `await`，否则得到的是 Promise 对象而非 Window 实例
  ```typescript
  // 错误
  const window = Window.getByLabel('image-viewer');
  await window.show(); // TypeError: window.show is not a function

  // 正确
  const window = await Window.getByLabel('image-viewer');
  await window.show(); // 正常工作
  ```
- **窗口找不到（返回 null）**: 窗口可能被 `close()` 销毁，需要重启应用。解决方案：使用 `hide()` 而非 `close()`
- **`setFocus()` 权限错误**: 从主窗口调用其他窗口的 `setFocus()` 需要权限，建议省略（`show()` 会自动获取焦点）
- **图片缩放中心点错误**: 确保 transform 顺序为 `translate scale`，而非 `scale translate`
- **图片拖动时被拖走**: 添加 `draggable="false"` 和 `@dragstart.prevent` 禁用浏览器默认拖拽行为

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

### 第三阶段：窗口吸附、预览、自动隐藏功能 ✅
**窗口吸附 MVP:**
- 拖动窗口到屏幕边缘（左/右/顶）自动吸附
- 吸附触发距离：10px
- 吸附后窗口尺寸：左/右边缘保持原宽度，高度等于屏幕高度；顶部边缘保持原尺寸
- 吸附后强制置顶
- 从吸附状态拖动到非边缘：只还原尺寸，保持当前位置
- 自定义拖动实现（mousedown/mousemove/mouseup），60fps 节流
- 防抖机制：吸附后 500ms 内不允许拖动

**拖动预览:**
- 拖动时显示半透明预览框（独立窗口）
- 预览框样式：半透明蓝色背景，带边框和阴影
- 预览框更新频率：100ms（10fps）
- 预览框显示吸附后的位置和尺寸

**自动隐藏:**
- 吸附成功后立即自动隐藏（无延迟）
- 隐藏时保留 10px 可见区域
- 隐藏动画：150ms，15fps，easeInOutCubic 缓动
- 鼠标悬停在 10px 可见区域时显示窗口
- 显示动画：150ms，15fps
- 鼠标离开窗口时立即隐藏（无延迟）
- 防止重复调用：添加 `isHiding` 和 `isShowing` 标志

### 第四阶段：独立图片查看器窗口功能 ✅
**窗口管理:**
- 独立窗口显示图片（label: "image-viewer"）
- 预配置窗口（tauri.conf.json），使用 `hide()`/`show()` 控制显示
- 根据图片尺寸自动调整窗口大小（最大屏幕的 80%）
- 窗口自动居中显示
- 使用 Tauri 事件系统传递图片数据

**图片交互:**
- 鼠标滚轮缩放（25% - 300%）
- 以鼠标位置为中心缩放（鼠标指向的点保持不动）
- 鼠标左键拖动平移图片
- 禁用浏览器默认图片拖拽行为
- 实时显示缩放百分比

**多图片支持:**
- 左右箭头按钮切换图片
- 显示图片计数（如 "2/5"）
- 切换图片时自动重置缩放和平移

**快捷键:**
- ESC: 关闭窗口
- 方向键左/右: 切换图片
- +/-: 放大/缩小
- 0: 重置缩放和平移

**控制按钮:**
- 关闭按钮
- 置顶/取消置顶按钮
- 重置缩放按钮
- 缩放百分比显示

## 下一步计划

1. **OCR 功能**: 图片文字识别
2. **快捷键支持**: 全局快捷键唤醒窗口
3. **主题切换**: 深色/浅色主题

详细计划请参考 `document/项目规划.md`
