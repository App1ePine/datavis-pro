// ============================================================================
// commands/operations.rs - 数据操作命令（简化版）
// ============================================================================
// 这个文件实现了基础的数据操作相关的 Tauri 命令
// Phase 1: 实现简单操作（drop_nulls, select_columns, drop_columns, rename_columns）

use crate::AppState;
use crate::models::{DatasetInfo, HistoryEntry, OperationType, ColumnInfo};
use crate::error::DataAnalystError;
use polars::prelude::*;
use std::collections::HashMap;
use tauri::State;

// ============================================================================
// 辅助函数：创建历史条目
// ============================================================================
/// 从 DataFrame 和操作类型创建历史条目
fn create_history_entry(
    df: DataFrame,
    operation: OperationType,
) -> Result<HistoryEntry, DataAnalystError> {
    // 生成唯一 ID
    let id = uuid::Uuid::new_v4().to_string();

    // 生成时间戳
    let timestamp = chrono::Utc::now().to_rfc3339();

    // 生成描述
    let description = operation.description();

    // 创建元信息
    let metadata = create_dataset_info(&df, &id, &description)?;

    Ok(HistoryEntry {
        id,
        operation,
        dataframe: df,
        metadata,
        timestamp,
        description,
    })
}

/// 从 DataFrame 创建 DatasetInfo
fn create_dataset_info(
    df: &DataFrame,
    id: &str,
    name: &str,
) -> Result<DatasetInfo, DataAnalystError> {
    let rows = df.height();

    // 提取列信息
    let columns: Vec<ColumnInfo> = df
        .get_columns()
        .iter()
        .map(|series| {
            let null_count = series.null_count();
            ColumnInfo {
                name: series.name().to_string(),
                dtype: format!("{:?}", series.dtype()),
                null_count,
            }
        })
        .collect();

    Ok(DatasetInfo {
        id: id.to_string(),
        name: name.to_string(),
        rows,
        columns,
        file_path: String::new(),
        imported_at: chrono::Utc::now().to_rfc3339(),
    })
}

