# DataVis Pro

<div align="center">

**基于 Tauri + Vue 3 + Rust + Polars 构建的高性能数据分析工具**

[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue.svg)](https://tauri.app/)
[![Vue](https://img.shields.io/badge/Vue-3.5-green.svg)](https://vuejs.org/)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

</div>

---

## 📖 项目简介

DataVis Pro `/ˈdeɪtə vɪz proʊ/` 是一款轻量级、高性能的数据分析桌面应用程序，专为数据分析师和数据科学家设计。它结合了 Rust
的高性能数据处理能力和
Vue 3 的现代化前端体验，提供流畅的数据导入、清洗、转换和可视化功能。

### ✨ 核心特性

- 🚀 **高性能数据处理**：基于 Polars 引擎，处理速度比 Pandas 快数倍
- 💾 **多格式支持**：支持 CSV、Excel (xlsx/xls)、Parquet 文件的导入和导出
- ⏮️ **完整的 Undo/Redo**：基于历史栈架构，支持无限次撤销和重做
- 🔍 **SQL 筛选**：使用标准 SQL WHERE 语法进行数据筛选
- 🧹 **数据清洗**：空值处理、列选择、类型转换等常用操作
- 📊 **大数据表格**：基于 ag-Grid 的高性能表格，支持百万行数据
- 🎨 **现代化 UI**：基于 Element Plus 的美观界面
- 🔒 **内存安全**：Rust 后端保证内存安全和线程安全

---

## 🛠️ 技术栈

### 前端

- **框架**：Vue 3 (Composition API)
- **语言**：TypeScript
- **UI 库**：Element Plus
- **表格**：ag-Grid Community
- **状态管理**：Pinia
- **构建工具**：Vite
- **包管理器**：Bun

### 后端

- **框架**：Tauri 2.0
- **语言**：Rust (2024 edition)
- **数据处理**：Polars (支持 SQL、Lazy Evaluation)
- **Excel 解析**：Calamine
- **序列化**：Serde + Serde JSON

### 架构特点

- **前后端分离**：通过 IPC (进程间通信) 交互
- **历史栈架构**：每次操作存储完整状态，支持 Undo/Redo
- **线性历史记录**：回退后新操作会丢弃后续历史
- **内存管理**：最大历史深度限制（默认 50 条）

---

## 🚀 快速开始

### 环境要求

- **Node.js**: >= 20.0.0
- **Bun**: >= 1.0.0 (推荐) 或 npm/yarn/pnpm
- **Rust**: >= 1.70.0
- **操作系统**: macOS / Windows / Linux

### 安装依赖

```bash
# 克隆项目
git clone <repository-url>
cd my-data-analyst

# 安装前端依赖
bun install

# 安装 Rust 依赖（自动）
cd src-tauri
cargo build
```

### 开发模式

```bash
# 启动开发服务器（前端 + Tauri）
bun run tauri dev

# 仅启动前端开发服务器
bun run dev
```

### 构建应用

```bash
# 构建前端
bun run build

# 构建 Tauri 应用（生成可执行文件）
bun run tauri build
```

构建完成后，可执行文件位于 `src-tauri/target/release/bundle/` 目录。

---

## 📚 功能说明

### 1. 文件导入

支持四种文件格式：

- **CSV**：逗号分隔值文件
- **TSV**：制表符分隔值文件
- **Excel**：.xlsx 和 .xls 格式（自动检测工作表）
- **Parquet**：高效的列式存储格式

**操作步骤**：

1. 点击顶部"导入数据"按钮
2. 选择文件类型过滤器（可选：所有格式、CSV、TSV、Excel、Parquet）
3. 选择文件并确认
4. 数据自动加载到表格中

### 2. 数据操作

#### 列操作

- **选择列**：保留选中的列，删除其他列
- **删除列**：删除选中的列
- **重命名列**：批量修改列名
- **类型转换**：转换列的数据类型（Int64, Float64, String, Boolean, Date, Datetime）

#### 行操作

- **删除空值行**：删除包含空值的行（可选择特定列）
- **删除全空行**：删除所有列都为空的行
- **SQL 筛选**：使用 SQL WHERE 语法筛选数据
    - 示例：`age > 18 AND status = 'active'`
    - 支持：`>`, `<`, `>=`, `<=`, `=`, `!=`, `AND`, `OR`, `IS NULL`, `LIKE`, `IN`

#### 空值填充

支持 7 种填充策略：

- **向前填充 (Forward)**：使用前一个非空值
- **向后填充 (Backward)**：使用后一个非空值
- **最小值填充 (Min)**：使用列的最小值
- **最大值填充 (Max)**：使用列的最大值
- **均值填充 (Mean)**：使用列的平均值
- **零值填充 (Zero)**：使用 0
- **1 值填充 (One)**：使用 1

### 3. 历史管理

- **撤销 (Undo)**：快捷键 `Ctrl+Z` (Windows/Linux) 或 `Cmd+Z` (macOS)
- **重做 (Redo)**：快捷键 `Ctrl+Shift+Z` 或 `Ctrl+Y` (Windows/Linux)，`Cmd+Shift+Z` (macOS)
- **重置数据**：恢复到刚导入时的状态（清除所有操作历史）
- **清空数据**：删除所有数据和历史

### 4. 数据导出

支持两种导出格式：

- **CSV**：通用表格格式，兼容性好，可用 Excel/WPS 直接打开
- **Parquet**：列式存储，压缩率高，适合大数据处理，需要工具解析

**操作步骤**：

1. 点击顶部"导出数据"按钮
2. 选择导出格式（CSV 或 Parquet）
3. 选择保存位置并确认

---

## 🏗️ 项目结构

```
my-data-analyst/
├── src/                          # 前端代码（Vue 3）
│   ├── App.vue                   # 主应用组件
│   ├── main.ts                   # 前端入口
│   ├── components/               # Vue 组件
│   │   ├── Sidebar.vue           # 左侧操作面板
│   │   ├── DataGrid.vue          # 数据表格
│   │   ├── DataInfoBar.vue       # 数据信息栏
│   │   ├── RightSidebar.vue      # 右侧统计面板
│   │   └── dialogs/              # 对话框组件
│   │       ├── ExportDialog.vue
│   │       ├── FillNullDialog.vue
│   │       ├── FilterDialog.vue
│   │       ├── SelectColumnsDialog.vue
│   │       ├── RenameColumnsDialog.vue
│   │       └── CastTypesDialog.vue
│   ├── stores/                   # Pinia 状态管理
│   │   └── dataStore.ts          # 数据状态
│   ├── types/                    # TypeScript 类型定义
│   │   ├── dataset.ts            # 数据集类型
│   │   └── history.ts            # 历史类型
│   └── utils/                    # 工具函数
│       └── tauri-commands.ts     # Tauri 命令封装
│
├── src-tauri/                    # 后端代码（Rust）
│   ├── src/
│   │   ├── main.rs               # Rust 入口
│   │   ├── lib.rs                # Tauri 应用核心
│   │   ├── error.rs              # 错误类型定义
│   │   ├── models/               # 数据模型
│   │   │   ├── dataset.rs        # DatasetInfo, ColumnInfo, DatasetData
│   │   │   └── history.rs        # HistoryEntry, OperationType
│   │   ├── data/                 # 数据处理
│   │   │   ├── store.rs          # 历史栈数据存储
│   │   │   └── loader.rs         # 文件加载器
│   │   └── commands/             # Tauri 命令
│   │       ├── file_import.rs    # 文件导入
│   │       ├── data_query.rs     # 数据查询
│   │       ├── data_export.rs    # 数据导出
│   │       ├── history.rs        # 历史管理
│   │       └── operations.rs     # 数据操作
│   ├── Cargo.toml                # Rust 依赖配置
│   └── tauri.conf.json           # Tauri 应用配置
│
├── TAURI架构说明.md               # 架构详细说明
├── CLAUDE.md                     # AI 协作指南
├── 开发进度.md                    # 开发进度跟踪
└── README.md                     # 本文件
```

---

## 🔧 开发指南

### 添加新的数据操作

1. **定义操作类型**（`src-tauri/src/models/history.rs`）：
   ```rust
   pub enum OperationType {
       // ... 现有操作
       YourNewOperation { params: YourParams },
   }
   ```

2. **实现 Rust 命令**（`src-tauri/src/commands/operations.rs`）：
   ```rust
   #[tauri::command]
   pub async fn your_new_operation(
       params: YourParams,
       state: State<'_, AppState>,
   ) -> Result<(), String> {
       // 实现逻辑
   }
   ```

3. **注册命令**（`src-tauri/src/lib.rs`）：
   ```rust
   .invoke_handler(tauri::generate_handler![
       // ... 现有命令
       your_new_operation,
   ])
   ```

4. **封装前端调用**（`src/utils/tauri-commands.ts`）：
   ```typescript
   export async function yourNewOperation(params: YourParams): Promise<void> {
       return await invoke('your_new_operation', { params });
   }
   ```

5. **在 Store 中添加方法**（`src/stores/dataStore.ts`）：
   ```typescript
   async function yourNewOperation(params: YourParams) {
       loading.value = true;
       try {
           await commands.yourNewOperation(params);
           await loadHistory();
           await loadCurrentData(0, 100);
       } finally {
           loading.value = false;
       }
   }
   ```

6. **创建 UI 组件**（`src/components/dialogs/YourDialog.vue`）

### 代码规范

- **TypeScript**：使用严格模式，启用 `noUnusedLocals` 和 `noUnusedParameters`
- **Vue**：使用 `<script setup>` 语法和 Composition API
- **Rust**：使用 `cargo fmt` 格式化，`cargo clippy` 检查
- **提交**：使用语义化提交信息（feat/fix/docs/refactor/test）

### 常用命令

```bash
# 类型检查
bun run vue-tsc --noEmit

# 代码检查和格式化
bun run check

# Rust 代码检查
cd src-tauri
cargo clippy
cargo fmt

# 运行测试
cargo test
```

---

## 📊 性能特点

- **数据处理速度**：Polars 比 Pandas 快 5-10 倍
- **内存占用**：Rust 后端内存占用低，无 GC 开销
- **应用体积**：打包后约 5-10 MB（比 Electron 小 10 倍）
- **启动速度**：冷启动 < 1 秒
- **表格渲染**：ag-Grid 虚拟滚动，支持百万行数据

---

## 🤝 贡献指南

欢迎贡献代码、报告问题或提出建议！

1. Fork 本项目
2. 创建特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交更改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 开启 Pull Request

---

## 📝 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件

---

## 🙏 致谢

- [Tauri](https://tauri.app/) - 轻量级桌面应用框架
- [Polars](https://www.pola.rs/) - 高性能数据处理引擎
- [Vue.js](https://vuejs.org/) - 渐进式 JavaScript 框架
- [Element Plus](https://element-plus.org/) - Vue 3 UI 组件库
- [ag-Grid](https://www.ag-grid.com/) - 企业级数据表格

---

<div align="center">

**⭐ 如果这个项目对你有帮助，请给一个 Star！**

Made with ❤️ using Tauri + Vue 3 + Rust

</div>