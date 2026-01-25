<template>
  <el-dialog :model-value="visible" title="重命名列 - Rename Columns" width="700px" @close="handleClose" align-center>
    <!-- 搜索栏 -->
    <div class="dialog-toolbar">
      <el-input v-model="searchText" placeholder="搜索列名..." clearable style="width: 300px">
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
    </div>

    <!-- 列重命名表格 -->
    <div class="rename-table">
      <el-scrollbar height="450px">
        <el-table :data="filteredColumns" stripe>
          <el-table-column label="原名称" prop="name" width="200">
            <template #default="{ row }">
              <span class="original-name">{{ row.name }}</span>
            </template>
          </el-table-column>
          <el-table-column label="数据类型" prop="dtype" width="150">
            <template #default="{ row }">
              <el-tag size="small" type="info">{{ row.dtype }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="新名称">
            <template #default="{ row }">
              <el-input
                v-model="renameMap[row.name]"
                placeholder="输入新名称..."
                :class="{ 'error-input': hasError(row.name) }"
                @input="validateName(row.name)"
              >
                <template #suffix>
                  <el-icon v-if="hasError(row.name)" color="#f56c6c">
                    <CircleClose />
                  </el-icon>
                  <el-icon v-else-if="renameMap[row.name] && renameMap[row.name] !== row.name" color="#67c23a">
                    <CircleCheck />
                  </el-icon>
                </template>
              </el-input>
              <div v-if="hasError(row.name)" class="error-message">
                {{ getErrorMessage(row.name) }}
              </div>
            </template>
          </el-table-column>
        </el-table>
        <el-empty v-if="filteredColumns.length === 0" description="没有找到匹配的列" :image-size="80" />
      </el-scrollbar>
    </div>

    <!-- 底部信息 -->
    <div class="dialog-footer-info">
      <span class="info-text"> 已修改 {{ changedCount }} / {{ columns.length }} 列 </span>
      <span v-if="hasAnyError" class="error-text"> ⚠️ 存在重复或无效的列名 </span>
    </div>

    <!-- 操作按钮 -->
    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" :disabled="changedCount === 0 || hasAnyError" @click="handleConfirm">
        确认操作
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { CircleCheck, CircleClose, Search } from '@element-plus/icons-vue';
import { computed, reactive, ref, watch } from 'vue';
import type { ColumnInfo } from '@/types/dataset';

interface Props {
  visible: boolean;
  columns: ColumnInfo[];
}

interface Emits {
  (e: 'update:visible', value: boolean): void;
  (e: 'confirm', mapping: Record<string, string>): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

// 搜索文本
const searchText = ref('');

// 重命名映射表：原名称 -> 新名称
const renameMap = reactive<Record<string, string>>({});

// 错误映射表：原名称 -> 错误信息
const errorMap = reactive<Record<string, string>>({});

// 过滤后的列
const filteredColumns = computed(() => {
  if (!searchText.value) {
    return props.columns;
  }
  const search = searchText.value.toLowerCase();
  return props.columns.filter((col) => col.name.toLowerCase().includes(search));
});

// 已修改的列数量
const changedCount = computed(() => {
  return Object.keys(renameMap).filter((oldName) => renameMap[oldName] && renameMap[oldName] !== oldName).length;
});

// 是否有任何错误
const hasAnyError = computed(() => {
  return Object.keys(errorMap).some((key) => errorMap[key]);
});

// 检查是否有错误
function hasError(columnName: string): boolean {
  return !!errorMap[columnName];
}

// 获取错误信息
function getErrorMessage(columnName: string): string {
  return errorMap[columnName] || '';
}

// 验证列名
function validateName(oldName: string) {
  const newName = renameMap[oldName];

  // 清除之前的错误
  delete errorMap[oldName];

  // 如果新名称为空或与原名称相同，不需要验证
  if (!newName || newName === oldName) {
    return;
  }

  // 检查是否为空字符串
  if (newName.trim() === '') {
    errorMap[oldName] = '列名不能为空';
    return;
  }

  // 检查是否与其他列名重复
  const allNewNames = Object.entries(renameMap)
    .filter(([key, value]) => value && value !== key)
    .map(([_, value]) => value);

  const allOriginalNames = props.columns.map((col) => col.name);

  // 检查新名称是否与其他新名称重复
  const duplicateInNew = allNewNames.filter((name) => name === newName).length > 1;
  if (duplicateInNew) {
    errorMap[oldName] = '列名重复';
    return;
  }

  // 检查新名称是否与未修改的原始列名重复
  const duplicateInOriginal = allOriginalNames.some((name) => name === newName && !renameMap[name]);
  if (duplicateInOriginal) {
    errorMap[oldName] = '列名与现有列重复';
    return;
  }
}

// 关闭对话框
function handleClose() {
  emit('update:visible', false);
  // 重置状态
  searchText.value = '';
  Object.keys(renameMap).forEach((key) => {
    delete renameMap[key];
  });
  Object.keys(errorMap).forEach((key) => {
    delete errorMap[key];
  });
}

// 确认操作
function handleConfirm() {
  // 只提交实际修改的列
  const mapping: Record<string, string> = {};
  Object.entries(renameMap).forEach(([oldName, newName]) => {
    if (newName && newName !== oldName) {
      mapping[oldName] = newName;
    }
  });

  emit('confirm', mapping);
  handleClose();
}

// 监听对话框打开，初始化重命名映射
watch(
  () => props.visible,
  (newVal) => {
    if (newVal) {
      // 初始化重命名映射为原名称
      props.columns.forEach((col) => {
        renameMap[col.name] = col.name;
      });
    }
  }
);
</script>

<style scoped>
.dialog-toolbar {
  margin-bottom: 16px;
}

.rename-table {
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  overflow: hidden;
}

.original-name {
  font-weight: 500;
  color: #303133;
}

.error-input :deep(.el-input__wrapper) {
  box-shadow: 0 0 0 1px #f56c6c inset;
}

.error-message {
  font-size: 12px;
  color: #f56c6c;
  margin-top: 4px;
}

.dialog-footer-info {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid #e4e7ed;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.info-text {
  font-size: 13px;
  color: #909399;
}

.error-text {
  font-size: 13px;
  color: #f56c6c;
  font-weight: 500;
}

:deep(.el-table) {
  font-size: 13px;
}

:deep(.el-table th) {
  background-color: #f5f7fa;
  font-weight: 600;
}
</style>
