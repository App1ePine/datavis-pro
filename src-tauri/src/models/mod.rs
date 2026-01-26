// ============================================================================
// models/mod.rs - 数据模型模块导出
// ============================================================================
// 这个文件是 models 模块的入口文件
// 它的作用是：
// 1. 声明子模块（dataset）
// 2. 重新导出子模块中的数据结构
//
// models 模块定义了应用中使用的所有数据结构
// 这些结构体用于：
// - 在 Rust 后端和前端之间传输数据（通过 JSON 序列化）
// - 存储数据集的元信息
// - 组织和管理数据
// ============================================================================

// ============================================================================
// 子模块声明
// ============================================================================

/// 数据集相关的数据模型
/// 定义了 DatasetInfo、ColumnInfo、DatasetData 等结构体
/// 这些结构体都实现了 Serialize 和 Deserialize trait，可以与 JSON 互相转换
pub mod dataset;

/// 图表相关的数据模型
/// 定义了 ChartType、ChartConfig、ChartData 等结构体
pub mod chart;

/// 操作历史相关的数据模型
/// 定义了 OperationType、HistoryEntry、HistoryEntryInfo 等结构体
/// 用于实现 undo/redo 功能和操作历史管理
pub mod history;
// ============================================================================
// 重新导出核心数据结构
// ============================================================================

/// 重新导出数据集相关的数据结构
///
/// ColumnInfo: 列信息
/// - name: String - 列名
/// - dtype: String - 数据类型（"Int64", "Float64", "String" 等）
/// - null_count: usize - 空值数量
///
/// DatasetInfo: 数据集元信息
/// - id: String - 数据集的唯一标识符（UUID）
/// - name: String - 文件名
/// - rows: usize - 总行数
/// - columns: Vec<ColumnInfo> - 列信息列表
/// - file_path: String - 原始文件路径
/// - imported_at: String - 导入时间（ISO 8601 格式）
///
/// DatasetData: 数据集的实际数据（用于传输给前端）
/// - columns: Vec<String> - 列名列表
/// - rows: Vec<Vec<serde_json::Value>> - 数据行（二维数组）
/// - total_rows: usize - 总行数（用于前端分页）
///
/// ColumnStats: 列统计信息
/// - name: String - 列名
/// - dtype: String - 数据类型
/// - total_count: usize - 总数目
/// - null_count: usize - 缺失值数量
/// - unique_count: usize - 唯一值数量
/// - max/min/mean/std: Option<f64> - 基础统计量（仅数值类型）
/// - q25/q50/q75: Option<f64> - 分位数（仅数值类型）
pub use dataset::{ColumnInfo, ColumnStats, DatasetData, DatasetInfo};

/// 重新导出历史相关的数据结构
///
/// OperationType: 操作类型枚举
/// - Import: 导入文件
/// - Unpivot: 纵表转横表
/// - Pivot: 横表转纵表
/// - DropNulls: 删除空值行
/// - SelectColumns: 选择列
/// - DropColumns: 删除列
/// - RenameColumns: 重命名列
/// - CastTypes: 转换列类型
/// - Filter: 筛选过滤
/// - FillNull: 空值填充
/// - RollingAverage: 移动平均
/// - RollingMedian: 移动中位数
///
/// FillStrategy: 空值填充策略
/// - Constant: 常量值
/// - Forward: 向前填充
/// - Backward: 向后填充
/// - Mean: 平均值
/// - Median: 中位数
/// - Min/Max: 最小/最大值
/// - Zero: 零
///
/// HistoryEntry: 历史条目（包含 DataFrame，不可序列化）
/// HistoryEntryInfo: 历史条目信息（可序列化，用于传输给前端）
pub use history::{HistoryEntry, HistoryEntryInfo, OperationType};

// ============================================================================
// 使用示例
// ============================================================================
//
// 在 commands 模块中使用：
//
// ```rust
// use crate::models::{DatasetInfo, ColumnInfo, DatasetData};
//
// // 1. 创建 DatasetInfo
// let info = DatasetInfo {
//     id: "uuid".to_string(),
//     name: "data.csv".to_string(),
//     rows: 1000,
//     columns: vec![
//         ColumnInfo {
//             name: "name".to_string(),
//             dtype: "String".to_string(),
//             null_count: 0,
//         },
//         ColumnInfo {
//             name: "age".to_string(),
//             dtype: "Int64".to_string(),
//             null_count: 5,
//         },
//     ],
//     file_path: "/path/to/data.csv".to_string(),
//     imported_at: "2024-01-01T00:00:00Z".to_string(),
// };
//
// // 2. 返回给前端（自动序列化为 JSON）
// #[tauri::command]
// pub async fn get_dataset_info(id: String) -> Result<DatasetInfo, String> {
//     Ok(info)
// }
//
// // 3. 创建 DatasetData
// let data = DatasetData {
//     columns: vec!["name".to_string(), "age".to_string()],
//     rows: vec![
//         vec![json!("Alice"), json!(25)],
//         vec![json!("Bob"), json!(30)],
//     ],
//     total_rows: 1000,
// };
// ```
//
// 前端接收数据：
//
// ```typescript
// // TypeScript 类型定义（与 Rust 结构体对应）
// interface ColumnInfo {
//   name: string;
//   dtype: string;
//   null_count: number;
// }
//
// interface DatasetInfo {
//   id: string;
//   name: string;
//   rows: number;
//   columns: ColumnInfo[];
//   file_path: string;
//   imported_at: string;
// }
//
// interface DatasetData {
//   columns: string[];
//   rows: any[][];
//   total_rows: number;
// }
//
// // 调用 Tauri 命令
// const info: DatasetInfo = await invoke('get_dataset_info', { id: 'uuid' });
// console.log(`数据集: ${info.name}, 行数: ${info.rows}`);
//
// const data: DatasetData = await invoke('get_dataset_data', {
//   datasetId: 'uuid',
//   offset: 0,
//   limit: 100
// });
// console.log(`返回 ${data.rows.length} 行，总共 ${data.total_rows} 行`);
// ```
//
// ============================================================================

