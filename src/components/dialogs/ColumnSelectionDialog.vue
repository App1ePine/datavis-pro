<template>
  <el-dialog :model-value="visible" :title="dialogTitle" width="600px" @close="handleClose" align-center>
    <!-- 搜索和操作栏 -->
    <div class="dialog-toolbar">
      <el-input v-model="searchText" placeholder="搜索列名..." clearable style="width: 300px">
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
      <div class="toolbar-actions">
        <el-button size="small" @click="selectAll">全选</el-button>
        <el-button size="small" @click="selectNone">反选</el-button>
      </div>
    </div>

    <!-- 列列表 -->
    <div class="column-list">
      <el-scrollbar>
        <el-checkbox-group v-model="selectedColumns">
          <div v-for="column in filteredColumns" :key="column.name" class="column-item">
            <el-checkbox :label="column.name">
              <div class="checkbox-content">
                <span class="column-name">{{ column.name }}</span>
                <el-tag size="small" type="info" class="column-type">
                  {{ column.dtype }}
                </el-tag>
              </div>
            </el-checkbox>
          </div>
        </el-checkbox-group>
        <el-empty v-if="filteredColumns.length === 0" description="没有找到匹配的列" :image-size="80" />
      </el-scrollbar>
    </div>

    <!-- 底部信息 -->
    <div class="dialog-footer-info">
      <span class="info-text"> 已选择 {{ selectedColumns.length }} / {{ columns.length }} 列 </span>
    </div>

    <!-- 操作按钮 -->
    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" :disabled="isConfirmDisabled" @click="handleConfirm"> 确认操作 </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { Search } from '@element-plus/icons-vue';
import { computed, ref, watch } from 'vue';
import type { ColumnInfo } from '@/types/dataset';

interface Props {
  visible: boolean;
  columns: ColumnInfo[];
  mode: 'select' | 'drop'; // 'select' 表示选择列，'drop' 表示删除列
}

interface Emits {
  (e: 'update:visible', value: boolean): void;
  (e: 'confirm', columns: string[]): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// 搜索文本
const searchText = ref('');

// 选中的列
const selectedColumns = ref<string[]>([]);

// 对话框标题
const dialogTitle = computed(() => {
  return props.mode === 'select' ? '选择列 - Select Columns' : '删除列 - Drop Columns';
});

// 过滤后的列
const filteredColumns = computed(() => {
  if (!searchText.value) {
    return props.columns;
  }
  const search = searchText.value.toLowerCase();
  return props.columns.filter((col) => col.name.toLowerCase().includes(search));
});

// 全选
function selectAll() {
  selectedColumns.value = filteredColumns.value.map((col) => col.name);
}

// 反选
function selectNone() {
  const currentSet = new Set(selectedColumns.value);

  // 移除当前过滤列表中已选中的
  filteredColumns.value.forEach((col) => {
    if (currentSet.has(col.name)) {
      currentSet.delete(col.name);
    } else {
      currentSet.add(col.name);
    }
  });

  selectedColumns.value = Array.from(currentSet);
}

// 关闭对话框
function handleClose() {
  emit('update:visible', false);
  // 重置状态
  searchText.value = '';
  selectedColumns.value = [];
}

// 确认按钮是否禁用
const isConfirmDisabled = computed(() => {
  if (props.mode === 'select') {
    // 选择模式：必须选择至少一列，且不能选择所有列
    return selectedColumns.value.length === 0 || selectedColumns.value.length === props.columns.length;
  } else {
    // 删除模式：必须选择至少一列删除，且不能选择所有列
    return selectedColumns.value.length === 0 || selectedColumns.value.length === props.columns.length;
  }
});

// 确认操作
function handleConfirm() {
  emit('confirm', selectedColumns.value);
  handleClose();
}

// 监听对话框打开，初始化选中状态
watch(
  () => props.visible,
  (newVal) => {
    if (newVal) {
      // 如果是选择模式，默认全选
      if (props.mode === 'select') {
        selectedColumns.value = props.columns.map((col) => col.name);
      } else {
        selectedColumns.value = [];
      }
    }
  }
);
</script>

<style scoped>
.dialog-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.toolbar-actions {
  display: flex;
  gap: 8px;
}

.column-list {
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  background-color: #fafafa;
}

.column-list > .el-scrollbar {
  max-height: 400px;
}

.column-item {
  padding: 12px 16px;
  border-bottom: 1px solid #e4e7ed;
  transition: background-color 0.2s;
}

.column-item:last-child {
  border-bottom: none;
}

.column-item:hover {
  background-color: #f0f2f5;
}

.column-name {
  font-weight: 500;
  color: #303133;
  margin-right: 8px;
}

.column-type {
  margin-left: 8px;
}

.dialog-footer-info {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid #e4e7ed;
}

.info-text {
  font-size: 13px;
  color: #909399;
}

:deep(.el-checkbox) {
  width: 100%;
  display: flex;
  align-items: center;
}

:deep(.el-checkbox__label) {
  flex: 1;
  display: flex;
  align-items: center;
  width: 100%;
}

.checkbox-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  width: 100%;
}
</style>
