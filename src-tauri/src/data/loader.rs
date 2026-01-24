// ============================================================================
// data/loader.rs - CSV/Excel 文件加载器
// ============================================================================
// 这个文件负责：
// 1. 读取 CSV 文件并转换为 Polars DataFrame
// 2. 读取 Excel 文件并转换为 Polars DataFrame
// 3. 从 DataFrame 提取元信息（DatasetInfo）
// 4. 将 DataFrame 转换为 JSON 格式（用于传输给前端）

// ============================================================================
// 导入依赖库
// ============================================================================
use crate::error::{DataAnalystError, Result};
// 自定义错误类型
use crate::models::{ColumnInfo, DatasetInfo};
use calamine::{open_workbook, DataType, Reader, Xlsx};
// Calamine: Excel 解析库
use chrono::{DateTime, Utc};
// Chrono: 时间处理库
use polars::prelude::*;
// Polars: 数据处理库
use std::path::Path;
// Path: 文件路径处理 // 数据模型

// ============================================================================
// CSV 文件加载
// ============================================================================
/// 从 CSV 文件加载 DataFrame
///
/// 这个函数使用 Polars 库读取 CSV 文件，并自动推断数据类型和分隔符
///
/// 支持的分隔符：
/// - 逗号 (,) - 标准 CSV
/// - 制表符 (\t) - TSV 文件
/// - 分号 (;) - 欧洲常用格式
/// - 竖线 (|) - 数据库导出常用
///
/// 参数：
/// - file_path: CSV 文件的路径（字符串切片）
///
/// 返回：
/// - Result<DataFrame>: 成功返回 DataFrame，失败返回错误
///
/// 示例：
/// ```rust
/// let df = load_csv("/path/to/data.csv")?;
/// println!("行数: {}", df.height());
/// println!("列数: {}", df.width());
/// ```
pub fn load_csv(file_path: &str) -> Result<DataFrame> {
    // 自动检测分隔符
    let separator = detect_csv_separator(file_path)?;

    // CsvReadOptions: CSV 读取选项配置
    let df = CsvReadOptions::default()
        // with_infer_schema_length: 设置类型推断的行数
        // Some(10000): 只读取前 10000 行来推断类型（平衡准确性和性能）
        // 对于大文件（25万行），这样可以大幅提升性能
        .with_infer_schema_length(Some(10000))
        // with_has_header: 指定是否有表头
        .with_has_header(true)
        // with_parse_options: 设置分隔符和日期解析
        .with_parse_options(
            CsvParseOptions::default()
                .with_separator(separator)  // 使用检测到的分隔符
                .with_try_parse_dates(true), // 尝试解析日期时间
        )
        // try_into_reader_with_file_path: 创建 CSV 读取器
        // Some(file_path.into()): 将字符串转换为 PathBuf
        // ? 操作符: 如果失败，自动返回错误
        .try_into_reader_with_file_path(Some(file_path.into()))?
        // finish: 完成读取，返回 DataFrame
        .finish()?;

    // 返回成功结果
    Ok(df)
}

// ============================================================================
// CSV 分隔符自动检测
// ============================================================================
/// 自动检测 CSV 文件的分隔符
///
/// 算法：
/// 1. 读取文件前 10 行作为样本
/// 2. 尝试常见的分隔符（逗号、制表符、分号、竖线）
/// 3. 计算每个分隔符产生的列数方差
/// 4. 选择方差最小的分隔符（列数最一致）
///
/// 参数：
/// - file_path: CSV 文件路径
///
/// 返回：
/// - Result<u8>: 检测到的分隔符（ASCII 字符）
fn detect_csv_separator(file_path: &str) -> Result<u8> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    // 打开文件
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);

    // 读取前 10 行作为样本
    let sample_lines: Vec<String> = reader
        .lines()
        .take(10)
        .filter_map(|line| line.ok())
        .collect();

    // 如果文件为空或只有一行，默认使用逗号
    if sample_lines.len() < 2 {
        return Ok(b',');
    }

    // 候选分隔符：逗号、制表符、分号、竖线
    let candidates = vec![
        (b',', "逗号"),
        (b'\t', "制表符"),
        (b';', "分号"),
        (b'|', "竖线"),
    ];

    let mut best_separator = b',';
    let mut best_score = f64::MAX;

    // 尝试每个候选分隔符
    for (sep, _name) in candidates {
        // 计算每行的列数
        let column_counts: Vec<usize> = sample_lines
            .iter()
            .map(|line| line.bytes().filter(|&b| b == sep).count() + 1)
            .collect();

        // 计算列数的平均值
        let mean = column_counts.iter().sum::<usize>() as f64 / column_counts.len() as f64;

        // 计算方差（列数的一致性）
        let variance: f64 = column_counts
            .iter()
            .map(|&count| {
                let diff = count as f64 - mean;
                diff * diff
            })
            .sum::<f64>()
            / column_counts.len() as f64;

        // 如果列数至少为 2（有效的分隔符），且方差更小，则更新最佳分隔符
        if mean >= 2.0 && variance < best_score {
            best_score = variance;
            best_separator = sep;
        }
    }

    Ok(best_separator)
}