// ============================================================================
// 1. 删除空值行（Drop Nulls）
// ============================================================================
/// 删除包含空值的行
///
/// 参数：
/// - subset: 要检查的列（None 表示检查所有列）
#[tauri::command]
pub async fn drop_nulls(
    subset: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut store = state.data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let current_df = store
        .get_current()
        .ok_or("没有数据")?
        .clone();

    // 执行 drop_nulls 操作
    let result_df = if let Some(cols) = &subset {
        // 使用 turbofish 语法明确指定类型
        current_df
            .drop_nulls::<String>(Some(cols.as_slice()))
            .map_err(|e| format!("删除空值行失败: {}", e))?
    } else {
        current_df
            .drop_nulls::<String>(None)
            .map_err(|e| format!("删除空值行失败: {}", e))?
    };

    let operation = OperationType::DropNulls { subset };
    let entry = create_history_entry(result_df, operation)
        .map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 2. 删除全空行（Drop All Nulls）
// ============================================================================
/// 删除所有列都为空的行
#[tauri::command]
pub async fn drop_all_nulls(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut store = state.data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let current_df = store
        .get_current()
        .ok_or("没有数据")?
        .clone();

    // 创建一个布尔掩码：所有列都为 null 的行
    let mut mask: Option<BooleanChunked> = None;

    for series in current_df.get_columns() {
        let is_null = series.is_null();
        mask = Some(match mask {
            None => is_null,
            Some(m) => &m & &is_null,
        });
    }

    let result_df = if let Some(mask) = mask {
        // 反转掩码：保留至少有一个非空值的行
        let keep_mask = !mask;
        current_df
            .filter(&keep_mask)
            .map_err(|e| format!("删除全空行失败: {}", e))?
    } else {
        current_df
    };

    let operation = OperationType::DropAllNulls;
    let entry = create_history_entry(result_df, operation)
        .map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 3. 选择列（Select Columns）
// ============================================================================
/// 保留指定的列
///
/// 参数：
/// - columns: 要保留的列名列表
#[tauri::command]
pub async fn select_columns(
    columns: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut store = state.data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let current_df = store
        .get_current()
        .ok_or("没有数据")?
        .clone();

    // 执行 select 操作
    let result_df = current_df
        .select(columns.iter().map(|s| s.as_str()))
        .map_err(|e| format!("选择列失败: {}", e))?;

    let operation = OperationType::SelectColumns { columns };
    let entry = create_history_entry(result_df, operation)
        .map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 4. 删除列（Drop Columns）
// ============================================================================
/// 删除指定的列
///
/// 参数：
/// - columns: 要删除的列名列表
#[tauri::command]
pub async fn drop_columns(
    columns: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut store = state.data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let current_df = store
        .get_current()
        .ok_or("没有数据")?
        .clone();

    // 执行 drop 操作
    let result_df = current_df.drop_many(columns.iter().map(|s| s.as_str()));

    let operation = OperationType::DropColumns { columns };
    let entry = create_history_entry(result_df, operation)
        .map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 5. 重命名列（Rename Columns）
// ============================================================================
/// 批量重命名列
///
/// 参数：
/// - mapping: 旧列名 -> 新列名的映射
#[tauri::command]
pub async fn rename_columns(
    mapping: HashMap<String, String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut store = state.data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let current_df = store
        .get_current()
        .ok_or("没有数据")?
        .clone();

    // 执行 rename 操作
    let mut result_df = current_df;
    for (old_name, new_name) in &mapping {
        result_df = result_df
            .rename(old_name.as_str(), new_name.as_str().into())
            .map_err(|e| format!("重命名列 {} 失败: {}", old_name, e))?
            .clone();
    }

    let operation = OperationType::RenameColumns { mapping };
    let entry = create_history_entry(result_df, operation)
        .map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 6. 转换列类型（Cast Types）
// ============================================================================
/// 批量转换列类型
///
/// 参数：
/// - mapping: 列名 -> 目标类型的映射
///   支持所有 Polars 基础类型
#[tauri::command]
pub async fn cast_types(
    mapping: HashMap<String, String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut store = state.data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let current_df = store
        .get_current()
        .ok_or("没有数据")?
        .clone();

    // 执行类型转换
    let mut result_df = current_df;
    for (col_name, target_type) in &mapping {
        let series = result_df
            .column(col_name)
            .map_err(|e| format!("找不到列 {}: {}", col_name, e))?
            .clone();

        // 解析目标类型
        let data_type = match target_type.as_str() {
            // 整数类型
            "Int8" => DataType::Int8,
            "Int16" => DataType::Int16,
            "Int32" => DataType::Int32,
            "Int64" => DataType::Int64,
            "UInt8" => DataType::UInt8,
            "UInt16" => DataType::UInt16,
            "UInt32" => DataType::UInt32,
            "UInt64" => DataType::UInt64,
            // 浮点类型
            "Float32" => DataType::Float32,
            "Float64" => DataType::Float64,
            // 字符串和布尔
            "String" => DataType::String,
            "Boolean" => DataType::Boolean,
            // 日期时间类型
            "Date" => DataType::Date,
            "Datetime" => DataType::Datetime(TimeUnit::Microseconds, None),
            "Time" => DataType::Time,
            "Duration" => DataType::Duration(TimeUnit::Microseconds),
            _ => return Err(format!("不支持的类型: {}", target_type)),
        };

        let casted_series = series.cast(&data_type)
            .map_err(|e| format!("转换列 {} 到 {} 失败: {}", col_name, target_type, e))?;

        result_df.with_column(casted_series)
            .map_err(|e| format!("更新列 {} 失败: {}", col_name, e))?;
    }

    let operation = OperationType::CastTypes { mapping };
    let entry = create_history_entry(result_df, operation)
        .map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 7. 筛选过滤（Filter）
// ============================================================================
/// 使用 SQL WHERE 子句筛选数据
///
/// 参数：
/// - expression: SQL WHERE 条件（不需要 "WHERE" 关键字）
///   例如：
///   - "age > 18"
///   - "name = 'Alice' AND age < 30"
///   - "salary >= 50000 OR department = 'IT'"
#[tauri::command]
pub async fn filter_data(
    expression: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut store = state.data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let current_df = store
        .get_current()
        .ok_or("没有数据")?
        .clone();

    // 使用 SQL 查询进行筛选
    // 构建完整的 SQL 查询：SELECT * FROM self WHERE <expression>
    let sql_query = format!("SELECT * FROM self WHERE {}", expression);

    // 创建 SQL 上下文并执行查询
    let mut ctx = polars::sql::SQLContext::new();
    ctx.register("self", current_df.lazy());

    let result_lf = ctx
        .execute(&sql_query)
        .map_err(|e| format!("SQL 查询执行失败: {}", e))?;

    let result_df = result_lf
        .collect()
        .map_err(|e| format!("收集查询结果失败: {}", e))?;

    let operation = OperationType::Filter { expression };
    let entry = create_history_entry(result_df, operation)
        .map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 8. 填充空值（Fill Null）
// ============================================================================
/// 填充空值
///
/// 参数：
/// - strategy: 填充策略（JSON 格式）
#[tauri::command]
pub async fn fill_null(
    strategy: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut store = state.data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let current_df = store
        .get_current()
        .ok_or("没有数据")?
        .clone();

    // 解析策略类型
    let strategy_type = strategy.get("strategy")
        .and_then(|v| v.as_str())
        .ok_or("缺少 strategy 字段")?;

    // 解析要填充的列（如果指定）
    let columns = strategy.get("columns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<String>>()
        });

    // 执行填充操作
    let filled_df = match strategy_type {
        "forward" => {
            current_df
                .fill_null(FillNullStrategy::Forward(None))
                .map_err(|e| format!("向前填充失败: {}", e))?
                .clone()
        }
        "backward" => {
            current_df
                .fill_null(FillNullStrategy::Backward(None))
                .map_err(|e| format!("向后填充失败: {}", e))?
                .clone()
        }
        "min" => {
            current_df
                .fill_null(FillNullStrategy::Min)
                .map_err(|e| format!("最小值填充失败: {}", e))?
                .clone()
        }
        "max" => {
            current_df
                .fill_null(FillNullStrategy::Max)
                .map_err(|e| format!("最大值填充失败: {}", e))?
                .clone()
        }
        "mean" => {
            current_df
                .fill_null(FillNullStrategy::Mean)
                .map_err(|e| format!("均值填充失败: {}", e))?
                .clone()
        }
        "zero" => {
            current_df
                .fill_null(FillNullStrategy::Zero)
                .map_err(|e| format!("零值填充失败: {}", e))?
                .clone()
        }
        "one" => {
            current_df
                .fill_null(FillNullStrategy::One)
                .map_err(|e| format!("1值填充失败: {}", e))?
                .clone()
        }
        _ => return Err(format!("不支持的填充策略: {}", strategy_type)),
    };

    // 如果指定了列，只替换这些列，其他列保持原样
    let result_df = if let Some(cols) = columns {
        if !cols.is_empty() {
            let mut final_df = current_df.clone();
            for col_name in &cols {
                if let Ok(filled_col) = filled_df.column(col_name) {
                    let _ = final_df
                        .with_column(filled_col.clone())
                        .map_err(|e| format!("替换列 {} 失败: {}", col_name, e))?;
                }
            }
            final_df
        } else {
            // 如果 columns 数组为空，填充所有列
            filled_df
        }
    } else {
        // 如果没有指定 columns，填充所有列
        filled_df
    };

    // 将前端的策略格式转换为后端的 FillStrategy 枚举
    let fill_strategy = match strategy_type {
        "forward" => crate::models::history::FillStrategy::Forward,
        "backward" => crate::models::history::FillStrategy::Backward,
        "min" => crate::models::history::FillStrategy::Min,
        "max" => crate::models::history::FillStrategy::Max,
        "mean" => crate::models::history::FillStrategy::Mean,
        "zero" => crate::models::history::FillStrategy::Zero,
        "one" => crate::models::history::FillStrategy::One,
        _ => return Err(format!("不支持的填充策略: {}", strategy_type)),
    };

    let operation = OperationType::FillNull {
        strategy: fill_strategy,
    };
    let entry = create_history_entry(result_df, operation)
        .map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 使用说明
// ============================================================================
//
// 前端调用示例：
//
// ```typescript
// import { invoke } from '@tauri-apps/api/core';
//
// // 1. 删除空值行
// await invoke('drop_nulls', { subset: ['age', 'salary'] });
//
// // 2. 删除全空行
// await invoke('drop_all_nulls');
//
// // 3. 选择列
// await invoke('select_columns', { columns: ['name', 'age', 'city'] });
//
// // 4. 删除列
// await invoke('drop_columns', { columns: ['temp_col'] });
//
// // 5. 重命名列
// await invoke('rename_columns', {
//   mapping: { 'old_name': 'new_name', 'age': '年龄' }
// });
//
// // 6. 转换列类型
// await invoke('cast_types', {
//   mapping: { 'age': 'Int64', 'salary': 'Float64' }
// });
//
// // 7. 筛选过滤
// await invoke('filter_data', {
//   expression: 'col("age") > 18'
// });
//
// // 8. 填充空值
// await invoke('fill_null', {
//   strategy: { strategy: 'forward', columns: ['age', 'salary'] }
// });
// ```
//
// ============================================================================

// ============================================================================
// TODO: 待实现的复杂操作
// ============================================================================
//
// 以下操作需要更复杂的 Polars API 使用，将在后续阶段实现：
//
// 1. unpivot_data - 纵表转横表（需要 melt API）
// 2. pivot_data - 横表转纵表（需要 pivot API）
// 3. rolling_average - 移动平均（需要 rolling API）
// 4. rolling_median - 移动中位数（需要 rolling API）
//
// ============================================================================
