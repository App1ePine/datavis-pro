// ============================================================================
// data/mod.rs - 数据处理模块导出
// ============================================================================
// 这个文件是 data 模块的入口文件
// 它的作用是：
// 1. 声明子模块（store, loader）
// 2. 重新导出子模块中的核心类型和函数
//
// data 模块负责：
// - 数据存储（store.rs）：内存中的 DataFrame 存储和管理
// - 数据加载（loader.rs）：CSV/Excel 文件读取和转换
// ============================================================================

// ============================================================================
// 子模块声明
// ============================================================================

/// 数据存储模块
/// 定义了 DataStore 结构体，用于在内存中管理多个数据集
/// 使用 HashMap 存储 DataFrame 和元信息
pub mod store;

/// 数据加载模块
/// 提供 CSV 和 Excel 文件的读取功能
/// 使用 Polars 处理 CSV，使用 Calamine 处理 Excel
pub mod loader;

// ============================================================================
// 重新导出核心类型和函数
// ============================================================================

/// 重新导出数据存储相关类型
///
/// DataStore: 数据存储结构体
/// - 管理多个数据集（HashMap<String, DataFrame>）
/// - 提供 insert、get、remove、clear 等方法
///
/// SharedDataStore: 线程安全的共享数据存储
/// - 类型别名：Arc<Mutex<DataStore>>
/// - Arc: 允许多个所有者共享数据
/// - Mutex: 确保同一时间只有一个线程可以访问数据
/// - 用于 Tauri 的 AppState，在多个命令之间共享数据
pub use store::{DataStore, SharedDataStore};

/// 重新导出数据加载相关函数
///
/// load_csv: 从 CSV 文件加载 DataFrame
/// - 参数：文件路径
/// - 返回：Result<DataFrame>
/// - 使用 Polars 的 CsvReader
///
/// load_excel: 从 Excel 文件加载 DataFrame
/// - 参数：文件路径、可选的工作表名称
/// - 返回：Result<DataFrame>
/// - 使用 Calamine 读取 Excel，然后转换为 DataFrame
///
/// load_parquet: 从 Parquet 文件加载 DataFrame
/// - 参数：文件路径
/// - 返回：Result<DataFrame>
/// - 使用 Polars 的 ParquetReader
///
/// create_dataset_info: 从 DataFrame 创建元信息
/// - 参数：数据集 ID、文件路径、DataFrame
/// - 返回：DatasetInfo（包含行数、列信息、导入时间等）
///
/// dataframe_to_json_rows: 将 DataFrame 转换为 JSON 格式
/// - 参数：DataFrame（通常是切片后的部分数据）
/// - 返回：Vec<Vec<serde_json::Value>>（二维数组）
/// - 用于将数据传输给前端
pub use loader::{create_dataset_info, dataframe_to_json_rows, load_csv, load_excel, load_parquet};

// ============================================================================
// 使用示例
// ============================================================================
//
// 在 commands 模块中使用：
//
// ```rust
// use crate::data::{load_csv, create_dataset_info, DataStore};
//
// // 1. 加载 CSV 文件
// let df = load_csv("/path/to/file.csv")?;
//
// // 2. 创建元信息
// let info = create_dataset_info("uuid", "/path/to/file.csv", &df);
//
// // 3. 存储到 DataStore
// let mut store = DataStore::new();
// store.insert("uuid".to_string(), df, info);
// ```
//
// 在 lib.rs 中创建 AppState：
//
// ```rust
// use crate::data::{DataStore, SharedDataStore};
// use std::sync::{Arc, Mutex};
//
// let data_store: SharedDataStore = Arc::new(Mutex::new(DataStore::new()));
//
// pub struct AppState {
//     pub data_store: SharedDataStore,
// }
// ```
//
// ============================================================================

// ============================================================================
// 数据流说明
// ============================================================================
//
// 1. 文件导入流程：
//    用户选择文件 → Tauri Dialog
//        ↓
//    前端调用 import_csv/import_excel 命令
//        ↓
//    Rust 后端使用 load_csv/load_excel 读取文件
//        ↓
//    生成 UUID 和 DatasetInfo
//        ↓
//    存储到 DataStore（Arc<Mutex<HashMap<String, DataFrame>>>）
//        ↓
//    返回 DatasetInfo 给前端
//
// 2. 数据查询流程：
//    前端调用 get_dataset_data 命令（带分页参数）
//        ↓
//    Rust 后端从 DataStore 获取 DataFrame
//        ↓
//    使用 slice() 切片（分页）
//        ↓
//    使用 dataframe_to_json_rows 转换为 JSON
//        ↓
//    返回 DatasetData 给前端
//        ↓
//    前端使用 ag-Grid 展示数据
//
// 3. 数据导出流程：
//    前端调用 export_csv 命令（带输出路径）
//        ↓
//    Rust 后端从 DataStore 获取 DataFrame
//        ↓
//    使用 Polars CsvWriter 写入文件
//        ↓
//    返回文件路径给前端
//
// ============================================================================

// ============================================================================
// 模块架构
// ============================================================================
//
// data/
// ├── mod.rs              ← 当前文件（模块入口）
// ├── store.rs            ← 数据存储
// │   ├── DataStore       ← 数据存储结构体
// │   │   ├── datasets: HashMap<String, DataFrame>
// │   │   └── metadata: HashMap<String, DatasetInfo>
// │   ├── new()           ← 创建新的 DataStore
// │   ├── insert()        ← 插入数据集
// │   ├── get()           ← 获取 DataFrame
// │   ├── get_info()      ← 获取元信息
// │   ├── list_all()      ← 列出所有数据集
// │   ├── remove()        ← 删除数据集
// │   └── clear()         ← 清空所有数据集
// └── loader.rs           ← 数据加载
//     ├── load_csv()      ← 加载 CSV 文件
//     ├── load_excel()    ← 加载 Excel 文件
//     ├── create_dataset_info() ← 创建元信息
//     └── dataframe_to_json_rows() ← DataFrame 转 JSON
//
// ============================================================================

// ============================================================================
// 关键概念
// ============================================================================
//
// 1. DataFrame（Polars）：
//    - 类似于 Pandas 的 DataFrame
//    - 列式存储，性能优异
//    - 支持 lazy evaluation（延迟执行）
//    - 支持多线程并行处理
//
// 2. Arc<Mutex<T>>：
//    - Arc: Atomic Reference Counting（原子引用计数）
//      - 允许多个所有者共享数据
//      - 线程安全的引用计数
//    - Mutex: Mutual Exclusion（互斥锁）
//      - 确保同一时间只有一个线程可以访问数据
//      - 防止数据竞争
//
// 3. HashMap<String, T>：
//    - 键值对存储
//    - 使用数据集 ID（UUID）作为键
//    - 快速查找（O(1) 时间复杂度）
//
// 4. Result<T, E>：
//    - Rust 的错误处理类型
//    - Ok(T): 成功，包含值
//    - Err(E): 失败，包含错误
//    - 使用 ? 操作符传播错误
//
// ============================================================================