// ============================================================================
// Parquet 文件加载
// ============================================================================
/// 从 Parquet 文件加载 DataFrame
///
/// Parquet 是一种列式存储格式，读取速度快，且保留完整的数据类型信息
///
/// 参数：
/// - file_path: Parquet 文件的路径
///
/// 返回：
/// - Result<DataFrame>: 成功返回 DataFrame，失败返回错误
///
/// 示例：
/// ```rust
/// let df = load_parquet("/path/to/data.parquet")?;
/// println!("行数: {}", df.height());
/// ```
pub fn load_parquet(file_path: &str) -> Result<DataFrame> {
    // 使用 Polars 的 ParquetReader 读取 Parquet 文件
    let file = std::fs::File::open(file_path)?;
    let df = ParquetReader::new(file).finish()?;
    Ok(df)
}

// ============================================================================
// Excel 文件加载
// ============================================================================
/// 从 Excel 文件加载 DataFrame
///
/// 这个函数使用 Calamine 库读取 Excel 文件（支持 .xlsx 和 .xls）
/// 然后将数据转换为 Polars DataFrame
///
/// 参数：
/// - file_path: Excel 文件的路径
/// - sheet_name: 可选的工作表名称
///   - Some("Sheet1"): 读取指定的工作表
///   - None: 读取第一个工作表
///
/// 返回：
/// - Result<DataFrame>: 成功返回 DataFrame，失败返回错误
///
/// 示例：
/// ```rust
/// // 读取第一个工作表
/// let df = load_excel("/path/to/data.xlsx", None)?;
///
/// // 读取指定工作表
/// let df = load_excel("/path/to/data.xlsx", Some("销售数据".to_string()))?;
/// ```
pub fn load_excel(file_path: &str, sheet_name: Option<String>) -> Result<DataFrame> {
    // open_workbook: 打开 Excel 文件
    // Xlsx<_>: 类型注解，表示这是一个 .xlsx 文件
    // ? 操作符: 如果打开失败，返回错误
    let mut workbook: Xlsx<_> = open_workbook(file_path)?;

    // 确定要读取的工作表名称
    let sheet = if let Some(name) = sheet_name {
        // 如果用户指定了工作表名称，使用指定的名称
        name
    } else {
        // 否则，使用第一个工作表
        workbook
            .sheet_names()
            .first() // 获取第一个工作表名称
            .ok_or_else(|| DataAnalystError::ExcelParseError("No sheets found".to_string()))?
            .clone() // 克隆字符串（因为 sheet_names() 返回的是引用）
    };

    // worksheet_range: 读取指定工作表的数据范围
    // 返回一个 Range 对象，包含所有单元格的数据
    let range = workbook.worksheet_range(&sheet)?;

    // 将 Excel Range 转换为 Polars DataFrame
    excel_range_to_dataframe(range)
}

