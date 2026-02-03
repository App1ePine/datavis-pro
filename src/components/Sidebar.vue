<template>
  <div class="sidebar-panel">
    <!-- 标题 -->
    <div class="sidebar-header">
      <el-icon class="header-icon"><Operation /></el-icon>
      <div class="header-text">
        <div class="header-title">工具箱</div>
        <div class="header-subtitle">数据处理工具</div>
      </div>
    </div>

    <!-- Undo/Redo 按钮区域 -->
    <div class="undo-redo-section">
      <el-button-group class="undo-redo-group">
        <el-button :disabled="!dataStore.canUndoFlag" size="default" title="撤销 (Ctrl+Z)" @click="handleUndo">
          <el-icon><Back /></el-icon>
          <span>撤销</span>
        </el-button>
        <el-button :disabled="!dataStore.canRedoFlag" size="default" title="重做 (Ctrl+Shift+Z)" @click="handleRedo">
          <span>重做</span>
          <el-icon><Right /></el-icon>
        </el-button>
      </el-button-group>
    </div>

    <!-- 工具箱折叠面板 -->
    <el-scrollbar class="sidebar-scrollbar">
      <div class="panel-content">
        <el-collapse v-model="activeNames">
          <!-- 数据清洗 -->
          <el-collapse-item name="1">
            <template #title>
              <div class="collapse-title">
                <el-icon class="title-icon"><Filter /></el-icon>
                <span>数据清洗</span>
              </div>
            </template>
            <div class="operation-list">
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showFilterDialog">
                <el-icon><Search /></el-icon>
                <span>筛选过滤</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showDropColumnsDialog">
                <el-icon><Delete /></el-icon>
                <span>删除列</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showSelectColumnsDialog">
                <el-icon><Select /></el-icon>
                <span>选择列</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="handleDropNulls">
                <el-icon><RemoveFilled /></el-icon>
                <span>删除空值行</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="handleDropAllNulls">
                <el-icon><RemoveFilled /></el-icon>
                <span>删除全空行</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showFillNullDialog">
                <el-icon><MagicStick /></el-icon>
                <span>填充空值</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showRenameColumnsDialog">
                <el-icon><Edit /></el-icon>
                <span>更改列名称</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showCastTypesDialog">
                <el-icon><Switch /></el-icon>
                <span>更改列类型</span>
              </el-button>
            </div>
          </el-collapse-item>

          <!-- 数据转换 -->
          <el-collapse-item name="2">
            <template #title>
              <div class="collapse-title">
                <el-icon class="title-icon"><Refresh /></el-icon>
                <span>数据转换</span>
              </div>
            </template>
            <div class="operation-list">
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showPivotDialog">
                <el-icon><Sort /></el-icon>
                <span>长表转宽表</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showUnpivotDialog">
                <el-icon><Sort /></el-icon>
                <span>宽表转长表</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showRollingDialog">
                <el-icon><TrendCharts /></el-icon>
                <span>滑动窗口</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showSortDialog">
                <el-icon><Sort /></el-icon>
                <span>排序</span>
              </el-button>
            </div>
          </el-collapse-item>

          <!-- 可视化 -->
          <el-collapse-item name="3">
            <template #title>
              <div class="collapse-title">
                <el-icon class="title-icon"><PieChart /></el-icon>
                <span>可视化</span>
              </div>
            </template>
            <div class="operation-list">
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showLineChartDialog">
                <el-icon><TrendCharts /></el-icon>
                <span>折线图</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showBarChartDialog">
                <el-icon><Histogram /></el-icon>
                <span>柱状图</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showScatterChartDialog">
                <el-icon><CirclePlus /></el-icon>
                <span>散点图</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showPieChartDialog">
                <el-icon><PieChart /></el-icon>
                <span>饼图</span>
              </el-button>
              <el-button :disabled="!hasData" class="operation-btn" size="small" @click="showHistogramChartDialog">
                <el-icon><Histogram /></el-icon>
                <span>直方图</span>
              </el-button>
            </div>
          </el-collapse-item>
        </el-collapse>
      </div>
    </el-scrollbar>

    <!-- 对话框组件 -->
    <ColumnSelectionDialog
      v-model:visible="columnSelectionVisible"
      :columns="currentColumns"
      :mode="columnSelectionMode"
      @confirm="handleColumnSelectionConfirm"
    />

    <RenameColumnsDialog
      v-model:visible="renameColumnsVisible"
      :columns="currentColumns"
      @confirm="handleRenameColumnsConfirm"
    />

    <CastTypesDialog v-model:visible="castTypesVisible" :columns="currentColumns" @confirm="handleCastTypesConfirm" />

    <FilterDialog v-model:visible="filterVisible" :columns="currentColumns" @confirm="handleFilterConfirm" />

    <FillNullDialog v-model:visible="fillNullVisible" :columns="currentColumns" @confirm="handleFillNullConfirm" />

    <SortDialog v-model:visible="sortVisible" :columns="currentColumns" @confirm="handleSortConfirm" />

    <PivotDialog v-model:visible="pivotVisible" />

    <UnpivotDialog v-model:visible="unpivotVisible" />

    <RollingDialog v-model:visible="rollingVisible" />

    <!-- 图表对话框 -->
    <LineChartDialog v-model="lineChartVisible" />
    <BarChartDialog v-model="barChartVisible" />
    <ScatterChartDialog v-model="scatterChartVisible" />
    <PieChartDialog v-model="pieChartVisible" />
    <HistogramChartDialog v-model="histogramChartVisible" />
  </div>
