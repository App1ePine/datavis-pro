# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

这是一个使用 Tauri + Vue 3 + TypeScript 构建的**数据分析桌面应用程序**。

**项目名称**：DataVis Pro `/ˈdeɪtə vɪz proʊ/`

支持 CSV/TSV/Excel/Parquet 文件导入、数据处理、表格展示和可视化分析。

### 技术栈

**前端：**

- Vue 3 + TypeScript + Vite
- Element Plus（UI 组件库）
- ag-Grid（高性能数据表格）
- Pinia（状态管理）
- @vueuse/core（组合式工具库）

**后端：**

- Rust (Tauri 2)
- Polars（数据处理引擎，支持 SQL）
- Calamine（Excel 文件解析）

**核心特性：**

- 历史栈架构（Undo/Redo）
- CSV 分隔符自动检测（逗号、制表符、分号、竖线）
- SQL 筛选功能
- 7 种空值填充策略

**数据流架构：**

```
用户选择文件 → Tauri Dialog
    ↓
Rust 后端（Polars）解析 CSV/Excel/Parquet
    ↓ (CSV 自动检测分隔符：, \t ; |)
存储到历史栈 DataStore（Vec<HistoryEntry>）
    ↓
前端通过 invoke() 查询数据（分页）
    ↓
ag-Grid 展示数据（虚拟滚动）
```

## Development Commands

### 包管理器

项目使用 **Bun** 作为包管理器（见 `tauri.conf.json` 中的 `beforeDevCommand` 和 `beforeBuildCommand`）。

### 常用命令

```bash
# 安装依赖
bun install

# 启动开发服务器（前端 + Tauri）
bun run tauri dev

# 仅启动前端开发服务器
bun run dev

# 类型检查
vue-tsc --noEmit

# 构建前端
bun run build

# 构建 Tauri 应用
bun run tauri build

# 预览构建结果
bun run preview
```

### Rust 相关命令

```bash
# 在 src-tauri 目录下
cd src-tauri

# 检查 Rust 代码
cargo check

# 运行 Rust 测试
cargo test

# 格式化 Rust 代码
cargo fmt

# Rust 代码检查
cargo clippy
```

## Architecture

### 前后端通信架构

这是一个典型的 Tauri 应用架构，前端和后端通过 IPC（进程间通信）交互：

- **前端（Vue 3）**: 运行在 WebView 中，使用 `@tauri-apps/api` 与后端通信
- **后端（Rust）**: 提供原生功能和系统级 API，通过 Tauri commands 暴露给前端

**通信流程**:

1. 前端通过 `invoke()` 调用 Rust 命令（见 `src/App.vue:10`）
2. Rust 端使用 `#[tauri::command]` 宏定义可调用的函数（见 `src-tauri/src/lib.rs:2-5`）
3. 命令需要在 `invoke_handler` 中注册（见 `src-tauri/src/lib.rs:11`）

### 目录结构

```
src/                    # Vue 前端代码
  ├── App.vue          # 主应用组件（布局：侧边栏 + 内容区）
  ├── main.ts          # Vue 应用入口（配置 Pinia 和 Element Plus）
  ├── components/      # Vue 组件
  │   ├── FileImport.vue   # 文件导入组件
  │   └── DataGrid.vue     # ag-Grid 表格组件
  ├── stores/          # Pinia 状态管理
  │   └── dataStore.ts     # 数据集状态管理
  ├── types/           # TypeScript 类型定义
  │   └── dataset.ts       # 数据集相关类型
  ├── utils/           # 工具函数
  │   └── tauri-commands.ts # Tauri 命令封装
  ├── assets/          # 静态资源
  └── vite-env.d.ts    # Vite 类型声明

src-tauri/             # Rust 后端代码
  ├── src/
  │   ├── main.rs      # Rust 应用入口（调用 lib.rs 的 run()）
  │   ├── lib.rs       # Tauri 应用逻辑、AppState 和命令注册
  │   ├── error.rs     # 自定义错误类型
  │   ├── models/      # 数据模型
  │   │   ├── mod.rs
  │   │   └── dataset.rs   # DatasetInfo, ColumnInfo, DatasetData
  │   ├── data/        # 数据处理模块
  │   │   ├── mod.rs
  │   │   ├── store.rs     # 内存数据存储（DataStore）
  │   │   └── loader.rs    # CSV/Excel 加载器
  │   └── commands/    # Tauri 命令
  │       ├── mod.rs
  │       ├── file_import.rs   # import_csv, import_excel
  │       └── data_query.rs    # get_dataset_list, get_dataset_info, get_dataset_data
  ├── Cargo.toml       # Rust 依赖配置
  └── tauri.conf.json  # Tauri 应用配置

public/                # 公共静态资源
dist/                  # 前端构建输出（由 Vite 生成）
```

