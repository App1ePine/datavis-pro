<template>
  <el-dialog
    :model-value="visible"
    title="更改列类型 - Cast Column Types"
    width="700px"
    @close="handleClose"
    align-center
  >
    <!-- 搜索栏 -->
    <div class="dialog-toolbar">
      <el-input v-model="searchText" placeholder="搜索列名..." clearable style="width: 300px">
        <template #prefix>
          <el-icon><Search /></el-icon>
        </template>
      </el-input>
    </div>

    <!-- 类型转换表格 -->
    <div class="cast-table">
      <el-scrollbar height="450px">
        <el-table :data="filteredColumns" stripe>
          <el-table-column label="列名" prop="name" width="200">
            <template #default="{ row }">
              <span class="column-name">{{ row.name }}</span>
            </template>
          </el-table-column>
          <el-table-column label="当前类型" prop="dtype" width="150">
            <template #default="{ row }">
              <el-tag size="small" type="info">{{ row.dtype }}</el-tag>
            </template>
          </el-table-column>
          <el-table-column label="目标类型">
            <template #default="{ row }">
              <el-select v-model="typeMap[row.name]" placeholder="选择类型" style="width: 100%">
                <el-option
                  v-for="type in availableTypes"
                  :key="type.value"
                  :label="type.label"
                  :value="type.value"
                  :disabled="type.value === row.dtype"
                >
                  <span>{{ type.label }}</span>
                  <span v-if="type.value === row.dtype" style="color: #909399; margin-left: 8px"> (当前) </span>
                </el-option>
              </el-select>
            </template>
          </el-table-column>
        </el-table>
        <el-empty v-if="filteredColumns.length === 0" description="没有找到匹配的列" :image-size="80" />
      </el-scrollbar>
    </div>

    <!-- 底部信息 -->
    <div class="dialog-footer-info">
      <span class="info-text"> 已修改 {{ changedCount }} / {{ columns.length }} 列 </span>
      <el-alert v-if="changedCount > 0" type="warning" :closable="false" show-icon style="margin-top: 12px">
        <template #title>
          <span style="font-size: 13px"> 类型转换可能失败，请确保数据格式正确 </span>
        </template>
      </el-alert>
    </div>

    <!-- 操作按钮 -->
    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" :disabled="changedCount === 0" @click="handleConfirm"> 确认操作 </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { Search } from '@element-plus/icons-vue';
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

// 可用的数据类型 - 支持所有 Polars 基础类型
const availableTypes = [
  // 整数类型
  { label: 'Int8 (8位整数)', value: 'Int8' },
  { label: 'Int16 (16位整数)', value: 'Int16' },
  { label: 'Int32 (32位整数)', value: 'Int32' },
  { label: 'Int64 (64位整数)', value: 'Int64' },
  { label: 'UInt8 (8位无符号整数)', value: 'UInt8' },
  { label: 'UInt16 (16位无符号整数)', value: 'UInt16' },
  { label: 'UInt32 (32位无符号整数)', value: 'UInt32' },
  { label: 'UInt64 (64位无符号整数)', value: 'UInt64' },
  // 浮点类型
  { label: 'Float32 (32位浮点数)', value: 'Float32' },
  { label: 'Float64 (64位浮点数)', value: 'Float64' },
  // 字符串和布尔
  { label: 'String (字符串)', value: 'String' },
  { label: 'Boolean (布尔)', value: 'Boolean' },
  // 日期时间类型
  { label: 'Date (日期)', value: 'Date' },
  { label: 'Datetime (日期时间)', value: 'Datetime' },
  { label: 'Time (时间)', value: 'Time' },
  { label: 'Duration (时长)', value: 'Duration' },
];

// 搜索文本
const searchText = ref('');

// 类型映射表：列名 -> 目标类型
const typeMap = reactive<Record<string, string>>({});

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
  return Object.keys(typeMap).filter((colName) => {
    const col = props.columns.find((c) => c.name === colName);
    return typeMap[colName] && col && typeMap[colName] !== col.dtype;
  }).length;
});

// 关闭对话框
function handleClose() {
  emit('update:visible', false);
  // 重置状态
  searchText.value = '';
  Object.keys(typeMap).forEach((key) => {
    delete typeMap[key];
  });
}

// 确认操作
function handleConfirm() {
  // 只提交实际修改的列
  const mapping: Record<string, string> = {};
  Object.entries(typeMap).forEach(([colName, targetType]) => {
    const col = props.columns.find((c) => c.name === colName);
    if (targetType && col && targetType !== col.dtype) {
      mapping[colName] = targetType;
    }
  });

  emit('confirm', mapping);
  handleClose();
}

// 监听对话框打开，初始化类型映射
watch(
  () => props.visible,
  (newVal) => {
    if (newVal) {
      // 初始化类型映射为当前类型
      props.columns.forEach((col) => {
        typeMap[col.name] = col.dtype;
      });
    }
  }
);
</script>

<style scoped>
.dialog-toolbar {
  margin-bottom: 16px;
}

.cast-table {
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  overflow: hidden;
}

.column-name {
  font-weight: 500;
  color: #303133;
}

.dialog-footer-info {
  margin-top: 16px;
  padding-top: 12px;
  border-top: 1px solid #e4e7ed;
}

.info-text {
  font-size: 13px;
  color: #909399;
  display: block;
  margin-bottom: 8px;
}

:deep(.el-table) {
  font-size: 13px;
}

:deep(.el-table th) {
  background-color: #f5f7fa;
  font-weight: 600;
}

:deep(.el-alert) {
  padding: 8px 12px;
}
</style>
