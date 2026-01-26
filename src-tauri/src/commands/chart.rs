// ============================================================================
// commands/chart.rs - 图表数据生成命令
// ============================================================================

use crate::AppState;
use crate::models::chart::{ChartConfig, ChartData, ChartType};
use polars::prelude::*;
use serde_json::Value as JsonValue;
use tauri::State;

/// 生成图表数据（ECharts dataset 格式）
#[tauri::command]
pub async fn generate_chart_data(config: ChartConfig, state: State<'_, AppState>) -> Result<ChartData, String> {
    // 获取当前数据集
    let current_df = {
        let store = state
            .data_store
            .lock()
            .map_err(|e| format!("Failed to lock data store: {}", e))?;
        store.get_current().ok_or("没有数据")?.clone()
    };

    // 根据图表类型构建 dataset
    let (dataset, data_count) = match config.chart_type {
        ChartType::Line | ChartType::Bar | ChartType::Scatter => build_xy_dataset(&current_df, &config)?,
        ChartType::Pie => build_pie_dataset(&current_df, &config)?,
        ChartType::Histogram => build_histogram_dataset(&current_df, &config)?,
    };

    Ok(ChartData {
        chart_type: config.chart_type,
        dataset,
        data_count,
    })
}

/// 构建 X-Y 轴图表的 dataset（折线图、柱状图、散点图）
fn build_xy_dataset(df: &DataFrame, config: &ChartConfig) -> Result<(Vec<Vec<JsonValue>>, usize), String> {
    let x_column = config
        .x_column
        .as_ref()
        .ok_or("折线图/柱状图/散点图需要指定 x_column")?;

    let y_columns = config
        .y_columns
        .as_ref()
        .ok_or("折线图/柱状图/散点图需要指定 y_columns")?;

    if y_columns.is_empty() {
        return Err("至少需要选择一个 Y 轴列".to_string());
    }

    // 构建列名列表：[x_column, y_column1, y_column2, ...]
    let mut columns = vec![x_column.clone()];
    columns.extend(y_columns.clone());

    // 验证列是否存在
    let column_names = df.get_column_names();
    for col_name in &columns {
        if !column_names.iter().any(|name| name.as_str() == col_name.as_str()) {
            return Err(format!("列 '{}' 不存在", col_name));
        }
    }

    // 选择指定的列
    let selected_df = df.select(&columns).map_err(|e| format!("选择列失败: {}", e))?;

    // 构建 dataset 二维数组
    let mut dataset: Vec<Vec<JsonValue>> = Vec::new();

    // 第一行：列名（表头）
    let header: Vec<JsonValue> = columns.iter().map(|name| JsonValue::String(name.clone())).collect();
    dataset.push(header);

    // 数据行
    let row_count = selected_df.height();
    for row_idx in 0..row_count {
        let mut row: Vec<JsonValue> = Vec::new();

        for col_name in &columns {
            let column = selected_df
                .column(col_name)
                .map_err(|e| format!("获取列 '{}' 失败: {}", col_name, e))?;

            let series = column.as_materialized_series();
            let value = series_value_to_json(series, row_idx)?;
            row.push(value);
        }

        dataset.push(row);
    }

    Ok((dataset, row_count))
}

/// 构建饼图的 dataset
fn build_pie_dataset(df: &DataFrame, config: &ChartConfig) -> Result<(Vec<Vec<JsonValue>>, usize), String> {
    let category_column = config.category_column.as_ref().ok_or("饼图需要指定 category_column")?;

    let value_column = config.value_column.as_ref().ok_or("饼图需要指定 value_column")?;

    let columns = vec![category_column.clone(), value_column.clone()];

    // 验证列是否存在
    let column_names = df.get_column_names();
    for col_name in &columns {
        if !column_names.iter().any(|name| name.as_str() == col_name.as_str()) {
            return Err(format!("列 '{}' 不存在", col_name));
        }
    }

    // 选择指定的列
    let selected_df = df.select(&columns).map_err(|e| format!("选择列失败: {}", e))?;

    // 构建 dataset 二维数组
    let mut dataset: Vec<Vec<JsonValue>> = Vec::new();

    // 第一行：列名（表头）
    let header: Vec<JsonValue> = columns.iter().map(|name| JsonValue::String(name.clone())).collect();
    dataset.push(header);

    // 数据行
    let row_count = selected_df.height();
    for row_idx in 0..row_count {
        let mut row: Vec<JsonValue> = Vec::new();

        for col_name in &columns {
            let column = selected_df
                .column(col_name)
                .map_err(|e| format!("获取列 '{}' 失败: {}", col_name, e))?;

            let series = column.as_materialized_series();
            let value = series_value_to_json(series, row_idx)?;
            row.push(value);
        }

        dataset.push(row);
    }

    Ok((dataset, row_count))
}

