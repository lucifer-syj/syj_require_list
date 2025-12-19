# 桌面悬浮待办事项工具

一个轻量级的桌面悬浮 TodoList 工具，支持窗口置顶、数据持久化存储。

## 功能特性

### 已实现功能 ✅
- **悬浮窗口**：320x600 无边框窗口，可自由拖拽
- **置顶控制**：通过图钉按钮控制窗口是否始终置顶
- **待办管理**：
  - 添加新待办事项
  - 标记完成/未完成
  - 删除待办
  - 完成的待办自动变灰并移到底部
- **数据持久化**：使用 SQLite 本地存储，数据永久保存

### 计划中功能 ⏳
- **图片支持**：上传图片，OCR 文字识别
- **边缘吸附**：窗口拖到屏幕边缘自动吸附
- **侧边隐藏**：吸附后自动隐藏，鼠标悬停展开（类似 QQ）
- **快捷键**：快速添加、切换等操作

## 技术栈

### 前端
- **Vue 3** - 渐进式 JavaScript 框架
- **TypeScript** - 类型安全
- **Vite** - 极速构建工具
- **Pinia** - Vue 3 状态管理

### 桌面端
- **Tauri 2.x** - Rust + WebView2
- **优势**：
  - 体积小（~15MB）
  - 内存占用低（~50MB）
  - 启动快速
  - 安全可靠

### 数据库
- **SQLite** - 通过 sqlx crate 操作
- **存储位置**：`C:\Users\用户名\AppData\Local\syj_require_list\todos.db`

## 快速开始

### 环境要求
- Windows 10/11
- Node.js 16+
- Rust 1.70+
- Visual Studio Build Tools（MSVC）

### 开发环境搭建

1. **安装 Rust**
```powershell
# 下载并运行 rustup-init.exe
# 或自定义安装位置
$env:RUSTUP_HOME = "E:\DevTools\Rust\rustup"
$env:CARGO_HOME = "E:\DevTools\Rust\cargo"
```

2. **安装 Visual Studio Build Tools**
- 下载地址：https://visualstudio.microsoft.com/zh-hans/downloads/
- 选择："使用 C++ 的桌面开发"

3. **克隆项目**
```bash
git clone <repository-url>
cd syj_require_list_project
```

4. **安装依赖**
```bash
npm install
```

5. **启动开发服务器**
```bash
npm run tauri dev
```

### 打包发布
```bash
npm run tauri build
```

生成的 exe 文件位于：`src-tauri/target/release/`

## 项目结构

```
syj_require_list_project/
├── src/                      # Vue 前端源码
│   ├── components/          # UI 组件
│   │   ├── TodoList.vue    # 待办列表
│   │   ├── TodoItem.vue    # 单个待办项
│   │   └── AddTodoForm.vue # 添加表单
│   ├── stores/             # Pinia 状态管理
│   │   └── todoStore.ts    # Todo store
│   ├── types/              # TypeScript 类型定义
│   │   └── todo.ts         # Todo 类型
│   ├── App.vue             # 根组件
│   └── main.ts             # 入口文件
├── src-tauri/               # Rust 后端源码
│   ├── src/
│   │   ├── lib.rs          # 主逻辑和数据库初始化
│   │   ├── commands.rs     # Tauri 命令（CRUD API）
│   │   └── database.rs     # 数据库模块（已弃用）
│   ├── Cargo.toml          # Rust 依赖
│   └── tauri.conf.json     # Tauri 配置
├── document/                # 项目文档
│   ├── 开发日志-*.md       # 开发日志
│   └── 项目规划.md         # 项目规划
└── README.md               # 本文件
```

## 数据库结构

```sql
CREATE TABLE todos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,              -- 待办内容
    source TEXT,                        -- 来源（手动/截图）
    image_path TEXT,                    -- 图片路径
    ocr_text TEXT,                      -- OCR 识别文字
    created_at TEXT NOT NULL,           -- 创建时间
    expected_finish_at TEXT,            -- 预期完成时间
    finished_at TEXT,                   -- 实际完成时间
    is_completed INTEGER DEFAULT 0,     -- 是否完成（0/1）
    order_index INTEGER                 -- 排序索引
);
```

## 常见问题

### 1. 编译内存不足
```powershell
# 限制并行编译任务数
$env:CARGO_BUILD_JOBS=1
npm run tauri dev
```

### 2. 权限错误
如果遇到权限相关错误，检查 `src-tauri/tauri.conf.json` 中的 `capabilities` 配置。

### 3. 数据库位置
数据库文件存储在：
- Windows: `C:\Users\<用户名>\AppData\Local\syj_require_list\todos.db`

## 开发计划

- [x] **第一阶段**：TodoList 核心功能（已完成）
- [ ] **第二阶段**：图片上传和 OCR 识别
- [ ] **第三阶段**：边缘吸附和侧边隐藏

详细计划请查看 `document/项目规划.md`

## 开发文档

- **开发日志**：`document/开发日志-*.md`
- **项目规划**：`document/项目规划.md`
- **开发指南**：`CLAUDE.md`

## License

MIT

## 致谢

- [Tauri](https://tauri.app/) - 桌面应用框架
- [Vue.js](https://vuejs.org/) - 前端框架
- [sqlx](https://github.com/launchbadge/sqlx) - Rust SQL 工具包
