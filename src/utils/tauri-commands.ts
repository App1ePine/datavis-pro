import { invoke } from '@tauri-apps/api/core';
import type { ColumnStats, DatasetData, DatasetInfo } from '@/types/dataset';
import type { FillStrategy, HistoryEntryInfo } from '@/types/history';

/**
 * 导入 CSV 文件
 */
export async function importCSV(filePath: string): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('import_csv', { filePath });
}

/**
 * 导入 Excel 文件
 */
export async function importExcel(filePath: string, sheetName?: string): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('import_excel', { filePath, sheetName });
}

/**
 * 导入 Parquet 文件
 */
export async function importParquet(filePath: string): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('import_parquet', { filePath });
}

/**
 * 获取当前数据集的分页数据
 */
export async function getCurrentData(offset: number, limit: number): Promise<DatasetData> {
  return await invoke<DatasetData>('get_current_data', {
    offset,
    limit,
  });
}

/**
 * 获取列统计信息
 */
export async function getColumnStats(columnName: string): Promise<ColumnStats> {
  return await invoke<ColumnStats>('get_column_stats', { columnName });
}

/**
 * 导出数据集为 CSV 文件
 */
export async function exportCSV(outputPath: string): Promise<string> {
  return await invoke<string>('export_csv', { outputPath });
}

/**
 * 导出数据集为 Parquet 文件
 */
export async function exportParquet(outputPath: string): Promise<string> {
  return await invoke<string>('export_parquet', { outputPath });
}

/**
 * 清空所有数据和历史
 */
export async function clearData(): Promise<void> {
  return await invoke<void>('clear_data');
}

// ==================== 历史管理命令 ====================

/**
 * 获取操作历史列表
 */
export async function getHistory(): Promise<HistoryEntryInfo[]> {
  return await invoke<HistoryEntryInfo[]>('get_history');
}

/**
 * 获取当前历史索引
 */
export async function getCurrentIndex(): Promise<number | null> {
  return await invoke<number | null>('get_current_index');
}

/**
 * 撤销操作（Undo）
 */
export async function undoOperation(): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('undo_operation');
}

/**
 * 重做操作（Redo）
 */
export async function redoOperation(): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('redo_operation');
}

/**
 * 跳转到指定历史节点
 */
export async function jumpToHistory(entryId: string): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('jump_to_history', { entryId });
}

/**
 * 检查是否可以撤销
 */
export async function canUndo(): Promise<boolean> {
  return await invoke<boolean>('can_undo');
}

/**
 * 检查是否可以重做
 */
export async function canRedo(): Promise<boolean> {
  return await invoke<boolean>('can_redo');
}

/**
 * 重置到初始状态（截断历史栈，只保留第一个节点）
 */
export async function resetToInitial(): Promise<void> {
  return await invoke<void>('reset_to_initial');
}

// ==================== 数据操作命令 ====================

/**
 * 删除包含空值的行
 * @param subset 可选，指定要检查的列名列表
 */
export async function dropNulls(subset?: string[]): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('drop_nulls', { subset });
}

/**
 * 删除全空行（所有列都为空的行）
 */
export async function dropAllNulls(): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('drop_all_nulls');
}

/**
 * 选择指定列（保留选中的列）
 */
export async function selectColumns(columns: string[]): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('select_columns', { columns });
}

/**
 * 删除指定列
 */
export async function dropColumns(columns: string[]): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('drop_columns', { columns });
}

/**
 * 重命名列
 * @param mapping 列名映射，key 为旧列名，value 为新列名
 */
export async function renameColumns(mapping: Record<string, string>): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('rename_columns', { mapping });
}

/**
 * 转换列类型
 * @param mapping 类型映射，key 为列名，value 为目标类型
 */
export async function castTypes(mapping: Record<string, string>): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('cast_types', { mapping });
}

/**
 * 筛选过滤（使用 Polars 表达式）
 * @param expression Polars 表达式字符串，如 "col('age') > 18"
 */
export async function filterData(expression: string): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('filter_data', { expression });
}

/**
 * 填充空值
 */
export async function fillNull(strategy: FillStrategy): Promise<DatasetInfo> {
  return await invoke<DatasetInfo>('fill_null', { strategy });
}