### 关键配置文件

- **tauri.conf.json**: Tauri 应用配置，包括窗口大小、开发/构建命令、应用标识符等
- **vite.config.ts**: Vite 配置，针对 Tauri 开发进行了优化（固定端口 1420，忽略 src-tauri 目录监听）
- **tsconfig.json**: TypeScript 配置，使用严格模式和 bundler 模块解析

### 添加新的 Tauri Command

1. 在 `src-tauri/src/commands/` 中定义命令函数（使用 `#[tauri::command]` 宏）
2. 在 `commands/mod.rs` 中导出命令
3. 在 `lib.rs` 的 `invoke_handler` 中注册命令
4. 在前端 `utils/tauri-commands.ts` 中封装调用函数

### Rust 模块架构

**AppState（应用状态）：**

- `DataStore`：内存中的数据存储，使用 `Arc<Mutex<HashMap<String, DataFrame>>>` 管理多个数据集
- 每个数据集有唯一 ID（UUID）和元数据（DatasetInfo）

**数据模型（models/dataset.rs）：**

- `DatasetInfo`：数据集元信息（ID、名称、行数、列信息、导入时间）
- `ColumnInfo`：列信息（列名、数据类型、空值数量）
- `DatasetData`：分页数据（列名、数据行、总行数）

**数据处理（data/）：**

- `loader.rs`：使用 Polars 读取 CSV，使用 Calamine 读取 Excel
- `store.rs`：DataStore 实现，提供 insert/get/list 方法

**命令（commands/）：**

- `file_import.rs`：`import_csv()`, `import_excel()` - 导入文件并返回 DatasetInfo
- `data_query.rs`：`get_dataset_list()`, `get_dataset_info()`, `get_dataset_data()` - 查询数据

### Pinia Store 设计

**dataStore（stores/dataStore.ts）：**

状态：

- `datasets: Map<string, DatasetInfo>` - 所有数据集的元信息
- `currentDatasetId: string | null` - 当前选中的数据集 ID
- `currentData: DatasetData | null` - 当前显示的数据（分页）
- `loading: boolean` - 加载状态
- `error: string | null` - 错误信息

操作：

- `importCSV(filePath)` - 导入 CSV 文件
- `importExcel(filePath, sheetName?)` - 导入 Excel 文件
- `loadDatasetData(datasetId, offset, limit)` - 加载数据集的分页数据
- `selectDataset(datasetId)` - 切换当前数据集

### 插件系统

项目已集成以下 Tauri 插件：

- `tauri-plugin-opener`：在默认浏览器中打开 URL
- `tauri-plugin-dialog`：文件选择对话框（用于导入 CSV/Excel）

添加新插件时需要：

1. 在 `src-tauri/Cargo.toml` 中添加依赖
2. 在 `lib.rs` 的 `Builder` 中使用 `.plugin()` 注册

### 关键依赖说明

**Rust 依赖：**

- `polars`：高性能数据处理引擎，支持 lazy evaluation 和 streaming
    - 启用特性：`lazy`, `csv`, `dtype-full`, `streaming`, `strings`, `temporal`
- `calamine`：纯 Rust Excel 解析库，支持 .xlsx 和 .xls 格式
- `thiserror` + `anyhow`：错误处理
- `tokio`：异步运行时（用于文件 I/O）

**前端依赖：**

- `element-plus`：Vue 3 UI 组件库
- `ag-grid-vue3` + `ag-grid-community`：企业级数据表格
- `pinia`：Vue 3 官方状态管理库
- `@tauri-apps/plugin-dialog`：文件选择对话框

## Development Notes

- Vite 开发服务器运行在固定端口 **1420**，HMR 端口为 **1421**
- TypeScript 使用严格模式，启用了 `noUnusedLocals` 和 `noUnusedParameters`
- Vue 组件使用 `<script setup>` 语法和 Composition API
- Rust 代码使用 2021 edition，库名为 `my_data_analyst_lib`