// ============================================================================
// Excel Range 转 DataFrame
// ============================================================================
/// 将 Excel Range 转换为 Polars DataFrame
///
/// 这是一个内部函数，处理 Excel 数据到 DataFrame 的转换
///
/// 转换逻辑：
/// 1. 第一行作为列名（表头）
/// 2. 其余行作为数据
/// 3. 所有数据先转换为字符串（简化处理）
///
/// 参数：
/// - range: Calamine 的 Range 对象，包含 Excel 单元格数据
///
/// 返回：
/// - Result<DataFrame>: 转换后的 DataFrame
fn excel_range_to_dataframe(range: calamine::Range<calamine::Data>) -> Result<DataFrame> {
    // 检查 Range 是否为空
    if range.is_empty() {
        return Err(DataAnalystError::InvalidDataFormat(
            "Empty Excel sheet".to_string(),
        ));
    }

    // get_size: 获取数据范围的大小
    // height: 行数，width: 列数
    let (height, width) = range.get_size();

    // 再次检查是否有数据
    if height == 0 || width == 0 {
        return Err(DataAnalystError::InvalidDataFormat(
            "Empty Excel sheet".to_string(),
        ));
    }

    // ------------------------------------------------------------------------
    // 1. 提取列名（第一行）
    // ------------------------------------------------------------------------
    let headers: Vec<String> = (0..width)
        .map(|col| {
            // get_value: 获取指定单元格的值
            // (0, col as u32): 第 0 行（表头），第 col 列
            range
                .get_value((0, col as u32))
                .and_then(|v| v.as_string()) // 尝试转换为字符串
                .unwrap_or_else(|| format!("Column_{}", col + 1)) // 如果失败，使用默认名称
        })
        .collect();

    // ------------------------------------------------------------------------
    // 2. 读取数据行（跳过第一行）
    // ------------------------------------------------------------------------
    // Vec<Column>: Polars 的列集合
    let mut columns: Vec<Column> = Vec::new();

    // 遍历每一列
    for (col_idx, header) in headers.iter().enumerate() {
        // 为每一列创建一个值数组
        let mut values: Vec<Option<String>> = Vec::new();

        // 遍历每一行（从第 1 行开始，跳过表头）
        for row_idx in 1..height {
            // 获取单元格的值
            let cell_value = range.get_value((row_idx as u32, col_idx as u32));

            // 将单元格值转换为字符串
            let value = cell_value.map(|v| match v {
                // 根据不同的数据类型进行转换
                calamine::Data::Int(i) => i.to_string(), // 整数
                calamine::Data::Float(f) => f.to_string(), // 浮点数
                calamine::Data::String(s) => s.clone(),  // 字符串
                calamine::Data::Bool(b) => b.to_string(), // 布尔值
                calamine::Data::DateTime(dt) => dt.to_string(), // 日期时间
                calamine::Data::Error(e) => format!("Error: {:?}", e), // 错误
                calamine::Data::Empty => String::new(),  // 空单元格
                _ => String::new(),                      // 其他类型
            });

            // 添加到值数组
            values.push(value);
        }

        // 创建 Polars Series（列）
        // Series::new: 创建一个新的列
        // header.as_str().into(): 列名
        // values: 列的值
        let series = Series::new(header.as_str().into(), values);

        // 将 Series 转换为 Column 并添加到列集合
        columns.push(series.into());
    }

    // ------------------------------------------------------------------------
    // 3. 创建 DataFrame
    // ------------------------------------------------------------------------
    // DataFrame::new: 从列集合创建 DataFrame
    // map_err: 将 Polars 错误转换为我们的自定义错误
    let df = DataFrame::new(columns)?;

    // ------------------------------------------------------------------------
    // 4. 类型推导：将字符串列转换为合适的类型
    // ------------------------------------------------------------------------
    // 遍历每一列，尝试将字符串转换为更具体的类型（数值、日期等）
    let mut typed_columns: Vec<Column> = Vec::new();

    for col in df.get_columns() {
        let series = col.as_materialized_series();

        // 尝试转换为数值类型
        if let Ok(numeric) = series.cast(&datatypes::DataType::Float64) {
            // 检查是否所有非空值都能成功转换
            if numeric.null_count() == series.null_count() {
                typed_columns.push(numeric.into());
                continue;
            }
        }

        // 尝试转换为整数类型
        if let Ok(integer) = series.cast(&datatypes::DataType::Int64) {
            if integer.null_count() == series.null_count() {
                typed_columns.push(integer.into());
                continue;
            }
        }

        // 尝试转换为布尔类型
        if let Ok(boolean) = series.cast(&datatypes::DataType::Boolean) {
            if boolean.null_count() == series.null_count() {
                typed_columns.push(boolean.into());
                continue;
            }
        }

        // 如果都不行，保持原始字符串类型
        typed_columns.push(col.clone());
    }

    // 创建类型推导后的 DataFrame
    DataFrame::new(typed_columns).map_err(|e| e.into())
}

