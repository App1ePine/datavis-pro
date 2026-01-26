<script lang="ts" setup>
import { computed } from 'vue';
import type { ColumnInfo } from '@/types/dataset';

interface Props {
  /** 可选的列列表 */
  columns: ColumnInfo[];
  /** 当前选中的列 */
  modelValue: string | string[];
  /** 是否多选 */
  multiple?: boolean;
  /** 占位符文本 */
  placeholder?: string;
  /** 是否只显示数值列 */
  numericOnly?: boolean;
  /** 是否禁用 */
  disabled?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  multiple: false,
  placeholder: '请选择列',
  numericOnly: false,
  disabled: false,
});

const emit = defineEmits<{
  'update:modelValue': [value: string | string[]];
}>();

// 过滤列（如果需要只显示数值列）
const filteredColumns = computed(() => {
  if (!props.numericOnly) {
    return props.columns;
  }
  return props.columns.filter((col) => {
    const dtype = col.dtype.toLowerCase();
    return dtype.includes('int') || dtype.includes('float') || dtype.includes('decimal') || dtype.includes('numeric');
  });
});

const handleChange = (value: string | string[]) => {
  emit('update:modelValue', value);
};
</script>

<template>
  <el-select
    :disabled="disabled"
    :model-value="modelValue"
    :multiple="multiple"
    :placeholder="placeholder"
    class="chart-select"
    clearable
    filterable
    @update:model-value="handleChange"
  >
    <el-option v-for="col in filteredColumns" :key="col.name" :label="col.name" :value="col.name">
      <span>{{ col.name }}</span>
      <span class="chart-column-type">
        {{ col.dtype }}
      </span>
    </el-option>
  </el-select>
</template>

<style scoped>
.chart-select {
  width: 100%;
}

.chart-column-type {
  float: right;
  color: var(--el-text-color-secondary);
  font-size: 12px;
  margin-left: 8px;
}
</style>
