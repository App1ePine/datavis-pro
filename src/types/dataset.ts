/**
 * 列信息
 */
export interface ColumnInfo {
  /** 列名 */
  name: string;
  /** 数据类型 */
  dtype: string;
  /** 空值数量 */
  null_count: number;
}

/**
 * 数据集元信息
 */
export interface DatasetInfo {
  /** 数据集唯一标识符 */
  id: string;
  /** 文件名 */
  name: string;
  /** 总行数 */
  rows: number;
  /** 列信息 */
  columns: ColumnInfo[];
  /** 原始文件路径 */
  file_path: string;
  /** 导入时间（ISO 8601 格式） */
  imported_at: string;
}

/**
 * 数据集数据（用于分页查询）
 */
export interface DatasetData {
  /** 列名列表 */
  columns: string[];
  /** 数据行（每行是一个值数组） */
  rows: Array<Array<string | number | boolean | null>>;
  /** 总行数 */
  total_rows: number;
}

/**
 * 列统计信息
 */
export interface ColumnStats {
  /** 列名 */
  name: string;
  /** 数据类型 */
  dtype: string;
  /** 总数目 */
  total_count: number;
  /** 缺失值数量 */
  null_count: number;
  /** 唯一值数量 */
  unique_count: number;
  /** 最大值（仅数值类型） */
  max: number | null;
  /** 最小值（仅数值类型） */
  min: number | null;
  /** 平均值（仅数值类型） */
  mean: number | null;
  /** 标准差（仅数值类型） */
  std: number | null;
  /** 25% 分位值（仅数值类型） */
  q25: number | null;
  /** 50% 分位值/中位数（仅数值类型） */
  q50: number | null;
  /** 75% 分位值（仅数值类型） */
  q75: number | null;
  /** 最早时间（仅日期时间类型） */
  min_datetime: string | null;
  /** 最晚时间（仅日期时间类型） */
  max_datetime: string | null;
  /** 时间跨度（仅日期时间类型，单位：天） */
  datetime_range_days: number | null;
  /** True 数量（仅布尔类型） */
  true_count: number | null;
  /** False 数量（仅布尔类型） */
  false_count: number | null;
}
