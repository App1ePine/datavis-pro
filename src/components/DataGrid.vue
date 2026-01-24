<template>
  <div class="data-grid-container">
    <ag-grid-vue
      v-if="dataStore.currentDataset"
      style="width: 100%; height: 100%"
      :theme="myTheme"
      :columnDefs="columnDefs"
      :defaultColDef="defaultColDef"
      :rowModelType="'infinite'"
      :cacheBlockSize="100"
      :cacheOverflowSize="2"
      :maxConcurrentDatasourceRequests="1"
      :infiniteInitialRowCount="1000"
      :maxBlocksInCache="10"
      :enableCellTextSelection="true"
      :ensureDomOrder="true"
      @grid-ready="onGridReady"
      @cell-clicked="onCellClicked"
    />
    <el-empty v-else description="暂无数据" />
  </div>
</template>

<script setup lang="ts">
import type {
  CellClickedEvent,
  GridApi,
  GridReadyEvent,
  IGetRowsParams,
  ValueFormatterParams,
} from 'ag-grid-community';
import { AllCommunityModule, ModuleRegistry, themeQuartz } from 'ag-grid-community';
import { AgGridVue } from 'ag-grid-vue3';
import { computed, watch } from 'vue';
import { useDataStore } from '@/stores/dataStore';

// 注册 AG Grid 社区模块
ModuleRegistry.registerModules([AllCommunityModule]);

// 配置主题
const myTheme = themeQuartz.withParams({
  accentColor: '#409eff',
  borderRadius: 4,
  spacing: 8,
});

const dataStore = useDataStore();

const columnDefs = computed(() => {
  if (!dataStore.currentData || !dataStore.currentDataset) return [];

  // 创建列类型映射表
  const columnTypeMap = new Map<string, string>();
  dataStore.currentDataset.columns.forEach((col) => {
    columnTypeMap.set(col.name, col.dtype);
  });

  return dataStore.currentData.columns.map((col) => {
    const dtype = columnTypeMap.get(col) || '';

    // 根据数据类型设置列宽度
    let minWidth = 150; // 默认宽度
    if (dtype.includes('Date') || dtype.includes('Datetime')) {
      minWidth = 180; // 日期时间类型
    } else if (dtype.includes('Int') || dtype.includes('Float') || dtype.includes('UInt')) {
      minWidth = 120; // 数值类型
    }

    return {
      field: col,
      headerName: col,
      sortable: false,
      filter: false,
      resizable: true,
      minWidth,
      // 自动格式化数值列，保留4位小数
      valueFormatter: (params: ValueFormatterParams) => {
        if (typeof params.value === 'number') {
          // 如果是整数，不显示小数点
          if (Number.isInteger(params.value)) {
            return params.value;
          }
          // 如果是浮点数，保留4位小数
          return params.value.toFixed(4);
        }
        return params.value;
      },
    };
  });
});

const defaultColDef = {
  resizable: true,
  sortable: false, // 禁用排序
  filter: false, // 禁用筛选
  minWidth: 150,
  flex: 1,
};

let gridApi: GridApi | null = null;

function onGridReady(params: GridReadyEvent) {
  gridApi = params.api;

  // 设置无限滚动数据源
  const datasource = {
    rowCount: undefined, // 未知总行数，会动态更新
    getRows: async (params: IGetRowsParams) => {
      const startRow = params.startRow;
      const endRow = params.endRow;
      const limit = endRow - startRow;

      try {
        if (!dataStore.currentDataset) {
          params.failCallback();
          return;
        }

        // 从后端获取数据
        const data = await dataStore.loadCurrentData(startRow, limit);

        // 转换数据格式
        const rows = data.rows.map((row) => {
          const obj: Record<string, string | number | boolean | null> = {};
          data.columns.forEach((col: string, i: number) => {
            obj[col] = row[i];
          });
          return obj;
        });

        // 通知 ag-Grid 数据加载完成
        params.successCallback(rows, data.total_rows);
      } catch (error) {
        console.error('加载数据失败:', error);
        params.failCallback();
      }
    },
  };

  params.api.setGridOption('datasource', datasource);
}

function onCellClicked(event: CellClickedEvent) {
  // 获取点击的列名
  const columnName = event.column?.getColId();
  if (!columnName) return;

  // 检查是否已经显示了该列的统计信息，避免重复计算
  if (dataStore.currentColumnStats?.name === columnName) {
    return; // 已经是当前列，不需要重新计算
  }

  // 触发获取列统计信息
  if (dataStore.currentDataset) {
    dataStore.loadColumnStats(columnName);
  }
}

// 监听当前数据集变化，重新设置数据源
watch(
  () => dataStore.currentDataset?.id,
  (newId) => {
    if (newId && gridApi) {
      // 重置网格，触发重新加载
      gridApi.setGridOption('datasource', gridApi.getGridOption('datasource'));
    }
  }
);
</script>

<style scoped>
.data-grid-container {
  width: 100%;
  height: 100%;
  position: relative;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  background-color: rgba(255, 255, 255, 0.9);
  z-index: 1000;
}

.loading-overlay p {
  margin-top: 16px;
  font-size: 14px;
  color: #606266;
}
</style>
