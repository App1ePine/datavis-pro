<template>
  <div class="sidebar-panel">
    <!-- 标题 -->
    <div class="sidebar-title">操作面板</div>

    <!-- Undo/Redo 按钮区域 -->
    <div class="undo-redo-section">
      <el-button-group class="undo-redo-group">
        <el-button size="small" :disabled="!dataStore.canUndoFlag" @click="handleUndo" title="撤销 (Ctrl+Z)">
          ↶ 撤销
        </el-button>
        <el-button size="small" :disabled="!dataStore.canRedoFlag" @click="handleRedo" title="重做 (Ctrl+Shift+Z)">
          ↷ 重做
        </el-button>
      </el-button-group>
    </div>

    <!-- 工具箱折叠面板 -->
    <el-scrollbar class="sidebar-scrollbar">
      <div class="panel-content">
        <el-collapse v-model="activeNames">
          <!-- 数据清洗 -->
          <el-collapse-item title="数据清洗" name="1">
            <div class="operation-list">
              <el-button size="small" class="operation-btn" @click="showFilterDialog" :disabled="!hasData">
                🔍 筛选过滤
              </el-button>
              <el-button size="small" class="operation-btn" @click="showDropColumnsDialog" :disabled="!hasData">
                🚮 删除列
              </el-button>
              <el-button size="small" class="operation-btn" @click="showSelectColumnsDialog" :disabled="!hasData">
                ✅ 选择列
              </el-button>
              <el-button size="small" class="operation-btn" @click="handleDropNulls" :disabled="!hasData">
                🗑️ 删除空值行
              </el-button>
              <el-button size="small" class="operation-btn" @click="handleDropAllNulls" :disabled="!hasData">
                🗑️ 删除全空行
              </el-button>
              <el-button size="small" class="operation-btn" @click="showFillNullDialog" :disabled="!hasData">
                ✨ 填充空值
              </el-button>
              <el-button size="small" class="operation-btn" @click="showRenameColumnsDialog" :disabled="!hasData">
                📝 更改列名称
              </el-button>
              <el-button size="small" class="operation-btn" @click="showCastTypesDialog" :disabled="!hasData">
                🆔 更改列类型
              </el-button>
            </div>
          </el-collapse-item>

          <!-- 数据转换 -->
          <el-collapse-item title="数据转换" name="2">
            <div class="operation-list">
              <el-button size="small" class="operation-btn" @click="handlePlaceholder('横转纵')" :disabled="!hasData">
                🔃 横转纵
              </el-button>
              <el-button size="small" class="operation-btn" @click="handlePlaceholder('纵转横')" :disabled="!hasData">
                🔄 纵转横
              </el-button>
              <el-button size="small" class="operation-btn" @click="handlePlaceholder('排序')" :disabled="!hasData">
                📶 排序
              </el-button>
            </div>
          </el-collapse-item>

          <!-- 可视化 -->
          <el-collapse-item title="可视化" name="3">
            <div class="operation-list">
              <el-button size="small" class="operation-btn" @click="handlePlaceholder('趋势图')" :disabled="!hasData">
                📈 趋势图
              </el-button>
              <el-button size="small" class="operation-btn" @click="handlePlaceholder('直方图')" :disabled="!hasData">
                📊 直方图
              </el-button>
              <el-button size="small" class="operation-btn" @click="handlePlaceholder('散点图')" :disabled="!hasData">
                🔵 散点图
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
  </div>
</template>

<script setup lang="ts">
import { ElMessage, ElMessageBox } from 'element-plus';
import { computed, ref } from 'vue';
import { useDataStore } from '@/stores/dataStore';
import type { FillStrategy } from '@/types/history';
import CastTypesDialog from './dialogs/CastTypesDialog.vue';
import ColumnSelectionDialog from './dialogs/ColumnSelectionDialog.vue';
import FillNullDialog from './dialogs/FillNullDialog.vue';
import FilterDialog from './dialogs/FilterDialog.vue';
import RenameColumnsDialog from './dialogs/RenameColumnsDialog.vue';

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

function showRenameColumnsDialog() {
  renameColumnsVisible.value = true;
}

function showCastTypesDialog() {
  castTypesVisible.value = true;
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

// 占位符函数（未实现的功能）
function handlePlaceholder(feature: string) {
  ElMessage.info(`${feature}功能开发中...`);
}
</script>

<style scoped>
.sidebar-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: #f5f7fa;
}

.sidebar-title {
  padding: 12px 16px;
  background-color: #e4e7ed;
  border-bottom: 1px solid #dcdfe6;
  font-weight: 500;
  color: #606266;
  font-size: 14px;
}

.undo-redo-section {
  padding: 12px 16px;
  background-color: #ffffff;
  border-bottom: 1px solid #e4e7ed;
}

.undo-redo-group {
  width: 100%;
  display: flex;
}

.undo-redo-group .el-button {
  flex: 1;
}

.sidebar-scrollbar {
  flex: 1;
  height: 0;
}

.panel-content {
  padding: 8px;
}

.operation-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 4px;
}

.operation-btn {
  width: 100%;
  justify-content: flex-start;
  margin-left: 0 !important;
}

:deep(.el-collapse) {
  border: none;
}

:deep(.el-collapse-item) {
  margin-bottom: 8px;
  background: white;
  border-radius: 4px;
  overflow: hidden;
}

:deep(.el-collapse-item__header) {
  background: white;
  border: none;
  padding: 0 12px;
  font-size: 13px;
  font-weight: 500;
  height: 40px;
}

:deep(.el-collapse-item__wrap) {
  border: none;
  background: white;
}

:deep(.el-collapse-item__content) {
  padding: 8px 12px 12px;
}
</style>
