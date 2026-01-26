// ============================================================================
// models/chart.rs - 图表相关数据模型
// ============================================================================

use serde::{Deserialize, Serialize};

/// 图表类型
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChartType {
    /// 折线图
    Line,
    /// 柱状图
    Bar,
    /// 散点图
    Scatter,
    /// 饼图
    Pie,
    /// 直方图
    Histogram,
}

/// 图表配置请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// 图表类型
    pub chart_type: ChartType,

    /// X 轴列名（折线图、柱状图、散点图使用）
    pub x_column: Option<String>,

    /// Y 轴列名列表（折线图、柱状图、散点图使用，支持多系列）
    pub y_columns: Option<Vec<String>>,

    /// 分类列名（饼图使用）
    pub category_column: Option<String>,

    /// 值列名（饼图使用）
    pub value_column: Option<String>,

    /// 图表标题（可选）
    pub title: Option<String>,

    /// 折线图样式（折线/面积/面积堆积）
    pub line_style: Option<String>,

    /// 双 Y 轴右侧列（折线图使用）
    pub y_axis_right_columns: Option<Vec<String>>,

    /// 直方图列名（直方图使用）
    pub histogram_column: Option<String>,

    /// 直方图分箱数量（直方图使用）
    pub histogram_bins: Option<usize>,
}

/// 图表数据响应（使用 ECharts dataset 格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    /// 图表类型
    pub chart_type: ChartType,

    /// ECharts dataset 格式的数据
    /// 二维数组，第一行是列名，后续行是数据
    /// 例如：[["date", "sales", "profit"], ["2024-01", 100, 50], ...]
    pub dataset: Vec<Vec<serde_json::Value>>,

    /// 数据行数（不包括表头）
    pub data_count: usize,
}
