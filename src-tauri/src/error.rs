// ============================================================================
// error.rs - 自定义错误类型定义
// ============================================================================
// 这个文件定义了应用中所有可能出现的错误类型
// 使用 thiserror 库来简化错误处理代码

// 导入 thiserror 库的 Error trait（特征）
// trait 类似于其他语言的接口（interface）
use thiserror::Error;

// ============================================================================
// 错误枚举类型
// ============================================================================
/// DataAnalystError - 数据分析应用的错误类型
///
/// enum（枚举）是 Rust 中的一种类型，可以表示多种可能的值
/// 每个变体（variant）代表一种特定的错误情况
///
/// #[derive(Error, Debug)] 是一个派生宏，自动实现以下功能：
/// - Error: 使这个类型成为一个标准错误类型
/// - Debug: 允许使用 {:?} 格式化输出错误信息
#[derive(Error, Debug)]
pub enum DataAnalystError {
    /// 文件读取错误
    ///
    /// #[error("...")] 定义了错误的显示消息
    /// {0} 表示插入第一个参数的值（这里是 String）
    #[error("Failed to read file: {0}")]
    FileReadError(String),

    /// CSV 解析错误
    #[error("Failed to parse CSV: {0}")]
    CsvParseError(String),

    /// Excel 解析错误
    #[error("Failed to parse Excel: {0}")]
    ExcelParseError(String),

    /// 数据集未找到错误
    #[error("Dataset not found: {0}")]
    DatasetNotFound(String),

    /// 无效的数据格式错误
    #[error("Invalid data format: {0}")]
    InvalidDataFormat(String),

    /// Polars 库错误
    #[error("Polars error: {0}")]
    PolarsError(String),

    /// IO（输入/输出）错误
    ///
    /// #[from] 属性表示可以自动从 std::io::Error 转换为这个错误类型
    /// 这样当函数返回 io::Error 时，会自动转换为 DataAnalystError::IoError
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON 序列化/反序列化错误
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// 无效操作错误
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

// ============================================================================
// 错误转换实现
// ============================================================================
// impl 关键字用于为类型实现方法或 trait
// From trait 用于定义类型之间的转换

/// 从 Polars 错误转换为自定义错误
///
/// 当 Polars 库返回错误时，这个实现会自动将其转换为 DataAnalystError
impl From<polars::error::PolarsError> for DataAnalystError {
    /// from 方法定义了如何进行转换
    ///
    /// 参数：
    /// - err: Polars 的错误对象
    ///
    /// 返回：
    /// - Self: 表示 DataAnalystError 类型
    fn from(err: polars::error::PolarsError) -> Self {
        // 将 Polars 错误转换为字符串，然后包装在 PolarsError 变体中
        // err.to_string() 将错误对象转换为可读的字符串
        DataAnalystError::PolarsError(err.to_string())
    }
}

/// 从 Calamine 错误转换为自定义错误
///
/// Calamine 是用于读取 Excel 文件的库
impl From<calamine::Error> for DataAnalystError {
    fn from(err: calamine::Error) -> Self {
        DataAnalystError::ExcelParseError(err.to_string())
    }
}

/// 从 Calamine XLSX 错误转换为自定义错误
impl From<calamine::XlsxError> for DataAnalystError {
    fn from(err: calamine::XlsxError) -> Self {
        DataAnalystError::ExcelParseError(err.to_string())
    }
}

/// 从自定义错误转换为字符串
///
/// 这个实现允许将 DataAnalystError 转换为 String
/// 在 Tauri 命令中，错误必须是 String 类型才能传递给前端
impl From<DataAnalystError> for String {
    fn from(err: DataAnalystError) -> Self {
        // to_string() 方法由 Error trait 提供
        // 它会使用 #[error("...")] 中定义的格式化字符串
        err.to_string()
    }
}

// ============================================================================
// 类型别名
// ============================================================================
/// Result 类型别名
///
/// 在 Rust 中，Result<T, E> 是一个枚举，表示操作可能成功（Ok(T)）或失败（Err(E)）
/// 这里我们定义了一个类型别名，将错误类型固定为 DataAnalystError
///
/// 使用示例：
/// ```rust
/// fn read_file() -> Result<String> {
///     // 如果成功，返回 Ok(data)
///     // 如果失败，返回 Err(DataAnalystError::FileReadError(...))
/// }
/// ```
pub type Result<T> = std::result::Result<T, DataAnalystError>;

// ============================================================================
// 错误处理流程说明
// ============================================================================
//
// 1. 当 Rust 代码中发生错误时：
//    - 函数返回 Result<T>（实际上是 Result<T, DataAnalystError>）
//    - 如果成功：Ok(value)
//    - 如果失败：Err(DataAnalystError::SomeError(...))
//
// 2. 错误自动转换：
//    - 使用 ? 操作符时，错误会自动转换
//    - 例如：let data = read_file()?;
//    - 如果 read_file 返回 io::Error，会自动转换为 DataAnalystError::IoError
//
// 3. 传递给前端：
//    - Tauri 命令返回 Result<T, String>
//    - DataAnalystError 会自动转换为 String（通过 From<DataAnalystError> for String）
//    - 前端收到的错误是一个可读的字符串消息
//
// ============================================================================