// ============================================================================
// 创建数据集元信息
// ============================================================================
/// 从 DataFrame 创建数据集元信息
///
/// 这个函数提取 DataFrame 的元数据，用于在前端显示
///
/// 参数：
/// - id: 数据集的唯一标识符（UUID）
/// - file_path: 原始文件路径
/// - df: Polars DataFrame
///
/// 返回：
/// - DatasetInfo: 数据集元信息结构体
///
/// 提取的信息包括：
/// - 文件名
/// - 行数
/// - 列信息（列名、数据类型、空值数量）
/// - 导入时间
pub fn create_dataset_info(id: &str, file_path: &str, df: &DataFrame) -> DatasetInfo {
    // ------------------------------------------------------------------------
    // 1. 提取文件名
    // ------------------------------------------------------------------------
    // Path::new: 创建路径对象
    // file_name: 获取文件名部分
    // and_then: 如果有值，继续处理
    // to_str: 转换为字符串
    // unwrap_or: 如果失败，使用默认值 "unknown"
    let file_name = Path::new(file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    // ------------------------------------------------------------------------
    // 2. 提取列信息
    // ------------------------------------------------------------------------
    // get_columns: 获取所有列
    // iter: 创建迭代器
    // map: 对每一列进行转换
    // collect: 收集到 Vec 中
    let columns = df
        .get_columns()
        .iter()
        .map(|col| {
            // dtype: 获取列的数据类型
            // format!("{:?}", ...): 将类型转换为字符串
            // 例如：Int64, Float64, Utf8（字符串）, Boolean, Date 等
            let dtype = format!("{:?}", col.dtype());

            // null_count: 获取空值数量
            let null_count = col.null_count();

            // 创建 ColumnInfo 结构体
            ColumnInfo {
                name: col.name().to_string(), // 列名
                dtype,                        // 数据类型
                null_count,                   // 空值数量
            }
        })
        .collect();

    // ------------------------------------------------------------------------
    // 3. 获取当前时间
    // ------------------------------------------------------------------------
    // Utc::now: 获取当前 UTC 时间
    let now: DateTime<Utc> = Utc::now();

    // ------------------------------------------------------------------------
    // 4. 创建并返回 DatasetInfo
    // ------------------------------------------------------------------------
    DatasetInfo {
        id: id.to_string(),               // 数据集 ID
        name: file_name,                  // 文件名
        rows: df.height(),                // 行数（height 返回 DataFrame 的行数）
        columns,                          // 列信息
        file_path: file_path.to_string(), // 原始文件路径
        imported_at: now.to_rfc3339(),    // ISO 8601 格式的时间字符串
    }
}

// ============================================================================
// DataFrame 转 JSON
// ============================================================================
/// 将 DataFrame 的一部分转换为 JSON 格式的行数据
///
/// 这个函数用于将 DataFrame 转换为前端可以使用的 JSON 格式
///
/// 参数：
/// - df: Polars DataFrame（通常是切片后的部分数据）
///
/// 返回：
/// - Vec<Vec<serde_json::Value>>: 二维数组
///   - 外层 Vec: 多行数据
///   - 内层 Vec: 一行中的多个值
///   - serde_json::Value: JSON 值（可以是数字、字符串、null 等）
///
/// 示例输出：
/// ```json
/// [
///   ["张三", 25, "北京"],
///   ["李四", 30, "上海"],
///   ["王五", null, "广州"]
/// ]
/// ```
pub fn dataframe_to_json_rows(df: &DataFrame) -> Vec<Vec<serde_json::Value>> {
    // 创建结果数组
    let mut rows = Vec::new();

    // height: 获取行数
    let height = df.height();

    // get_columns: 获取所有列
    let columns = df.get_columns();

    // 遍历每一行
    for row_idx in 0..height {
        // 创建一行的数据
        let mut row = Vec::new();

        // 遍历每一列
        for col in columns {
            // as_materialized_series: 获取物化的 Series
            // （确保数据已经加载到内存中）
            let series = col.as_materialized_series();

            // 将单元格值转换为 JSON
            let value = series_value_to_json(series, row_idx);

            // 添加到 行数据
            row.push(value);
        }

        // 添加到结果数组
        rows.push(row);
    }

    rows
}

// ============================================================================
// Series 值转 JSON
// ============================================================================
/// 将 Series 中的单个值转换为 JSON Value
///
/// 这是一个内部函数，处理不同数据类型到 JSON 的转换
///
/// 参数：
/// - series: Polars Series（列）
/// - idx: 行索引
///
/// 返回：
/// - serde_json::Value: JSON 值
fn series_value_to_json(series: &Series, idx: usize) -> serde_json::Value {
    // 导入 AnyValue 类型（Polars 的通用值类型）
    use datatypes::AnyValue;

    // get: 获取指定索引的值
    // unwrap: 假设索引有效（在实际使用中，索引总是有效的）
    let any_value = series.get(idx).unwrap();

    // 根据值的类型进行转换
    match any_value {
        // Null 值 → JSON null
        AnyValue::Null => serde_json::Value::Null,

        // 布尔值 → JSON boolean
        AnyValue::Boolean(b) => serde_json::Value::Bool(b),

        // 字符串 → JSON string
        AnyValue::String(s) => serde_json::Value::String(s.to_string()),

        // 无符号整数 → JSON number
        AnyValue::UInt8(v) => serde_json::json!(v),
        AnyValue::UInt16(v) => serde_json::json!(v),
        AnyValue::UInt32(v) => serde_json::json!(v),
        AnyValue::UInt64(v) => serde_json::json!(v),

        // 有符号整数 → JSON number
        AnyValue::Int8(v) => serde_json::json!(v),
        AnyValue::Int16(v) => serde_json::json!(v),
        AnyValue::Int32(v) => serde_json::json!(v),
        AnyValue::Int64(v) => serde_json::json!(v),

        // 浮点数 → JSON number（需要检查是否有效）
        AnyValue::Float32(v) => {
            if v.is_finite() {
                // 检查是否是有限数（不是 NaN 或 Infinity）
                serde_json::json!(v)
            } else {
                serde_json::Value::Null // 无效的浮点数转换为 null
            }
        }
        AnyValue::Float64(v) => {
            if v.is_finite() {
                serde_json::json!(v)
            } else {
                serde_json::Value::Null
            }
        }

        // 日期和时间类型 → JSON string (格式化为 yyyy-MM-dd HH:mm:ss)
        AnyValue::Date(d) => {
            // Date 类型：转换为 yyyy-MM-dd 格式
            serde_json::Value::String(format!(
                "{:04}-{:02}-{:02}",
                d / 10000,
                (d % 10000) / 100,
                d % 100
            ))
        }
        AnyValue::Datetime(dt, time_unit, _) => {
            // Datetime 类型：转换为 yyyy-MM-dd HH:mm:ss 格式
            use chrono::{DateTime, Datelike, Timelike};

            let timestamp_seconds = match time_unit {
                TimeUnit::Nanoseconds => dt / 1_000_000_000,
                TimeUnit::Microseconds => dt / 1_000_000,
                TimeUnit::Milliseconds => dt / 1_000,
            };

            if let Some(dt_utc) = DateTime::from_timestamp(timestamp_seconds, 0) {
                let naive_dt = dt_utc.naive_utc();
                serde_json::Value::String(format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                    naive_dt.year(),
                    naive_dt.month(),
                    naive_dt.day(),
                    naive_dt.hour(),
                    naive_dt.minute(),
                    naive_dt.second()
                ))
            } else {
                serde_json::Value::String(dt.to_string())
            }
        }
        AnyValue::Time(t) => {
            // Time 类型：转换为 HH:mm:ss 格式
            let hours = t / 3_600_000_000_000;
            let minutes = (t % 3_600_000_000_000) / 60_000_000_000;
            let seconds = (t % 60_000_000_000) / 1_000_000_000;
            serde_json::Value::String(format!("{:02}:{:02}:{:02}", hours, minutes, seconds))
        }

        // 其他类型 → 使用 Debug 格式转换为字符串
        _ => serde_json::Value::String(format!("{:?}", any_value)),
    }
}

