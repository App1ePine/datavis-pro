// ============================================================================
// commands/mod.rs - 命令模块导出
// ============================================================================
// 这个文件是 commands 模块的入口文件
// 它的作用是：
// 1. 声明子模块（data_export, data_query, file_import）
// 2. 重新导出子模块中的公共函数，方便外部使用
//
// 这样做的好处：
// - 外部代码可以直接使用 `use crate::commands::import_csv`
// - 而不需要写 `use crate::commands::file_import::import_csv`
// - 简化了导入路径，提高了代码可读性
// ============================================================================

// ============================================================================
// 子模块声明
// ============================================================================
// pub mod: 声明一个公共子模块
// Rust 会自动查找对应的文件：
// - pub mod data_export → 查找 commands/data_export.rs
// - pub mod data_query → 查找 commands/data_query.rs
// - pub mod file_import → 查找 commands/file_import.rs

/// 数据导出和清空命令模块
/// 包含：export_csv, clear_dataset, clear_all_datasets
pub mod data_export;

/// 数据查询命令模块
/// 包含：get_dataset_list, get_dataset_info, get_dataset_data
pub mod data_query;

/// 文件导入命令模块
/// 包含：import_csv, import_excel
pub mod file_import;

/// 历史管理命令模块
/// 包含：get_history, undo_operation, redo_operation, jump_to_history, can_undo, can_redo
pub mod history;

/// 数据操作命令模块
/// 包含：unpivot_data, drop_nulls, drop_all_nulls, select_columns, drop_columns,
///       rename_columns, cast_types, fill_null, rolling_average, rolling_median
pub mod operations;

// ============================================================================
// 重新导出（Re-exports）
// ============================================================================
// pub use: 将子模块中的项重新导出到当前模块
// 这样外部代码就可以直接从 commands 模块导入，而不需要知道具体在哪个子模块中
//
// 例如：
// - 有了 pub use，可以写：use crate::commands::export_csv;
// - 没有 pub use，需要写：use crate::commands::data_export::export_csv;

/// 这些命令用于将数据导出为 CSV/Parquet 文件，以及清空数据
pub use data_export::{export_csv, export_parquet, clear_data};

/// 这些命令用于查询数据集列表、元信息和实际数据
pub use data_query::{get_current_data, get_current_info, get_column_stats};

/// 这些命令用于导入 CSV、Excel 和 Parquet 文件
pub use file_import::{import_csv, import_excel, import_parquet};

/// 这些命令用于管理操作历史（undo/redo）
pub use history::{get_history, get_current_index, undo_operation, redo_operation, jump_to_history, can_undo, can_redo, reset_to_initial};

/// 这些命令用于数据操作（Phase 1: 基础操作）
pub use operations::{
    drop_nulls, drop_all_nulls, select_columns, drop_columns,
    rename_columns, cast_types, filter_data, fill_null
};

// ============================================================================
// 使用示例
// ============================================================================
//
// 在 lib.rs 中注册命令时，可以直接使用：
//
// ```rust
// use crate::commands::*;  // 导入所有命令
//
// tauri::Builder::default()
//     .invoke_handler(tauri::generate_handler![
//         import_csv,           // 来自 file_import 模块
//         import_excel,         // 来自 file_import 模块
//         get_dataset_list,     // 来自 data_query 模块
//         get_dataset_info,     // 来自 data_query 模块
//         get_dataset_data,     // 来自 data_query 模块
//         export_csv,           // 来自 data_export 模块
//         clear_dataset,        // 来自 data_export 模块
//         clear_all_datasets,   // 来自 data_export 模块
//     ])
// ```
//
// 如果没有 pub use，就需要写：
// ```rust
// use crate::commands::file_import::{import_csv, import_excel};
// use crate::commands::data_query::{get_dataset_list, get_dataset_info, get_dataset_data};
// use crate::commands::data_export::{export_csv, clear_dataset, clear_all_datasets};
// ```
//
// ============================================================================

// ============================================================================
// Rust 模块系统说明
// ============================================================================
//
// 1. 模块声明（pub mod）：
//    - 告诉 Rust 编译器："这里有一个子模块"
//    - Rust 会查找对应的文件或目录
//    - pub 表示这个模块是公开的，外部可以访问
//
// 2. 重新导出（pub use）：
//    - 将子模块中的项"提升"到当前模块
//    - 简化导入路径
//    - 提供更清晰的 API 接口
//
// 3. 模块路径：
//    - 绝对路径：crate::commands::file_import::import_csv
//    - 相对路径：super::file_import::import_csv（从父模块开始）
//    - 简化路径：crate::commands::import_csv（通过 pub use）
//
// 4. 可见性：
//    - pub：公开，任何地方都可以访问
//    - pub(crate)：仅在当前 crate 内可见
//    - 无修饰符：私有，仅在当前模块内可见
//
// ============================================================================

// ============================================================================
// 命令模块架构
// ============================================================================
//
// commands/
// ├── mod.rs              ← 当前文件（模块入口）
// ├── file_import.rs      ← 文件导入命令
// │   ├── import_csv()
// │   └── import_excel()
// ├── data_query.rs       ← 数据查询命令
// │   ├── get_dataset_list()
// │   ├── get_dataset_info()
// │   └── get_dataset_data()
// └── data_export.rs      ← 数据导出命令
//     ├── export_csv()
//     ├── clear_dataset()
//     └── clear_all_datasets()
//
// 所有命令都是 Tauri 命令（使用 #[tauri::command] 宏）
// 前端可以通过 invoke() 调用这些命令
//
// ============================================================================
