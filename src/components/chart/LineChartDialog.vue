<script lang="ts" setup>
import { storeToRefs } from 'pinia';
import { computed, ref, watch } from 'vue';
import { useDataStore } from '@/stores/dataStore';
import type { ChartConfig, LineChartStyle } from '@/types/dataset';
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

const rightAxisColumns = computed(() => {
  const selected = new Set(formData.value.yColumns);
  return (currentInfo.value?.columns || []).filter((col) => selected.has(col.name));
});

// 表单数据
const formData = ref({
  xColumn: '',
  yColumns: [] as string[],
  title: '',
  lineStyle: 'line' as LineChartStyle,
  enableDualAxis: false,
  rightYColumns: [] as string[],
});

// 对话框可见性
const dialogVisible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

watch(
  () => formData.value.yColumns,
  (next) => {
    formData.value.rightYColumns = formData.value.rightYColumns.filter((col) => next.includes(col));
  },
  { deep: true }
);

watch(
  () => formData.value.enableDualAxis,
  (enabled) => {
    if (!enabled) {
      formData.value.rightYColumns = [];
    }
  }
);

// 基础对话框引用
const baseDialogRef = ref<{ handleSubmit: (config: ChartConfig) => void }>();

// 提交表单
const handleSubmit = () => {
  // 验证
  if (!formData.value.xColumn) {
    return;
  }
  if (formData.value.yColumns.length === 0) {
    return;
  }

  const config: ChartConfig = {
    chart_type: 'line',
    x_column: formData.value.xColumn,
    y_columns: formData.value.yColumns,
    title: formData.value.title || undefined,
    line_style: formData.value.lineStyle,
    y_axis_right_columns: formData.value.enableDualAxis ? formData.value.rightYColumns : undefined,
  };

  baseDialogRef.value?.handleSubmit(config);
};

// 重置表单
const resetForm = () => {
  formData.value = {
    xColumn: '',
    yColumns: [],
    title: '',
    lineStyle: 'line',
    enableDualAxis: false,
    rightYColumns: [],
  };
};
</script>

<template>
  <BaseChartDialog ref="baseDialogRef" v-model="dialogVisible" title="折线图配置" @closed="resetForm">
    <template #form>
      <el-form :model="formData" label-width="100px">
        <el-form-item label="X 轴" required>
          <ChartColumnSelector
            v-model="formData.xColumn"
            :columns="currentInfo?.columns || []"
            placeholder="选择 X 轴列"
          />
        </el-form-item>

        <el-form-item label="Y 轴" required>
          <ChartColumnSelector
            v-model="formData.yColumns"
            :columns="currentInfo?.columns || []"
            :multiple="true"
            :numeric-only="true"
            placeholder="选择 Y 轴列（可多选）"
          />
          <div class="chart-form-hint">支持多选，每个 Y 轴列将作为一个系列显示</div>
        </el-form-item>

        <el-form-item label="图表样式">
          <el-select v-model="formData.lineStyle" class="chart-dialog-select" placeholder="选择样式">
            <el-option label="折线" value="line" />
            <el-option label="面积" value="area" />
            <el-option label="面积堆积" value="stack_area" />
          </el-select>
        </el-form-item>

        <el-form-item label="双 Y 轴">
          <el-switch v-model="formData.enableDualAxis" />
          <div class="chart-form-hint">开启后可指定右侧 Y 轴系列</div>
        </el-form-item>

        <el-form-item v-if="formData.enableDualAxis" label="右侧 Y 轴">
          <ChartColumnSelector
            v-model="formData.rightYColumns"
            :columns="rightAxisColumns"
            :multiple="true"
            :numeric-only="true"
            placeholder="选择右侧 Y 轴列"
          />
          <div class="chart-form-hint">仅对已选择的 Y 轴列生效</div>
        </el-form-item>

        <el-form-item label="图表标题">
          <el-input v-model="formData.title" clearable placeholder="可选" />
        </el-form-item>
      </el-form>
    </template>

    <template #footer>
      <el-button @click="dialogVisible = false">取消</el-button>
      <el-button :disabled="!formData.xColumn || formData.yColumns.length === 0" type="primary" @click="handleSubmit">
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
