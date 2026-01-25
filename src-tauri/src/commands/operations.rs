// ============================================================================
// commands/operations.rs - 数据操作命令（简化版）
// ============================================================================
// 这个文件实现了基础的数据操作相关的 Tauri 命令
// Phase 1: 实现简单操作（drop_nulls, select_columns, drop_columns, rename_columns）
// Phase 2: 实现复杂操作（unpivot, pivot, rolling）

use crate::AppState;
use crate::error::DataAnalystError;
use crate::models::history::FillStrategy;
use crate::models::{ColumnInfo, DatasetInfo, HistoryEntry, OperationType};
use polars::prelude::*;
use std::collections::HashMap;
use tauri::State;

// Use polars-ops directly for stable pivot implementation
// Function called via fully qualified path in pivot_data

// ============================================================================
// 辅助函数：创建历史条目
// ============================================================================
/// 从 DataFrame 和操作类型创建历史条目
fn create_history_entry(df: DataFrame, operation: OperationType) -> Result<HistoryEntry, DataAnalystError> {
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
fn create_dataset_info(df: &DataFrame, id: &str, name: &str) -> Result<DatasetInfo, DataAnalystError> {
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
pub async fn drop_nulls(subset: Option<Vec<String>>, state: State<'_, AppState>) -> Result<(), String> {
    let (current_df, subset_clone) = {
        let store = state
            .data_store
            .lock()
            .map_err(|e| format!("Failed to lock data store: {}", e))?;
        let df = store.get_current().ok_or("没有数据")?.clone();
        (df, subset.clone())
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        if let Some(cols) = &subset_clone {
            current_df
                .drop_nulls::<String>(Some(cols.as_slice()))
                .map_err(|e| format!("删除空值行失败: {}", e))
        } else {
            current_df
                .drop_nulls::<String>(None)
                .map_err(|e| format!("删除空值行失败: {}", e))
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let operation = OperationType::DropNulls { subset };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 2. 删除全空行（Drop All Nulls）
// ============================================================================
/// 删除所有列都为空的行
#[tauri::command]
pub async fn drop_all_nulls(state: State<'_, AppState>) -> Result<(), String> {
    let current_df = {
        let store = state
            .data_store
            .lock()
            .map_err(|e| format!("Failed to lock data store: {}", e))?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        let mut mask: Option<BooleanChunked> = None;

        for series in current_df.get_columns() {
            let is_null = series.is_null();
            mask = Some(match mask {
                None => is_null,
                Some(m) => &m & &is_null,
            });
        }

        if let Some(mask) = mask {
            let keep_mask = !mask;
            current_df
                .filter(&keep_mask)
                .map_err(|e| format!("删除全空行失败: {}", e))
        } else {
            Ok::<DataFrame, String>(current_df)
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let operation = OperationType::DropAllNulls;
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;

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
pub async fn select_columns(columns: Vec<String>, state: State<'_, AppState>) -> Result<(), String> {
    let cols_clone = columns.clone();

    let current_df = {
        let store = state
            .data_store
            .lock()
            .map_err(|e| format!("Failed to lock data store: {}", e))?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        current_df
            .select(cols_clone.iter().map(|s| s.as_str()))
            .map_err(|e| format!("选择列失败: {}", e))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let operation = OperationType::SelectColumns { columns };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;

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
pub async fn drop_columns(columns: Vec<String>, state: State<'_, AppState>) -> Result<(), String> {
    let cols_clone = columns.clone();

    let current_df = {
        let store = state
            .data_store
            .lock()
            .map_err(|e| format!("Failed to lock data store: {}", e))?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        Ok::<DataFrame, String>(current_df.drop_many(cols_clone.iter().map(|s| s.as_str())))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let operation = OperationType::DropColumns { columns };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;

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
pub async fn rename_columns(mapping: HashMap<String, String>, state: State<'_, AppState>) -> Result<(), String> {
    let mapping_clone = mapping.clone();

    let current_df = {
        let store = state
            .data_store
            .lock()
            .map_err(|e| format!("Failed to lock data store: {}", e))?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        let mut df = current_df;
        for (old_name, new_name) in &mapping_clone {
            df = df
                .rename(old_name.as_str(), new_name.as_str().into())
                .map_err(|e| format!("重命名列 {} 失败: {}", old_name, e))?
                .clone();
        }
        Ok::<DataFrame, String>(df)
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let operation = OperationType::RenameColumns { mapping };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;

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
pub async fn cast_types(mapping: HashMap<String, String>, state: State<'_, AppState>) -> Result<(), String> {
    let mapping_clone = mapping.clone();

    let current_df = {
        let store = state
            .data_store
            .lock()
            .map_err(|e| format!("Failed to lock data store: {}", e))?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        let mut df = current_df;
        for (col_name, target_type) in &mapping_clone {
            let series = df
                .column(col_name)
                .map_err(|e| format!("找不到列 {}: {}", col_name, e))?
                .clone();

            let data_type = match target_type.as_str() {
                "Int8" => DataType::Int8,
                "Int16" => DataType::Int16,
                "Int32" => DataType::Int32,
                "Int64" => DataType::Int64,
                "UInt8" => DataType::UInt8,
                "UInt16" => DataType::UInt16,
                "UInt32" => DataType::UInt32,
                "UInt64" => DataType::UInt64,
                "Float32" => DataType::Float32,
                "Float64" => DataType::Float64,
                "String" => DataType::String,
                "Boolean" => DataType::Boolean,
                "Date" => DataType::Date,
                "Datetime" => DataType::Datetime(TimeUnit::Microseconds, None),
                "Time" => DataType::Time,
                "Duration" => DataType::Duration(TimeUnit::Microseconds),
                _ => return Err(format!("不支持的类型: {}", target_type)),
            };

            let casted_series = series
                .cast(&data_type)
                .map_err(|e| format!("转换列 {} 到 {} 失败: {}", col_name, target_type, e))?;

            df = df
                .with_column(casted_series)
                .map_err(|e| format!("更新列 {} 失败: {}", col_name, e))?
                .clone();
        }
        Ok::<DataFrame, String>(df)
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let operation = OperationType::CastTypes { mapping };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;

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
#[tauri::command]
pub async fn filter_data(expression: String, state: State<'_, AppState>) -> Result<(), String> {
    let expr_clone = expression.clone();

    let current_df = {
        let store = state
            .data_store
            .lock()
            .map_err(|e| format!("Failed to lock data store: {}", e))?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        let sql_query = format!("SELECT * FROM self WHERE {}", expr_clone);
        let mut ctx = polars::sql::SQLContext::new();
        ctx.register("self", current_df.lazy());

        let result_lf = ctx
            .execute(&sql_query)
            .map_err(|e| format!("SQL 查询执行失败: {}", e))?;

        result_lf.collect().map_err(|e| format!("收集查询结果失败: {}", e))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let operation = OperationType::Filter { expression };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;

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
pub async fn fill_null(strategy: serde_json::Value, state: State<'_, AppState>) -> Result<(), String> {
    // 预处理参数
    let strategy_type = strategy
        .get("strategy")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or("缺少 strategy 字段")?;

    let columns = strategy.get("columns").and_then(|v| v.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect::<Vec<String>>()
    });

    let strat_val_opt = strategy
        .get("value")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_default();

    // Cloning for closure
    let strategy_type_clone = strategy_type.clone();

    let current_df = {
        let store = state
            .data_store
            .lock()
            .map_err(|e| format!("Failed to lock data store: {}", e))?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        let filled_df = match strategy_type_clone.as_str() {
            "forward" => current_df.fill_null(FillNullStrategy::Forward(None)).unwrap(),
            "backward" => current_df.fill_null(FillNullStrategy::Backward(None)).unwrap(),
            "min" => current_df.fill_null(FillNullStrategy::Min).unwrap(),
            "max" => current_df.fill_null(FillNullStrategy::Max).unwrap(),
            "mean" => current_df.fill_null(FillNullStrategy::Mean).unwrap(),
            "zero" => current_df.fill_null(FillNullStrategy::Zero).unwrap(),
            "one" => current_df.fill_null(FillNullStrategy::One).unwrap(),
            // 对于 constant，这里简化处理，实际应解析 value
            "constant" => current_df.fill_null(FillNullStrategy::Forward(None)).unwrap(),
            _ => return Err(format!("不支持的填充策略: {}", strategy_type_clone)),
        };

        if let Some(cols) = columns {
            if !cols.is_empty() {
                let mut final_df = current_df.clone();
                for col_name in &cols {
                    if let Ok(filled_col) = filled_df.column(col_name) {
                        final_df = final_df
                            .with_column(filled_col.clone())
                            .map_err(|e| format!("替换列 {} 失败: {}", col_name, e))?
                            .clone();
                    }
                }
                Ok::<DataFrame, String>(final_df)
            } else {
                Ok::<DataFrame, String>(filled_df)
            }
        } else {
            Ok::<DataFrame, String>(filled_df)
        }
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    // 重新解析 strategy 用于记录
    let fill_strategy = match strategy_type.as_str() {
        "forward" => FillStrategy::Forward,
        "backward" => FillStrategy::Backward,
        "min" => FillStrategy::Min,
        "max" => FillStrategy::Max,
        "mean" => FillStrategy::Mean,
        "zero" => FillStrategy::Zero,
        "one" => FillStrategy::One,
        "constant" => FillStrategy::Constant { value: strat_val_opt },
        _ => FillStrategy::Forward,
    };

    let operation = OperationType::FillNull {
        strategy: fill_strategy,
    };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 9. 纵表转横表（Unpivot/Melt）
// ============================================================================
#[tauri::command]
pub async fn unpivot_data(
    id_vars: Vec<String>,
    value_vars: Vec<String>,
    variable_name: Option<String>,
    value_name: Option<String>,
    sort_column: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let var_name = variable_name.unwrap_or_else(|| "variable".to_string());
    let val_name = value_name.unwrap_or_else(|| "value".to_string());

    // Clonable vars for closure
    let id_vars_clone = id_vars.clone();
    let value_vars_clone = value_vars.clone();
    let var_name_clone = var_name.clone();
    let val_name_clone = val_name.clone();
    let sort_col_clone = sort_column.clone();

    let current_df = {
        let store = state
            .data_store
            .lock()
            .map_err(|e| format!("Failed to lock data store: {}", e))?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        // Polars 0.52+ unpivot: (args, index)
        // With 'pivot' feature enabled, 'unpivot' method should be available on DataFrame
        // If unpivot is not available, we use melt (alias usually)
        let df = current_df
            .unpivot(&value_vars_clone, &id_vars_clone)
            .map_err(|e| format!("Unpivot 操作失败: {}", e))?;

        let mut df = df;
        if var_name_clone != "variable" {
            df = df
                .rename("variable", var_name_clone.as_str().into())
                .map_err(|e| format!("重命名 variable 失败: {}", e))?
                .clone();
        }
        if val_name_clone != "value" {
            df = df
                .rename("value", val_name_clone.as_str().into())
                .map_err(|e| format!("重命名 value 失败: {}", e))?
                .clone();
        }

        if let Some(sort_col) = sort_col_clone {
            df = df
                .sort([sort_col.as_str()], SortMultipleOptions::default())
                .map_err(|e| format!("排序失败: {}", e))?;
        }

        Ok::<DataFrame, String>(df)
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let operation = OperationType::Unpivot {
        id_vars,
        value_vars,
        variable_name: var_name,
        value_name: val_name,
        sort_column,
    };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 10. 横表转纵表（Pivot）
// ============================================================================
#[tauri::command]
pub async fn pivot_data(
    index: Vec<String>,
    columns: String,
    values: String,
    aggregate: Option<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Clone for closure
    let index_clone = index.clone();
    let columns_clone = columns.clone();
    let values_clone = values.clone();
    let agg_str = aggregate.clone().unwrap_or_else(|| "first".to_string());

    let current_df = {
        let store = state
            .data_store
            .lock()
            .map_err(|e| format!("Failed to lock data store: {}", e))?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        // 使用 LazyFrame 进行聚合和重塑，避免 Eager Pivot 类型问题
        let agg_expr = match agg_str.as_str() {
            "first" => col(&values_clone).first(),
            "last" => col(&values_clone).last(),
            "sum" => col(&values_clone).sum(),
            "mean" => col(&values_clone).mean(),
            "min" => col(&values_clone).min(),
            "max" => col(&values_clone).max(),
            "count" => col(&values_clone).count(),
            "median" => col(&values_clone).median(),
            _ => return Err(format!("不支持的聚合函数: {}", agg_str)),
        };

        // 1. GroupBy + Aggregate (预聚合)
        let mut group_cols = index_clone.clone();
        group_cols.push(columns_clone.clone());

        let grouped = current_df
            .lazy()
            .group_by(group_cols.iter().map(|s| col(s)).collect::<Vec<_>>())
            .agg([agg_expr.alias(&values_clone)])
            .collect()
            .map_err(|e| format!("分组聚合失败: {}", e))?;

        // 2. Pivot (重塑)
        // 使用 polars-ops 直接调用 pivot_stable 函数
        // pivot_stable 签名: (df, on, index, values, sort, agg_fn, separator)
        // on: columns that will become headers (columns_clone)
        // index: columns to keep as rows (index_clone)
        // values: value columns (values_clone)
        // agg_fn: None (since we pre-aggregated)

        polars_ops::frame::pivot::pivot_stable(
            &grouped,
            [columns_clone],      // on
            Some(index_clone),    // index
            Some([values_clone]), // values
            false,                // sort
            None,                 // agg_fn
            None,                 // separator
        )
        .map_err(|e| format!("透视表操作失败: {}", e))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let operation = OperationType::Pivot {
        index,
        columns,
        values: values.to_string(),
    };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 11. 移动平均（Rolling Average）
// ============================================================================
#[tauri::command]
pub async fn rolling_average(
    column: String,
    window_size: usize,
    center: bool,
    min_periods: Option<usize>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if window_size < 1 {
        return Err("窗口大小必须至少为 1".to_string());
    }

    let column_clone = column.clone();

    let current_df = {
        let store = state
            .data_store
            .lock()
            .map_err(|e| format!("Failed to lock data store: {}", e))?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        let new_col_name = format!("{}_rolling_avg_{}", column_clone, window_size);

        // Config options
        let options = RollingOptionsFixedWindow {
            window_size,
            min_periods: min_periods.unwrap_or(1),
            weights: None,
            center,
            fn_params: None,
        };

        current_df
            .lazy()
            .with_column(col(&column_clone).rolling_mean(options).alias(&new_col_name))
            .collect()
            .map_err(|e| format!("计算移动平均失败: {}", e))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let operation = OperationType::RollingAverage {
        column,
        window_size,
        center,
        min_periods,
    };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 12. 移动中位数（Rolling Median）
// ============================================================================
#[tauri::command]
pub async fn rolling_median(
    column: String,
    window_size: usize,
    center: bool,
    min_periods: Option<usize>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if window_size < 1 {
        return Err("窗口大小必须至少为 1".to_string());
    }

    let column_clone = column.clone();

    let current_df = {
        let store = state
            .data_store
            .lock()
            .map_err(|e| format!("Failed to lock data store: {}", e))?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        let new_col_name = format!("{}_rolling_median_{}", column_clone, window_size);

        // Use rolling options
        let options = RollingOptionsFixedWindow {
            window_size,
            min_periods: min_periods.unwrap_or(1),
            weights: None,
            center,
            fn_params: None,
        };

        current_df
            .lazy()
            .with_column(col(&column_clone).rolling_median(options).alias(&new_col_name))
            .collect()
            .map_err(|e| format!("计算移动中位数失败: {}", e))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let operation = OperationType::RollingMedian {
        column,
        window_size,
        center,
        min_periods,
    };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;

    store.push_operation(entry);

    Ok(())
}

// ============================================================================
// 13. 移动求和（Rolling Sum）
// ============================================================================
#[tauri::command]
pub async fn rolling_sum(
    column: String,
    window_size: usize,
    center: bool,
    min_periods: Option<usize>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if window_size < 1 {
        return Err("窗口大小必须至少为 1".to_string());
    }

    let column_clone = column.clone();
    let current_df = {
        let store = state.data_store.lock().map_err(|e| e.to_string())?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        let new_col_name = format!("{}_rolling_sum_{}", column_clone, window_size);
        let options = RollingOptionsFixedWindow {
            window_size,
            min_periods: min_periods.unwrap_or(1),
            weights: None,
            center,
            fn_params: None,
        };

        current_df
            .lazy()
            .with_column(col(&column_clone).rolling_sum(options).alias(&new_col_name))
            .collect()
            .map_err(|e| format!("计算移动求和失败: {}", e))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state.data_store.lock().map_err(|e| e.to_string())?;
    let operation = OperationType::RollingSum {
        column,
        window_size,
        center,
        min_periods,
    };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;
    store.push_operation(entry);
    Ok(())
}

// ============================================================================
// 14. 移动最小值（Rolling Min）
// ============================================================================
#[tauri::command]
pub async fn rolling_min(
    column: String,
    window_size: usize,
    center: bool,
    min_periods: Option<usize>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if window_size < 1 {
        return Err("窗口大小必须至少为 1".to_string());
    }
    let column_clone = column.clone();
    let current_df = {
        let store = state.data_store.lock().map_err(|e| e.to_string())?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        let new_col_name = format!("{}_rolling_min_{}", column_clone, window_size);
        let options = RollingOptionsFixedWindow {
            window_size,
            min_periods: min_periods.unwrap_or(1),
            weights: None,
            center,
            fn_params: None,
        };
        current_df
            .lazy()
            .with_column(col(&column_clone).rolling_min(options).alias(&new_col_name))
            .collect()
            .map_err(|e| format!("计算移动最小值失败: {}", e))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state.data_store.lock().map_err(|e| e.to_string())?;
    let operation = OperationType::RollingMin {
        column,
        window_size,
        center,
        min_periods,
    };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;
    store.push_operation(entry);
    Ok(())
}

// ============================================================================
// 15. 移动最大值（Rolling Max）
// ============================================================================
#[tauri::command]
pub async fn rolling_max(
    column: String,
    window_size: usize,
    center: bool,
    min_periods: Option<usize>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if window_size < 1 {
        return Err("窗口大小必须至少为 1".to_string());
    }
    let column_clone = column.clone();
    let current_df = {
        let store = state.data_store.lock().map_err(|e| e.to_string())?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        let new_col_name = format!("{}_rolling_max_{}", column_clone, window_size);
        let options = RollingOptionsFixedWindow {
            window_size,
            min_periods: min_periods.unwrap_or(1),
            weights: None,
            center,
            fn_params: None,
        };
        current_df
            .lazy()
            .with_column(col(&column_clone).rolling_max(options).alias(&new_col_name))
            .collect()
            .map_err(|e| format!("计算移动最大值失败: {}", e))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state.data_store.lock().map_err(|e| e.to_string())?;
    let operation = OperationType::RollingMax {
        column,
        window_size,
        center,
        min_periods,
    };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;
    store.push_operation(entry);
    Ok(())
}

// ============================================================================
// 16. 移动标准差（Rolling Std）
// ============================================================================
#[tauri::command]
pub async fn rolling_std(
    column: String,
    window_size: usize,
    center: bool,
    min_periods: Option<usize>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if window_size < 1 {
        return Err("窗口大小必须至少为 1".to_string());
    }
    let column_clone = column.clone();
    let current_df = {
        let store = state.data_store.lock().map_err(|e| e.to_string())?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        let new_col_name = format!("{}_rolling_std_{}", column_clone, window_size);
        let options = RollingOptionsFixedWindow {
            window_size,
            min_periods: min_periods.unwrap_or(1),
            weights: None,
            center,
            fn_params: None,
        };
        current_df
            .lazy()
            .with_column(col(&column_clone).rolling_std(options).alias(&new_col_name))
            .collect()
            .map_err(|e| format!("计算移动标准差失败: {}", e))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state.data_store.lock().map_err(|e| e.to_string())?;
    let operation = OperationType::RollingStd {
        column,
        window_size,
        center,
        min_periods,
    };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;
    store.push_operation(entry);
    Ok(())
}

// ============================================================================
// 17. 移动方差（Rolling Var）
// ============================================================================
#[tauri::command]
pub async fn rolling_var(
    column: String,
    window_size: usize,
    center: bool,
    min_periods: Option<usize>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if window_size < 1 {
        return Err("窗口大小必须至少为 1".to_string());
    }
    let column_clone = column.clone();
    let current_df = {
        let store = state.data_store.lock().map_err(|e| e.to_string())?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        let new_col_name = format!("{}_rolling_var_{}", column_clone, window_size);
        let options = RollingOptionsFixedWindow {
            window_size,
            min_periods: min_periods.unwrap_or(1),
            weights: None,
            center,
            fn_params: None,
        };
        current_df
            .lazy()
            .with_column(col(&column_clone).rolling_var(options).alias(&new_col_name))
            .collect()
            .map_err(|e| format!("计算移动方差失败: {}", e))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state.data_store.lock().map_err(|e| e.to_string())?;
    let operation = OperationType::RollingVar {
        column,
        window_size,
        center,
        min_periods,
    };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;
    store.push_operation(entry);
    Ok(())
}

// ============================================================================
// 19. 移动分位数（Rolling Quantile）
// ============================================================================
#[tauri::command]
pub async fn rolling_quantile(
    column: String,
    window_size: usize,
    quantile: f64,
    center: bool,
    min_periods: Option<usize>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    if window_size < 1 {
        return Err("窗口大小必须至少为 1".to_string());
    }
    if quantile < 0.0 || quantile > 1.0 {
        return Err("分位数必须在 0.0 到 1.0 之间".to_string());
    }

    let column_clone = column.clone();
    let current_df = {
        let store = state.data_store.lock().map_err(|e| e.to_string())?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    let result_df = tauri::async_runtime::spawn_blocking(move || {
        let new_col_name = format!("{}_rolling_quantile_{}_{}", column_clone, window_size, quantile);

        let options = RollingOptionsFixedWindow {
            window_size,
            min_periods: min_periods.unwrap_or(1),
            weights: None,
            center,
            fn_params: None,
        };

        // rolling_quantile(method, quantile, options)
        current_df
            .lazy()
            .with_column(
                col(&column_clone)
                    .rolling_quantile(QuantileMethod::Linear, quantile, options)
                    .alias(&new_col_name),
            )
            .collect()
            .map_err(|e| format!("计算移动分位数失败: {}", e))
    })
    .await
    .map_err(|e| e.to_string())??;

    let mut store = state.data_store.lock().map_err(|e| e.to_string())?;
    let operation = OperationType::RollingQuantile {
        column,
        window_size,
        quantile,
        center,
        min_periods,
    };
    let entry = create_history_entry(result_df, operation).map_err(|e| e.to_string())?;
    store.push_operation(entry);
    Ok(())
}
