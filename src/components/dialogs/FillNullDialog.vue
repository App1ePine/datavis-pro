<template>
  <el-dialog
    :model-value="visible"
    title="填充空值 - Fill Null Values"
    width="600px"
    @update:model-value="(val: boolean) => $emit('update:visible', val)"
    @close="handleClose"
    align-center
  >
    <div class="fill-null-content">
      <!-- 策略选择 -->
      <div class="strategy-section">
        <h4 class="section-title">填充策略</h4>
        <el-radio-group v-model="strategy" class="strategy-group">
          <el-radio value="forward" border>
            <div class="radio-content">
              <span class="radio-label">向前填充</span>
              <span class="radio-desc">使用前一个非空值填充</span>
            </div>
          </el-radio>
          <el-radio value="backward" border>
            <div class="radio-content">
              <span class="radio-label">向后填充</span>
              <span class="radio-desc">使用后一个非空值填充</span>
            </div>
          </el-radio>
          <el-radio value="min" border>
            <div class="radio-content">
              <span class="radio-label">最小值填充</span>
              <span class="radio-desc">使用列的最小值填充</span>
            </div>
          </el-radio>
          <el-radio value="max" border>
            <div class="radio-content">
              <span class="radio-label">最大值填充</span>
              <span class="radio-desc">使用列的最大值填充</span>
            </div>
          </el-radio>
          <el-radio value="mean" border>
            <div class="radio-content">
              <span class="radio-label">均值填充</span>
              <span class="radio-desc">使用列的平均值填充</span>
            </div>
          </el-radio>
          <el-radio value="zero" border>
            <div class="radio-content">
              <span class="radio-label">0 值填充</span>
              <span class="radio-desc">使用 0 填充</span>
            </div>
          </el-radio>
          <el-radio value="one" border>
            <div class="radio-content">
              <span class="radio-label">1 值填充</span>
              <span class="radio-desc">使用 1 填充</span>
            </div>
          </el-radio>
        </el-radio-group>
      </div>

      <!-- 列选择 -->
      <div class="columns-section">
        <h4 class="section-title">
          选择要填充的列
          <span class="subtitle">（必须选择至少一列）</span>
        </h4>

        <!-- 搜索框 -->
        <el-input v-model="searchText" placeholder="搜索列名..." :prefix-icon="Search" clearable class="search-input" />

        <!-- 列选择列表 -->
        <div class="columns-list">
          <el-checkbox-group v-model="selectedColumns">
            <div v-for="col in filteredColumns" :key="col.name" class="column-item">
              <el-checkbox :value="col.name">
                <span class="column-name">{{ col.name }}</span>
                <el-tag size="small" type="info" class="column-type">{{ col.dtype }}</el-tag>
                <el-tag v-if="col.null_count > 0" size="small" type="warning" class="null-tag">
                  {{ col.null_count }} 空值
                </el-tag>
              </el-checkbox>
            </div>
          </el-checkbox-group>
        </div>

        <!-- 快捷选择按钮 -->
        <div class="quick-actions">
          <el-button size="small" @click="selectAll">全选</el-button>
          <el-button size="small" @click="selectNone">清空</el-button>
          <el-button size="small" @click="selectWithNulls">仅选择有空值的列</el-button>
        </div>
      </div>

      <!-- 提示信息 -->
      <el-alert
        v-if="selectedColumns.length > 0"
        :title="`将对 ${selectedColumns.length} 列执行填充操作`"
        type="info"
        :closable="false"
        show-icon
      />
    </div>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" @click="handleConfirm" :disabled="selectedColumns.length === 0"> 确认填充 </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { Search } from '@element-plus/icons-vue';
import { computed, ref } from 'vue';
import type { ColumnInfo } from '@/types/dataset';
import type { FillStrategy } from '@/types/history';

interface Props {
  visible: boolean;
  columns: ColumnInfo[];
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'update:visible': [value: boolean];
  confirm: [strategy: FillStrategy];
}>();

// 状态
const strategy = ref<'forward' | 'backward' | 'min' | 'max' | 'mean' | 'zero' | 'one'>('forward');
const selectedColumns = ref<string[]>([]);
const searchText = ref('');

// 过滤后的列
const filteredColumns = computed(() => {
  if (!searchText.value) return props.columns;
  const search = searchText.value.toLowerCase();
  return props.columns.filter((col) => col.name.toLowerCase().includes(search));
});

// 快捷选择
function selectAll() {
  selectedColumns.value = filteredColumns.value.map((col) => col.name);
}

function selectNone() {
  selectedColumns.value = [];
}

function selectWithNulls() {
  selectedColumns.value = filteredColumns.value.filter((col) => col.null_count > 0).map((col) => col.name);
}

// 关闭对话框
function handleClose() {
  emit('update:visible', false);
  // 重置状态
  strategy.value = 'forward';
  selectedColumns.value = [];
  searchText.value = '';
}

// 确认操作
function handleConfirm() {
  const fillStrategy: FillStrategy = {
    strategy: strategy.value,
    columns: selectedColumns.value,
  };

  emit('confirm', fillStrategy);
  handleClose();
}
</script>

<style scoped>
.fill-null-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #1d2129;
  margin: 0 0 12px 0;
  letter-spacing: 0.01em;
}

.subtitle {
  font-size: 12px;
  font-weight: normal;
  color: #86909c;
  margin-left: 8px;
}

/* 策略选择 */
.strategy-section {
  padding-bottom: 20px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.06);
}

.strategy-group {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 12px;
  width: 100%;
}

.strategy-group :deep(.el-radio) {
  margin-right: 0;
  margin-left: 0 !important;
  width: 100%;
  height: auto;
  padding: 12px 16px;
}

.strategy-group :deep(.el-radio.is-bordered) {
  border-radius: 10px;
  border-color: rgba(0, 0, 0, 0.06);
  transition: all 0.2s ease;
}

.strategy-group :deep(.el-radio.is-bordered:hover) {
  border-color: #409eff;
  box-shadow: 0 2px 8px rgba(64, 158, 255, 0.1);
}

.strategy-group :deep(.el-radio.is-bordered.is-checked) {
  border-color: #409eff;
  background-color: rgba(64, 158, 255, 0.04);
}

.radio-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.radio-label {
  font-size: 14px;
  font-weight: 500;
  color: #1d2129;
}

.radio-desc {
  font-size: 12px;
  color: #86909c;
}

/* 自定义值 */
.custom-value-section {
  padding-bottom: 20px;
  border-bottom: 1px solid rgba(0, 0, 0, 0.06);
}

/* 列选择 */
.columns-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.search-input {
  margin-bottom: 8px;
}

.columns-list {
  max-height: 240px;
  overflow-y: auto;
  border: 1px solid rgba(0, 0, 0, 0.06);
  border-radius: 8px;
  padding: 12px;
  background-color: #fafbfc;
}

.column-item {
  padding: 8px 0;
  border-bottom: 1px solid rgba(0, 0, 0, 0.04);
  transition: all 0.15s ease;
}

.column-item:last-child {
  border-bottom: none;
}

.column-item :deep(.el-checkbox) {
  width: 100%;
  display: flex;
  align-items: center;
}

.column-item :deep(.el-checkbox__label) {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
}

.column-name {
  font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
  font-size: 13px;
  color: #1d2129;
  flex: 1;
}

.column-type {
  margin-left: auto;
}

.null-tag {
  margin-left: 8px;
}

.quick-actions {
  display: flex;
  gap: 8px;
  padding-top: 8px;
  border-top: 1px solid rgba(0, 0, 0, 0.06);
}
</style>
