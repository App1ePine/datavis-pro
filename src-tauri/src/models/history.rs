// ============================================================================
// models/history.rs - 操作历史数据结构
// ============================================================================
// 这个文件定义了操作历史相关的数据结构
// 用于实现 undo/redo 功能和操作历史管理

use crate::models::DatasetInfo;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// 操作类型枚举
// ============================================================================
/// OperationType - 数据操作类型
///
/// 使用 Rust 的枚举来表示不同的数据操作
/// 每个变体可以携带该操作所需的参数
///
/// #[serde(tag = "type", content = "params")]：
/// - tag = "type": 在 JSON 中添加 "type" 字段表示操作类型
/// - content = "params": 参数存储在 "params" 字段中
///
/// 序列化示例：
/// ```json
/// {
///   "type": "Unpivot",
///   "params": {
///     "id_vars": ["name"],
///     "value_vars": ["2020", "2021"]
///   }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "params")]
pub enum OperationType {
    /// 导入文件操作
    Import { file_path: String },

    /// 横表转纵表（Unpivot/Melt）
    /// 将宽格式转为长格式
    ///
    /// 参数：
    /// - id_vars: 标识列（保持不变的列）
    /// - value_vars: 值列（需要转换的列）
    Unpivot {
        id_vars: Vec<String>,
        value_vars: Vec<String>,
        variable_name: String,
        value_name: String,
        sort_column: Option<String>,
    },

    /// 纵表转横表（Pivot）
    /// 将长格式转为宽格式
    ///
    /// 参数：
    /// - index: 索引列（行标识）
    /// - columns: 列名来源列
    /// - values: 值来源列
    Pivot {
        index: Vec<String>,
        columns: String,
        values: String,
    },

    /// 删除空值行
    ///
    /// 参数：
    /// - subset: 要检查的列（None 表示检查所有列）
    DropNulls { subset: Option<Vec<String>> },

    /// 删除全空行
    /// 删除所有列都为空的行
    DropAllNulls,

    /// 选择列
    ///
    /// 参数：
    /// - columns: 要保留的列名列表
    SelectColumns { columns: Vec<String> },

    /// 删除列
    ///
    /// 参数：
    /// - columns: 要删除的列名列表
    DropColumns { columns: Vec<String> },

    /// 重命名列
    ///
    /// 参数：
    /// - mapping: 旧列名 -> 新列名的映射
    RenameColumns { mapping: HashMap<String, String> },

    /// 转换列类型
    ///
    /// 参数：
    /// - mapping: 列名 -> 目标类型的映射
    ///   目标类型: "Int64", "Float64", "String", "Boolean", "Date"
    CastTypes { mapping: HashMap<String, String> },

    /// 筛选过滤
    ///
    /// 参数：
    /// - expression: Polars 表达式字符串
    Filter { expression: String },

    /// 空值填充
    ///
    /// 参数：
    /// - strategy: 填充策略
    FillNull { strategy: FillStrategy },

    /// 移动平均
    RollingAverage {
        column: String,
        window_size: usize,
        center: bool,
        min_periods: Option<usize>,
    },

    /// 移动中位数
    RollingMedian {
        column: String,
        window_size: usize,
        center: bool,
        min_periods: Option<usize>,
    },

    /// 移动求和
    RollingSum {
        column: String,
        window_size: usize,
        center: bool,
        min_periods: Option<usize>,
    },

    /// 移动最小值
    RollingMin {
        column: String,
        window_size: usize,
        center: bool,
        min_periods: Option<usize>,
    },

    /// 移动最大值
    RollingMax {
        column: String,
        window_size: usize,
        center: bool,
        min_periods: Option<usize>,
    },

    /// 移动标准差
    RollingStd {
        column: String,
        window_size: usize,
        center: bool,
        min_periods: Option<usize>,
    },

    /// 移动方差
    RollingVar {
        column: String,
        window_size: usize,
        center: bool,
        min_periods: Option<usize>,
    },

    /// 移动分位数
    RollingQuantile {
        column: String,
        window_size: usize,
        quantile: f64,
        center: bool,
        min_periods: Option<usize>,
    },
}

