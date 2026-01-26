<script lang="ts" setup>
import { storeToRefs } from 'pinia';
import { computed, ref } from 'vue';
import { useDataStore } from '@/stores/dataStore';
import type { ChartConfig } from '@/types/dataset';
import BaseChartDialog from './BaseChartDialog.vue';
import ChartColumnSelector from './ChartColumnSelector.vue';

interface Props {
  modelValue: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

const dataStore = useDataStore();
const { currentInfo } = storeToRefs(dataStore);

// 表单数据
const formData = ref({
  categoryColumn: '',
  valueColumn: '',
  title: '',
});

// 对话框可见性
const dialogVisible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

// 基础对话框引用
const baseDialogRef = ref<{ handleSubmit: (config: ChartConfig) => void }>();

// 提交表单
const handleSubmit = () => {
  // 验证
  if (!formData.value.categoryColumn) {
    return;
  }
  if (!formData.value.valueColumn) {
    return;
  }

  const config: ChartConfig = {
    chart_type: 'pie',
    category_column: formData.value.categoryColumn,
    value_column: formData.value.valueColumn,
    title: formData.value.title || undefined,
  };

  baseDialogRef.value?.handleSubmit(config);
};

// 重置表单
const resetForm = () => {
  formData.value = {
    categoryColumn: '',
    valueColumn: '',
    title: '',
  };
};
</script>

<template>
  <BaseChartDialog ref="baseDialogRef" v-model="dialogVisible" title="饼图配置" @closed="resetForm">
    <template #form>
      <el-form :model="formData" label-width="100px">
        <el-form-item label="分类列" required>
          <ChartColumnSelector
            v-model="formData.categoryColumn"
            :columns="currentInfo?.columns || []"
            placeholder="选择分类列"
          />
          <div class="chart-form-hint">用于饼图的扇区名称</div>
        </el-form-item>

        <el-form-item label="值列" required>
          <ChartColumnSelector
            v-model="formData.valueColumn"
            :columns="currentInfo?.columns || []"
            :numeric-only="true"
            placeholder="选择值列"
          />
          <div class="chart-form-hint">用于饼图的扇区大小</div>
        </el-form-item>

        <el-form-item label="图表标题">
          <el-input v-model="formData.title" clearable placeholder="可选" />
        </el-form-item>
      </el-form>
    </template>

    <template #footer>
      <el-button @click="dialogVisible = false">取消</el-button>
      <el-button :disabled="!formData.categoryColumn || !formData.valueColumn" type="primary" @click="handleSubmit">
        生成图表
      </el-button>
    </template>
  </BaseChartDialog>
</template>

<style scoped>
.chart-form-hint {
  color: var(--el-text-color-secondary);
  font-size: 12px;
  margin-top: 4px;
}
</style>