/// 构建直方图的 dataset
fn build_histogram_dataset(df: &DataFrame, config: &ChartConfig) -> Result<(Vec<Vec<JsonValue>>, usize), String> {
    let column_name = config
        .histogram_column
        .as_ref()
        .ok_or("直方图需要指定 histogram_column")?;

    let bins = config.histogram_bins.unwrap_or(10);
    if bins == 0 {
        return Err("直方图分箱数量必须大于 0".to_string());
    }

    let column_names = df.get_column_names();
    if !column_names.iter().any(|name| name.as_str() == column_name.as_str()) {
        return Err(format!("列 '{}' 不存在", column_name));
    }

    let series = df
        .column(column_name)
        .map_err(|e| format!("获取列 '{}' 失败: {}", column_name, e))?
        .as_materialized_series()
        .clone();

    let dtype = series.dtype();
    let is_numeric = matches!(
        dtype,
        DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Float32
            | DataType::Float64
            | DataType::Decimal(_, _)
    );

    if !is_numeric {
        return Err("直方图仅支持数值列".to_string());
    }

    let mut values: Vec<f64> = Vec::new();
    for idx in 0..series.len() {
        let value = series.get(idx).map_err(|e| format!("获取值失败: {}", e))?;
        if let Some(v) = any_value_to_f64(value) {
            values.push(v);
        }
    }

    if values.is_empty() {
        return Err("直方图列无可用数值".to_string());
    }

    let min = values.iter().cloned().fold(f64::INFINITY, |a, b| a.min(b));
    let max = values.iter().cloned().fold(f64::NEG_INFINITY, |a, b| a.max(b));

    let mut bin_width = (max - min) / bins as f64;
    if !bin_width.is_finite() || bin_width == 0.0 {
        bin_width = 1.0;
    }

    let mut counts = vec![0usize; bins];
    for v in values.iter().copied() {
        let mut idx = ((v - min) / bin_width).floor() as isize;
        if idx < 0 {
            idx = 0;
        }
        let mut idx = idx as usize;
        if idx >= bins {
            idx = bins - 1;
        }
        counts[idx] += 1;
    }

    let mut dataset: Vec<Vec<JsonValue>> = Vec::new();
    dataset.push(vec![
        JsonValue::String("bin".to_string()),
        JsonValue::String("count".to_string()),
    ]);

    for i in 0..bins {
        let start = min + bin_width * i as f64;
        let end = start + bin_width;
        let label = format!("{} ~ {}", format_bin_value(start), format_bin_value(end));
        dataset.push(vec![JsonValue::String(label), JsonValue::Number(counts[i].into())]);
    }

    Ok((dataset, values.len()))
}

fn any_value_to_f64(value: AnyValue) -> Option<f64> {
    match value {
        AnyValue::Null => None,
        AnyValue::Int8(v) => Some(v as f64),
        AnyValue::Int16(v) => Some(v as f64),
        AnyValue::Int32(v) => Some(v as f64),
        AnyValue::Int64(v) => Some(v as f64),
        AnyValue::UInt8(v) => Some(v as f64),
        AnyValue::UInt16(v) => Some(v as f64),
        AnyValue::UInt32(v) => Some(v as f64),
        AnyValue::UInt64(v) => Some(v as f64),
        AnyValue::Float32(v) => {
            if v.is_finite() {
                Some(v as f64)
            } else {
                None
            }
        }
        AnyValue::Float64(v) => {
            if v.is_finite() {
                Some(v)
            } else {
                None
            }
        }
        AnyValue::Decimal(v, scale, _) => {
            let divisor = 10f64.powi(scale as i32);
            Some(v as f64 / divisor)
        }
        _ => None,
    }
}

fn format_bin_value(value: f64) -> String {
    let mut s = format!("{:.4}", value);
    while s.contains('.') && s.ends_with('0') {
        s.pop();
    }
    if s.ends_with('.') {
        s.pop();
    }
    if s.is_empty() { "0".to_string() } else { s }
}

/// 将 Polars Series 的某个值转换为 JSON 值
fn series_value_to_json(series: &Series, idx: usize) -> Result<JsonValue, String> {
    let any_value = series.get(idx).map_err(|e| format!("获取值失败: {}", e))?;

    let json_value = match any_value {
        AnyValue::Null => JsonValue::Null,
        AnyValue::Boolean(b) => JsonValue::Bool(b),
        AnyValue::String(s) => JsonValue::String(s.to_string()),
        AnyValue::UInt8(v) => JsonValue::Number(v.into()),
        AnyValue::UInt16(v) => JsonValue::Number(v.into()),
        AnyValue::UInt32(v) => JsonValue::Number(v.into()),
        AnyValue::UInt64(v) => JsonValue::Number(v.into()),
        AnyValue::Int8(v) => JsonValue::Number(v.into()),
        AnyValue::Int16(v) => JsonValue::Number(v.into()),
        AnyValue::Int32(v) => JsonValue::Number(v.into()),
        AnyValue::Int64(v) => JsonValue::Number(v.into()),
        AnyValue::Float32(v) => {
            if v.is_finite() {
                serde_json::Number::from_f64(v as f64)
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null)
            } else {
                JsonValue::Null
            }
        }
        AnyValue::Float64(v) => {
            if v.is_finite() {
                serde_json::Number::from_f64(v)
                    .map(JsonValue::Number)
                    .unwrap_or(JsonValue::Null)
            } else {
                JsonValue::Null
            }
        }
        AnyValue::Date(v) => {
            // 将日期转换为字符串 (YYYY-MM-DD)
            let days = v;
            let date = chrono::NaiveDate::from_num_days_from_ce_opt(days + 719_163).ok_or("日期转换失败")?;
            JsonValue::String(date.format("%Y-%m-%d").to_string())
        }
        AnyValue::Datetime(v, time_unit, _) => {
            // 将日期时间转换为字符串
            let timestamp = match time_unit {
                TimeUnit::Nanoseconds => v / 1_000_000_000,
                TimeUnit::Microseconds => v / 1_000_000,
                TimeUnit::Milliseconds => v / 1_000,
            };
            let datetime = chrono::DateTime::from_timestamp(timestamp, 0).ok_or("日期时间转换失败")?;
            JsonValue::String(datetime.format("%Y-%m-%d %H:%M:%S").to_string())
        }
        _ => JsonValue::String(format!("{}", any_value)),
    };

    Ok(json_value)
}