</template>

<script lang="ts" setup>
import {
  Back,
  CirclePlus,
  Delete,
  Edit,
  Filter,
  Histogram,
  MagicStick,
  Operation,
  PieChart,
  Refresh,
  RemoveFilled,
  Right,
  Search,
  Select,
  Sort,
  Switch,
  TrendCharts,
} from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { computed, ref } from 'vue';
import { useDataStore } from '@/stores/dataStore';
import type { FillStrategy } from '@/types/history';
import BarChartDialog from './chart/BarChartDialog.vue';
import HistogramChartDialog from './chart/HistogramChartDialog.vue';
import LineChartDialog from './chart/LineChartDialog.vue';
import PieChartDialog from './chart/PieChartDialog.vue';
import ScatterChartDialog from './chart/ScatterChartDialog.vue';
import CastTypesDialog from './dialogs/CastTypesDialog.vue';
import ColumnSelectionDialog from './dialogs/ColumnSelectionDialog.vue';
import FillNullDialog from './dialogs/FillNullDialog.vue';
import FilterDialog from './dialogs/FilterDialog.vue';
import PivotDialog from './dialogs/PivotDialog.vue';
import RenameColumnsDialog from './dialogs/RenameColumnsDialog.vue';
import RollingDialog from './dialogs/RollingDialog.vue';
import SortDialog from './dialogs/SortDialog.vue';
import UnpivotDialog from './dialogs/UnpivotDialog.vue';

const dataStore = useDataStore();
const activeNames = ref(['1', '2', '3']);

// 计算属性：是否有数据
const hasData = computed(() => dataStore.hasData);

// 计算属性：当前列信息
const currentColumns = computed(() => {
  return dataStore.currentDataset?.columns || [];
});

// 对话框可见性状态
const columnSelectionVisible = ref(false);
const columnSelectionMode = ref<'select' | 'drop'>('select');
const renameColumnsVisible = ref(false);
const castTypesVisible = ref(false);
const filterVisible = ref(false);
const fillNullVisible = ref(false);
const sortVisible = ref(false);
const pivotVisible = ref(false);
const unpivotVisible = ref(false);
const rollingVisible = ref(false);
const lineChartVisible = ref(false);
const barChartVisible = ref(false);
const scatterChartVisible = ref(false);
const pieChartVisible = ref(false);
const histogramChartVisible = ref(false);

// Undo/Redo 操作
async function handleUndo() {
  try {
    await dataStore.undo();
    ElMessage.success('已撤销上一步操作');
  } catch (e) {
    console.error('撤销失败:', e);
    ElMessage.error(`撤销失败: ${e}`);
  }
}

async function handleRedo() {
  try {
    await dataStore.redo();
    ElMessage.success('已重做操作');
  } catch (e) {
    console.error('重做失败:', e);
    ElMessage.error(`重做失败: ${e}`);
  }
}