// ============================================================================
// 性能说明
// ============================================================================
//
// 1. CSV 读取性能：
//    - Polars 使用多线程并行读取
//    - 自动推断数据类型（前 1000 行）
//    - 对于大文件（>100MB），读取速度仍然很快
//
// 2. Excel 读取性能：
//    - Calamine 是用纯 Rust 实现，速度快
//    - 但 Excel 格式比 CSV 复杂，读取稍慢
//    - 所有数据先转换为字符串（简化处理，但可能损失类型信息）
//
// 3. DataFrame 转 JSON：
//    - 需要遍历所有单元格，对于大数据集可能较慢
//    - 建议使用分页，每次只转换需要显示的数据
//    - 前端通常只需要 100-1000 行数据
//
// 4. 内存使用：
//    - DataFrame 存储在内存中
//    - 对于大文件，内存占用可能很高
//    - 未来可以考虑使用 Lazy DataFrame 或 SQLite
//
// ============================================================================

// ============================================================================
// 使用示例
// ============================================================================
//
// ```rust
// // 1. 加载 CSV 文件
// let df = load_csv("/path/to/data.csv")?;
// println!("加载了 {} 行数据", df.height());
//
// // 2. 加载 Excel 文件
// let df = load_excel("/path/to/data.xlsx", None)?;
//
// // 3. 创建元信息
// let info = create_dataset_info("uuid", "/path/to/data.csv", &df);
// println!("文件名: {}", info.name);
// println!("列数: {}", info.columns.len());
//
// // 4. 转换为 JSON
// let json_rows = dataframe_to_json_rows(&df);
// println!("第一行: {:?}", json_rows[0]);
// ```
//
// ============================================================================
