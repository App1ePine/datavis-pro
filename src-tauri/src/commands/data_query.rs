// ============================================================================
// commands/data_query.rs - 数据查询命令（新架构）
// ============================================================================
// 这个文件定义了数据查询相关的 Tauri 命令
// 新架构：只有一个当前数据集，不再有多数据集列表

// ============================================================================
// 导入依赖
// ============================================================================
use crate::data::dataframe_to_json_rows;
use crate::models::{ColumnStats, DatasetData, DatasetInfo};
use crate::AppState;
use polars::prelude::*;

// ============================================================================
// 获取当前数据集的元信息
// ============================================================================
/// 获取当前数据集的元信息
///
/// 返回当前正在查看的数据集的元信息
///
/// 参数：
/// - state: 应用状态（自动注入）
///
/// 返回：
/// - Result<Option<DatasetInfo>, String>:
///   - Some(info): 有数据时返回元信息
///   - None: 没有数据
#[tauri::command]
pub async fn get_current_info(
    state: tauri::State<'_, AppState>,
) -> Result<Option<DatasetInfo>, String> {
    let store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    Ok(store.get_current_info().cloned())
}

// ============================================================================
// 获取当前数据集的分页数据
// ============================================================================
/// 获取当前数据集的分页数据
///
/// 支持分页，避免一次性传输大量数据
///
/// 参数：
/// - offset: 起始行索引（从 0 开始）
/// - limit: 要获取的行数
/// - state: 应用状态（自动注入）
///
/// 返回：
/// - Result<DatasetData, String>:
///   - 成功：返回 DatasetData（包含列名、数据行、总行数）
///   - 失败：返回错误消息
#[tauri::command]
pub async fn get_current_data(
    offset: usize,
    limit: usize,
    state: tauri::State<'_, AppState>,
) -> Result<DatasetData, String> {
    let store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    // 获取当前 DataFrame
    let df = store
        .get_current()
        .ok_or("没有数据")?;

    // 获取总行数
    let total_rows = df.height();

    // 计算实际的切片范围
    let start = offset.min(total_rows);
    let end = (offset + limit).min(total_rows);

    // 切片 DataFrame
    let sliced_df = df.slice(start as i64, end - start);

    // 提取列名
    let columns: Vec<String> = sliced_df
        .get_column_names()
        .iter()
        .map(|s| s.to_string())
        .collect();

    // 转换为 JSON 行
    let rows = dataframe_to_json_rows(&sliced_df);

    Ok(DatasetData {
        columns,
        rows,
        total_rows,
    })
}

// ============================================================================
// 获取列统计信息
// ============================================================================
/// 获取指定列的统计信息
///
/// 参数：
/// - column_name: 列名
/// - state: 应用状态
///
/// 返回：
/// - Result<ColumnStats, String>: 列统计信息
#[tauri::command]
pub async fn get_column_stats(
    column_name: String,
    state: tauri::State<'_, AppState>,
) -> Result<ColumnStats, String> {
    let store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let df = store
        .get_current()
        .ok_or("没有数据")?;

    let series = df
        .column(&column_name)
        .map_err(|e| format!("找不到列 {}: {}", column_name, e))?;

    // 基础统计
    let total_count = series.len();
    let null_count = series.null_count();
    let unique_count = series.n_unique()
        .map_err(|e| format!("计算唯一值数量失败: {}", e))?;

    // 数值统计
    let (max, min, mean, std, q25, q50, q75) = if series.dtype().is_numeric() {
        // 转换为 Float64 进行统计
        match series.cast(&DataType::Float64) {
            Ok(float_series) => {
                match float_series.f64() {
                    Ok(ca) => {
                        let max_val = ca.max();
                        let min_val = ca.min();
                        let mean_val = ca.mean();
                        let std_val = ca.std(1);
                        let q25_val = ca.quantile(0.25, QuantileMethod::default())
                            .ok().flatten();
                        let q50_val = ca.quantile(0.50, QuantileMethod::default())
                            .ok().flatten();
                        let q75_val = ca.quantile(0.75, QuantileMethod::default())
                            .ok().flatten();
                        (max_val, min_val, mean_val, std_val, q25_val, q50_val, q75_val)
                    }
                    Err(_) => (None, None, None, None, None, None, None)
                }
            }
            Err(_) => (None, None, None, None, None, None, None)
        }
    } else {
        (None, None, None, None, None, None, None)
    };

    // 布尔统计
    let (true_count, false_count) = if matches!(series.dtype(), DataType::Boolean) {
        match series.bool() {
            Ok(bool_series) => {
                let true_cnt = bool_series.sum().map(|v| v as usize);
                let false_cnt = true_cnt.map(|t| total_count - null_count - t);
                (true_cnt, false_cnt)
            }
            Err(_) => (None, None)
        }
    } else {
        (None, None)
    };

    // 日期时间统计
    let (min_datetime, max_datetime, datetime_range_days) = match series.dtype() {
        DataType::Date => {
            // Date 类型：转换为物理类型 i32
            if let Ok(date_series) = series.date() {
                let physical = date_series.physical();
                let min_val = physical.min();
                let max_val = physical.max();

                let min_str = min_val.and_then(|days| {
                    chrono::NaiveDate::from_num_days_from_ce_opt(days + 719163)
                        .map(|d| d.format("%Y-%m-%d").to_string())
                });

                let max_str = max_val.and_then(|days| {
                    chrono::NaiveDate::from_num_days_from_ce_opt(days + 719163)
                        .map(|d| d.format("%Y-%m-%d").to_string())
                });

                let range_days = match (min_val, max_val) {
                    (Some(min), Some(max)) => Some((max - min) as f64),
                    _ => None
                };

                (min_str, max_str, range_days)
            } else {
                (None, None, None)
            }
        }
        DataType::Datetime(_, _) => {
            // Datetime 类型：转换为物理类型 i64
            if let Ok(dt_series) = series.datetime() {
                let physical = dt_series.physical();
                let min_val = physical.min();
                let max_val = physical.max();

                let min_str = min_val.map(|micros| {
                    let secs = micros / 1_000_000;
                    let nanos = ((micros % 1_000_000) * 1000) as u32;
                    chrono::DateTime::from_timestamp(secs, nanos)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                });

                let max_str = max_val.map(|micros| {
                    let secs = micros / 1_000_000;
                    let nanos = ((micros % 1_000_000) * 1000) as u32;
                    chrono::DateTime::from_timestamp(secs, nanos)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_default()
                });

                let range_days = match (min_val, max_val) {
                    (Some(min), Some(max)) => {
                        let diff_micros = max - min;
                        Some(diff_micros as f64 / 86_400_000_000.0)
                    }
                    _ => None
                };

                (min_str, max_str, range_days)
            } else {
                (None, None, None)
            }
        }
        _ => (None, None, None)
    };

    Ok(ColumnStats {
        name: column_name,
        dtype: format!("{:?}", series.dtype()),
        total_count,
        null_count,
        unique_count,
        max,
        min,
        mean,
        std,
        q25,
        q50,
        q75,
        min_datetime,
        max_datetime,
        datetime_range_days,
        true_count,
        false_count,
    })
}