// ============================================================================
// 数据结构设计说明
// ============================================================================
//
// 1. 为什么分离 DatasetInfo 和 DatasetData？
//    - DatasetInfo: 轻量级元信息，用于列表显示
//      - 前端可以快速获取所有数据集的概览
//      - 不需要传输大量数据
//    - DatasetData: 实际数据，按需加载
//      - 支持分页，避免一次性传输大量数据
//      - 只在用户查看时才加载
//
// 2. 为什么使用 Vec<Vec<serde_json::Value>>？
//    - 灵活性：可以表示任意类型的数据
//    - 简单性：不需要为每种数据类型定义结构体
//    - 兼容性：JSON 是前后端通信的标准格式
//    - 缺点：类型安全性较弱，需要前端自行处理类型
//
// 3. 为什么存储 file_path？
//    - 用户可能需要知道数据来源
//    - 可以用于重新加载文件
//    - 可以用于导出时的默认文件名
//
// 4. 为什么使用 String 存储 dtype？
//    - Polars 的类型系统很复杂（Int8, Int16, Int32, Int64, Float32, Float64 等）
//    - 前端不需要知道具体的 Rust 类型
//    - 使用字符串表示更简单，易于理解
//
// ============================================================================

// ============================================================================
// JSON 序列化示例
// ============================================================================
//
// DatasetInfo 序列化为 JSON：
//
// ```json
// {
//   "id": "550e8400-e29b-41d4-a716-446655440000",
//   "name": "sales_data.csv",
//   "rows": 1000,
//   "columns": [
//     {
//       "name": "date",
//       "dtype": "Date",
//       "null_count": 0
//     },
//     {
//       "name": "amount",
//       "dtype": "Float64",
//       "null_count": 5
//     },
//     {
//       "name": "customer",
//       "dtype": "String",
//       "null_count": 0
//     }
//   ],
//   "file_path": "/Users/user/data/sales_data.csv",
//   "imported_at": "2024-01-15T10:30:00Z"
// }
// ```
//
// DatasetData 序列化为 JSON：
//
// ```json
// {
//   "columns": ["date", "amount", "customer"],
//   "rows": [
//     ["2024-01-01", 100.5, "Alice"],
//     ["2024-01-02", 200.0, "Bob"],
//     ["2024-01-03", null, "Charlie"]
//   ],
//   "total_rows": 1000
// }
// ```
//
// ============================================================================

// ============================================================================
// 模块架构
// ============================================================================
//
// models/
// ├── mod.rs              ← 当前文件（模块入口）
// └── dataset.rs          ← 数据集相关结构体
//     ├── ColumnInfo      ← 列信息
//     ├── DatasetInfo     ← 数据集元信息
//     └── DatasetData     ← 数据集数据
//
// 所有结构体都实现了：
// - Serialize: 可以转换为 JSON（发送给前端）
// - Deserialize: 可以从 JSON 转换（接收前端数据）
// - Debug: 可以使用 {:?} 格式化打印
// - Clone: 可以克隆（创建副本）
//
// ============================================================================

// ============================================================================
// Serde 序列化说明
// ============================================================================
//
// 1. #[derive(Serialize, Deserialize)]：
//    - 自动实现 JSON 序列化和反序列化
//    - Tauri 会自动处理 Rust 类型和 JSON 之间的转换
//
// 2. 字段命名：
//    - Rust: snake_case（例如：null_count）
//    - JSON: camelCase（例如：nullCount）
//    - 可以使用 #[serde(rename = "nullCount")] 自定义
//    - 当前项目使用 snake_case（前后端统一）
//
// 3. Option<T>：
//    - Rust: Option<String>
//    - JSON: null 或 "value"
//    - Serde 自动处理 null 值
//
// 4. Vec<T>：
//    - Rust: Vec<ColumnInfo>
//    - JSON: [{"name": "col1", ...}, {"name": "col2", ...}]
//    - Serde 自动处理数组
//
// ============================================================================
