import type { DatasetInfo } from './dataset';

/**
 * 填充策略
 */
export interface FillStrategy {
  /** 策略类型 */
  strategy: 'forward' | 'backward' | 'min' | 'max' | 'mean' | 'zero' | 'one';
  /** 要填充的列（必须指定） */
  columns: string[];
}

/**
 * 操作类型（与 Rust 端保持一致）
 */
export type OperationType =
  | { type: 'Import'; params: { file_path: string } }
  | { type: 'Unpivot'; params: { id_vars: string[]; value_vars: string[] } }
  | { type: 'Pivot'; params: { index: string[]; columns: string[]; values: string[] } }
  | { type: 'DropNulls'; params: { subset?: string[] } }
  | { type: 'DropAllNulls'; params: null }
  | { type: 'SelectColumns'; params: { columns: string[] } }
  | { type: 'DropColumns'; params: { columns: string[] } }
  | { type: 'RenameColumns'; params: { mapping: Record<string, string> } }
  | { type: 'CastTypes'; params: { mapping: Record<string, string> } }
  | { type: 'Filter'; params: { expression: string } }
  | { type: 'FillNull'; params: { strategy: FillStrategy } }
  | { type: 'RollingAverage'; params: { column: string; window_size: number } }
  | { type: 'RollingMedian'; params: { column: string; window_size: number } };

/**
 * 历史条目信息
 */
export interface HistoryEntryInfo {
  /** 历史条目唯一标识符 */
  id: string;
  /** 执行的操作 */
  operation: OperationType;
  /** 操作后的数据集元信息 */
  metadata: DatasetInfo;
  /** 操作时间戳（ISO 8601 格式） */
  timestamp: string;
  /** 操作描述（人类可读） */
  description: string;
}