// ============================================================================
// 空值填充策略
// ============================================================================
/// FillStrategy - 空值填充策略
///
/// 定义了不同的空值填充方法
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum FillStrategy {
    /// 使用常量值填充
    Constant { value: String },

    /// 向前填充（使用前一个非空值）
    Forward,

    /// 向后填充（使用后一个非空值）
    Backward,

    /// 使用平均值填充（仅数值列）
    Mean,

    /// 使用中位数填充（仅数值列）
    Median,

    /// 使用最小值填充（仅数值列）
    Min,

    /// 使用最大值填充（仅数值列）
    Max,

    /// 使用零填充
    Zero,

    /// 使用 1 填充
    One,
}

// ============================================================================
// 历史条目结构体（内部使用，包含 DataFrame）
// ============================================================================
/// HistoryEntry - 操作历史条目
///
/// 存储每次操作后的完整状态
/// 注意：这个结构体不能序列化（因为 DataFrame 不能序列化）
/// 仅在 Rust 后端内部使用
pub struct HistoryEntry {
    /// 唯一标识符
    pub id: String,

    /// 操作类型
    pub operation: OperationType,

    /// 操作后的 DataFrame（完整快照）
    pub dataframe: DataFrame,

    /// 数据集元信息
    pub metadata: DatasetInfo,

    /// 时间戳（ISO 8601 格式）
    pub timestamp: String,

    /// 操作描述（用于显示）
    pub description: String,
}

// ============================================================================
// 历史条目信息（可序列化版本，不包含 DataFrame）
// ============================================================================
/// HistoryEntryInfo - 历史条目信息
///
/// 这是 HistoryEntry 的轻量级版本，用于传输给前端
/// 不包含 DataFrame，只包含元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntryInfo {
    /// 唯一标识符
    pub id: String,

    /// 操作类型
    pub operation: OperationType,

    /// 数据集元信息
    pub metadata: DatasetInfo,

    /// 时间戳
    pub timestamp: String,

    /// 操作描述
    pub description: String,
}

// ============================================================================
// 从 HistoryEntry 转换为 HistoryEntryInfo
// ============================================================================
impl From<&HistoryEntry> for HistoryEntryInfo {
    fn from(entry: &HistoryEntry) -> Self {
        Self {
            id: entry.id.clone(),
            operation: entry.operation.clone(),
            metadata: entry.metadata.clone(),
            timestamp: entry.timestamp.clone(),
            description: entry.description.clone(),
        }
    }
}