// 数据操作：删除空值行
async function handleDropNulls() {
  try {
    await ElMessageBox.confirm(
      '此操作将删除包含任何空值的行。例如：如果某行有一列为空，整行都会被删除。是否继续？',
      '确认删除空值行',
      {
        confirmButtonText: '确定删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );

    await dataStore.dropNulls();
    ElMessage.success('已删除包含空值的行');
  } catch (e) {
    if (e !== 'cancel') {
      console.error('删除空值行失败:', e);
      ElMessage.error(`删除空值行失败: ${e}`);
    }
  }
}

// 数据操作：删除全空行
async function handleDropAllNulls() {
  try {
    await ElMessageBox.confirm(
      '此操作将删除所有列都为空的行。例如：只有当整行都是空值时才会被删除，部分有值的行会保留。是否继续？',
      '确认删除全空行',
      {
        confirmButtonText: '确定删除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    );

    await dataStore.dropAllNulls();
    ElMessage.success('已删除全空行');
  } catch (e) {
    if (e !== 'cancel') {
      console.error('删除全空行失败:', e);
      ElMessage.error(`删除全空行失败: ${e}`);
    }
  }
}

// 显示对话框
function showFilterDialog() {
  filterVisible.value = true;
}

function showDropColumnsDialog() {
  columnSelectionMode.value = 'drop';
  columnSelectionVisible.value = true;
}

function showSelectColumnsDialog() {
  columnSelectionMode.value = 'select';
  columnSelectionVisible.value = true;
}

function showFillNullDialog() {
  fillNullVisible.value = true;
}

function showSortDialog() {
  sortVisible.value = true;
}

function showRenameColumnsDialog() {
  renameColumnsVisible.value = true;
}

function showCastTypesDialog() {
  castTypesVisible.value = true;
}

function showPivotDialog() {
  pivotVisible.value = true;
}

function showUnpivotDialog() {
  unpivotVisible.value = true;
}

function showRollingDialog() {
  rollingVisible.value = true;
}

function showLineChartDialog() {
  lineChartVisible.value = true;
}

function showBarChartDialog() {
  barChartVisible.value = true;
}

function showScatterChartDialog() {
  scatterChartVisible.value = true;
}

function showPieChartDialog() {
  pieChartVisible.value = true;
}

function showHistogramChartDialog() {
  histogramChartVisible.value = true;
}

// 对话框确认处理
async function handleColumnSelectionConfirm(columns: string[]) {
  try {
    if (columnSelectionMode.value === 'select') {
      await dataStore.selectColumns(columns);
      ElMessage.success(`已选择 ${columns.length} 列`);
    } else {
      await dataStore.dropColumns(columns);
      ElMessage.success(`已删除 ${columns.length} 列`);
    }
  } catch (e) {
    console.error('列操作失败:', e);
    ElMessage.error(`列操作失败: ${e}`);
  }
}

async function handleRenameColumnsConfirm(mapping: Record<string, string>) {
  try {
    await dataStore.renameColumns(mapping);
    const count = Object.keys(mapping).length;
    ElMessage.success(`已重命名 ${count} 列`);
  } catch (e) {
    console.error('重命名列失败:', e);
    ElMessage.error(`重命名列失败: ${e}`);
  }
}

async function handleCastTypesConfirm(mapping: Record<string, string>) {
  try {
    await dataStore.castTypes(mapping);
    const count = Object.keys(mapping).length;
    ElMessage.success(`已转换 ${count} 列的类型`);
  } catch (e) {
    console.error('转换列类型失败:', e);
    ElMessage.error(`转换列类型失败: ${e}`);
  }
}

async function handleFilterConfirm(expression: string) {
  try {
    await dataStore.filterData(expression);
    ElMessage.success('数据筛选成功');
  } catch (e) {
    console.error('筛选数据失败:', e);
    ElMessage.error(`筛选数据失败: ${e}`);
  }
}

async function handleFillNullConfirm(strategy: FillStrategy) {
  try {
    await dataStore.fillNull(strategy);
    const columnText = strategy.columns ? `${strategy.columns.length} 列` : '所有列';
    ElMessage.success(`已对 ${columnText} 执行填充操作`);
  } catch (e) {
    console.error('填充空值失败:', e);
    ElMessage.error(`填充空值失败: ${e}`);
  }
}

async function handleSortConfirm(payload: { column: string; descending: boolean; nullsLast: boolean }) {
  try {
    await dataStore.sortData(payload.column, payload.descending, payload.nullsLast);
    ElMessage.success('排序完成');
  } catch (e) {
    console.error('排序失败:', e);
    ElMessage.error(`排序失败: ${e}`);
  }
}
</script>

<style scoped>
.sidebar-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: #ffffff;
}

.sidebar-header {
  padding: 20px;
  background-color: #ffffff;
  border-bottom: 1px solid #e4e7ed;
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-icon {
  font-size: 28px;
  line-height: 1;
}

.header-text {
  flex: 1;
}

.header-title {
  font-size: 16px;
  font-weight: 700;
  color: #303133;
  line-height: 1.3;
  margin-bottom: 2px;
}

.header-subtitle {
  font-size: 12px;
  color: #909399;
  line-height: 1;
}

.undo-redo-section {
  padding: 4px;
  background-color: #f5f7fa;
  border-bottom: 1px solid #dcdfe6;
}

.undo-redo-group {
  width: 100%;
  display: flex;
}

.undo-redo-group .el-button {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.history-info {
  text-align: center;
  margin-top: 8px;
}

.history-text {
  font-size: 12px;
  color: #909399;
  font-family: 'Courier New', monospace;
}

.sidebar-scrollbar {
  flex: 1;
  height: 0;
}

.panel-content {
  padding: 16px;
}

.operation-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
}

.operation-btn {
  width: 100%;
  justify-content: flex-start;
  margin-left: 0 !important;
  display: flex;
  align-items: center;
  gap: 8px;
}

:deep(.el-collapse) {
  border: none;
}

:deep(.el-collapse-item) {
  margin-bottom: 12px;
  background: white;
  border-radius: 8px;
  overflow: hidden;
  border: 1px solid #ebeef5;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.04);
}

:deep(.el-collapse-item__header) {
  background: #fafafa;
  border: none;
  padding: 0 16px;
  font-size: 14px;
  font-weight: 600;
  height: 48px;
  color: #303133;
}

.collapse-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.title-icon {
  font-size: 18px;
  color: #409eff;
}

:deep(.el-collapse-item__wrap) {
  border: none;
  background: white;
}

:deep(.el-collapse-item__content) {
  padding: 0;
}
</style>
