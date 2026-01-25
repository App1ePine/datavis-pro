import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import type { ColumnStats, DatasetData } from '@/types/dataset';
import type { FillStrategy, HistoryEntryInfo } from '@/types/history';
import * as commands from '@/utils/tauri-commands';

export const useDataStore = defineStore('data', () => {
  // ==================== 状态 ====================

  /** 操作历史列表 */
  const history = ref<HistoryEntryInfo[]>([]);

  /** 当前历史索引（null 表示没有数据） */
  const currentIndex = ref<number | null>(null);

  /** 当前显示的数据（分页） */
  const currentData = ref<DatasetData | null>(null);

  /** 当前列统计信息 */
  const currentColumnStats = ref<ColumnStats | null>(null);

  /** 加载状态 */
  const loading = ref(false);

  /** 错误信息 */
  const error = ref<string | null>(null);

  /** 是否可以撤销 */
  const canUndoFlag = ref(false);

  /** 是否可以重做 */
  const canRedoFlag = ref(false);

  // ==================== 计算属性 ====================

  /** 当前数据集元信息 */
  const currentDataset = computed(() => {
    if (currentIndex.value === null || history.value.length === 0) {
      return null;
    }
    return history.value[currentIndex.value]?.metadata || null;
  });

  /** 当前数据集元信息 */
  const currentInfo = computed(() => currentDataset.value);

  /** 是否有数据 */
  const hasData = computed(() => currentDataset.value !== null);

  // ==================== 历史管理 ====================

  /**
   * 加载历史列表
   */
  async function loadHistory() {
    try {
      history.value = await commands.getHistory();
      currentIndex.value = await commands.getCurrentIndex();
      await refreshUndoRedoState();
    } catch (e) {
      error.value = String(e);
      console.error('加载历史失败:', e);
      throw e;
    }
  }

  /**
   * 刷新 Undo/Redo 状态
   */
  async function refreshUndoRedoState() {
    try {
      canUndoFlag.value = await commands.canUndo();
      canRedoFlag.value = await commands.canRedo();
    } catch (e) {
      console.error('刷新 Undo/Redo 状态失败:', e);
    }
  }

  /**
   * 撤销操作
   */
  async function undo() {
    loading.value = true;
    error.value = null;
    try {
      await commands.undoOperation();
      await loadHistory();
      // 重新加载当前数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 重做操作
   */
  async function redo() {
    loading.value = true;
    error.value = null;
    try {
      await commands.redoOperation();
      await loadHistory();
      // 重新加载当前数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 跳转到指定历史节点
   */
  async function jumpTo(entryId: string) {
    loading.value = true;
    error.value = null;
    try {
      await commands.jumpToHistory(entryId);
      await loadHistory();
      // 重新加载当前数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 重置到初始状态（刚导入时的数据）
   */
  async function resetToInitial() {
    loading.value = true;
    error.value = null;
    try {
      await commands.resetToInitial();
      await loadHistory();
      // 重新加载当前数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 清空所有数据和历史
   */
  async function clearData() {
    loading.value = true;
    error.value = null;
    try {
      await commands.clearData();
      // 清空本地状态
      history.value = [];
      currentIndex.value = null;
      currentData.value = null;
      currentColumnStats.value = null;
      canUndoFlag.value = false;
      canRedoFlag.value = false;
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  // ==================== 数据加载 ====================

  /**
   * 加载当前数据集的数据（用于无限滚动）
   */
  async function loadCurrentData(offset = 0, limit = 100) {
    if (!currentDataset.value) {
      throw new Error('没有当前数据集');
    }

    error.value = null;
    try {
      const data = await commands.getCurrentData(offset, limit);
      // 只在第一次加载时更新 currentData（用于获取列定义）
      if (offset === 0) {
        currentData.value = data;
      }
      return data;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  /**
   * 刷新当前数据（用于操作后重新加载）
   */
  async function refreshCurrentData() {
    await loadHistory();
    if (currentDataset.value) {
      await loadCurrentData(0, 100);
    }
  }

  /**
   * 加载列统计信息
   */
  async function loadColumnStats(columnName: string) {
    if (!currentDataset.value) {
      throw new Error('没有当前数据集');
    }

    try {
      currentColumnStats.value = await commands.getColumnStats(columnName);
    } catch (e) {
      error.value = String(e);
      console.error('加载列统计信息失败:', e);
      throw e;
    }
  }

  // ==================== 文件导入 ====================

  /**
   * 导入 CSV 文件
   */
  async function importCSV(filePath: string) {
    loading.value = true;
    error.value = null;
    try {
      await commands.importCSV(filePath);
      await loadHistory();
      // 加载第一批数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 导入 Excel 文件
   */
  async function importExcel(filePath: string, sheetName?: string) {
    loading.value = true;
    error.value = null;
    try {
      await commands.importExcel(filePath, sheetName);
      await loadHistory();
      // 加载第一批数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 导入 Parquet 文件
   */
  async function importParquet(filePath: string) {
    loading.value = true;
    error.value = null;
    try {
      await commands.importParquet(filePath);
      await loadHistory();
      // 加载第一批数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  // ==================== 数据导出 ====================

  /**
   * 导出数据集为 CSV
   */
  async function exportDataset(outputPath: string) {
    if (!currentDataset.value) {
      throw new Error('没有当前数据集');
    }

    loading.value = true;
    error.value = null;
    try {
      return await commands.exportCSV(outputPath);
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 导出数据集为 Parquet
   */
  async function exportDatasetParquet(outputPath: string) {
    if (!currentDataset.value) {
      throw new Error('没有当前数据集');
    }

    loading.value = true;
    error.value = null;
    try {
      return await commands.exportParquet(outputPath);
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  // ==================== 数据操作 ====================

  /**
   * 删除包含空值的行
   */
  async function dropNulls(subset?: string[]) {
    loading.value = true;
    error.value = null;
    try {
      await commands.dropNulls(subset);
      await loadHistory();
      // 重新加载数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 删除全空行
   */
  async function dropAllNulls() {
    loading.value = true;
    error.value = null;
    try {
      await commands.dropAllNulls();
      await loadHistory();
      // 重新加载数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 选择列（保留选中的列）
   */
  async function selectColumns(columns: string[]) {
    loading.value = true;
    error.value = null;
    try {
      await commands.selectColumns(columns);
      await loadHistory();
      // 重新加载数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 删除列
   */
  async function dropColumns(columns: string[]) {
    loading.value = true;
    error.value = null;
    try {
      await commands.dropColumns(columns);
      await loadHistory();
      // 重新加载数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 重命名列
   */
  async function renameColumns(mapping: Record<string, string>) {
    loading.value = true;
    error.value = null;
    try {
      await commands.renameColumns(mapping);
      await loadHistory();
      // 重新加载数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 转换列类型
   */
  async function castTypes(mapping: Record<string, string>) {
    loading.value = true;
    error.value = null;
    try {
      await commands.castTypes(mapping);
      await loadHistory();
      // 重新加载数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 筛选过滤
   */
  async function filterData(expression: string) {
    loading.value = true;
    error.value = null;
    try {
      await commands.filterData(expression);
      await loadHistory();
      // 重新加载数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 填充空值
   */
  async function fillNull(strategy: FillStrategy) {
    loading.value = true;
    error.value = null;
    try {
      await commands.fillNull(strategy);
      await loadHistory();
      // 重新加载数据
      if (currentDataset.value) {
        await loadCurrentData(0, 100);
      }
    } catch (e) {
      error.value = String(e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  // ==================== 返回 ====================

  return {
    // 状态
    history,
    currentIndex,
    currentData,
    currentColumnStats,
    loading,
    error,
    canUndoFlag,
    canRedoFlag,

    // 计算属性
    currentDataset,
    currentInfo,
    hasData,

    // 历史管理
    loadHistory,
    refreshUndoRedoState,
    undo,
    redo,
    jumpTo,
    resetToInitial,
    clearData,

    // 数据加载
    loadCurrentData,
    refreshCurrentData,
    loadColumnStats,

    // 文件导入
    importCSV,
    importExcel,
    importParquet,

    // 数据导出
    exportDataset,
    exportDatasetParquet,

    // 数据操作
    dropNulls,
    dropAllNulls,
    selectColumns,
    dropColumns,
    renameColumns,
    castTypes,
    filterData,
    fillNull,
  };
});