// ============================================================================
// 操作描述生成
// ============================================================================
impl OperationType {
    /// 生成操作的人类可读描述
    pub fn description(&self) -> String {
        match self {
            OperationType::Import { file_path } => {
                let file_name = std::path::Path::new(file_path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("未知文件");
                format!("导入文件: {}", file_name)
            }
            OperationType::Unpivot {
                id_vars,
                value_vars,
                sort_column,
                ..
            } => {
                let base = format!("横表转纵表 (标识列: {}, 值列: {})", id_vars.len(), value_vars.len());
                if let Some(sort_col) = sort_column {
                    format!("{} [按 {} 排序]", base, sort_col)
                } else {
                    base
                }
            }
            OperationType::Pivot { index, columns, values } => {
                format!(
                    "纵表转横表 (索引: {}, 列: {}, 值: {})",
                    index.join(", "),
                    columns,
                    values
                )
            }
            OperationType::DropNulls { subset } => {
                if let Some(cols) = subset {
                    format!("删除空值行 (检查 {} 列)", cols.len())
                } else {
                    "删除空值行 (检查所有列)".to_string()
                }
            }
            OperationType::DropAllNulls => "删除全空行".to_string(),
            OperationType::SelectColumns { columns } => {
                format!("选择列 ({} 列)", columns.len())
            }
            OperationType::DropColumns { columns } => {
                format!("删除列 ({} 列)", columns.len())
            }
            OperationType::RenameColumns { mapping } => {
                format!("重命名列 ({} 列)", mapping.len())
            }
            OperationType::CastTypes { mapping } => {
                format!("转换列类型 ({} 列)", mapping.len())
            }
            OperationType::Filter { .. } => "筛选过滤".to_string(),
            OperationType::FillNull { strategy } => {
                let strategy_name = match strategy {
                    FillStrategy::Constant { .. } => "常量",
                    FillStrategy::Forward => "向前填充",
                    FillStrategy::Backward => "向后填充",
                    FillStrategy::Mean => "平均值",
                    FillStrategy::Median => "中位数",
                    FillStrategy::Min => "最小值",
                    FillStrategy::Max => "最大值",
                    FillStrategy::Zero => "0",
                    FillStrategy::One => "1",
                };
                format!("空值填充 ({})", strategy_name)
            }
            OperationType::RollingAverage {
                column,
                window_size,
                center,
                min_periods,
            } => {
                let center_str = if *center { "是" } else { "否" };
                let min_p = min_periods.unwrap_or(1);
                format!(
                    "移动平均 (列: {}, 窗口: {}, 居中: {}, 最小样本: {})",
                    column, window_size, center_str, min_p
                )
            }
            OperationType::RollingMedian {
                column,
                window_size,
                center,
                min_periods,
            } => {
                let center_str = if *center { "是" } else { "否" };
                let min_p = min_periods.unwrap_or(1);
                format!(
                    "移动中位数 (列: {}, 窗口: {}, 居中: {}, 最小样本: {})",
                    column, window_size, center_str, min_p
                )
            }
            OperationType::RollingSum {
                column,
                window_size,
                center,
                min_periods,
            } => {
                let center_str = if *center { "是" } else { "否" };
                let min_p = min_periods.unwrap_or(1);
                format!(
                    "移动求和 (列: {}, 窗口: {}, 居中: {}, 最小样本: {})",
                    column, window_size, center_str, min_p
                )
            }
            OperationType::RollingMin {
                column,
                window_size,
                center,
                min_periods,
            } => {
                let center_str = if *center { "是" } else { "否" };
                let min_p = min_periods.unwrap_or(1);
                format!(
                    "移动最小值 (列: {}, 窗口: {}, 居中: {}, 最小样本: {})",
                    column, window_size, center_str, min_p
                )
            }
            OperationType::RollingMax {
                column,
                window_size,
                center,
                min_periods,
            } => {
                let center_str = if *center { "是" } else { "否" };
                let min_p = min_periods.unwrap_or(1);
                format!(
                    "移动最大值 (列: {}, 窗口: {}, 居中: {}, 最小样本: {})",
                    column, window_size, center_str, min_p
                )
            }
            OperationType::RollingStd {
                column,
                window_size,
                center,
                min_periods,
            } => {
                let center_str = if *center { "是" } else { "否" };
                let min_p = min_periods.unwrap_or(1);
                format!(
                    "移动标准差 (列: {}, 窗口: {}, 居中: {}, 最小样本: {})",
                    column, window_size, center_str, min_p
                )
            }
            OperationType::RollingVar {
                column,
                window_size,
                center,
                min_periods,
            } => {
                let center_str = if *center { "是" } else { "否" };
                let min_p = min_periods.unwrap_or(1);
                format!(
                    "移动方差 (列: {}, 窗口: {}, 居中: {}, 最小样本: {})",
                    column, window_size, center_str, min_p
                )
            }

            OperationType::RollingQuantile {
                column,
                window_size,
                quantile,
                center,
                min_periods,
            } => {
                let center_str = if *center { "是" } else { "否" };
                let min_p = min_periods.unwrap_or(1);
                format!(
                    "移动分位数 (列: {}, 窗口: {}, 分位: {}, 居中: {}, 最小样本: {})",
                    column, window_size, quantile, center_str, min_p
                )
            }
        }
    }
}

// ============================================================================
// 使用说明
// ============================================================================
//
// 1. 创建历史条目：
//    ```rust
//    let entry = HistoryEntry {
//        id: uuid::Uuid::new_v4().to_string(),
//        operation: OperationType::Import {
//            file_path: "/path/to/file.csv".to_string(),
//        },
//        dataframe: df.clone(),
//        metadata: info,
//        timestamp: chrono::Utc::now().to_rfc3339(),
//        description: operation.description(),
//    };
//    ```
//
// 2. 转换为可序列化版本：
//    ```rust
//    let info: HistoryEntryInfo = (&entry).into();
//    ```
//
// 3. 获取操作描述：
//    ```rust
//    let desc = operation.description();
//    ```
//
// ============================================================================
