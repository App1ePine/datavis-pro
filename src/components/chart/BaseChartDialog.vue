<script lang="ts" setup>
import { ElMessage } from 'element-plus';
import { computed, ref } from 'vue';
import type { ChartConfig, ChartData } from '@/types/dataset';
import { generateChartData } from '@/utils/tauri-commands';
import ChartDisplayDialog from './ChartDisplayDialog.vue';

interface Props {
  /** 对话框标题 */
  title: string;
  /** 是否显示对话框 */
  modelValue: boolean;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
  closed: [];
}>();

// 图表显示对话框
const chartDisplayVisible = ref(false);
const chartData = ref<ChartData | null>(null);
const chartConfig = ref<ChartConfig | null>(null);

// 对话框可见性
const dialogVisible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

// 暴露给子组件的方法
const handleSubmit = async (config: ChartConfig) => {
  try {
    // 调用 Rust 后端生成图表数据
    chartData.value = await generateChartData(config);
    chartConfig.value = config;

    // 关闭配置对话框，打开图表显示对话框
    dialogVisible.value = false;
    chartDisplayVisible.value = true;
  } catch (error) {
    ElMessage.error(`生成图表失败: ${error}`);
  }
};

// 暴露方法给父组件
defineExpose({
  handleSubmit,
});
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    :close-on-click-modal="false"
    :title="title"
    destroy-on-close
    width="600px"
    @closed="emit('closed')"
  >
    <!-- 表单内容插槽 -->
    <slot :on-submit="handleSubmit" name="form"></slot>

    <!-- 底部按钮插槽 -->
    <template #footer>
      <slot name="footer">
        <el-button @click="dialogVisible = false">取消</el-button>
      </slot>
    </template>
  </el-dialog>

  <!-- 图表显示对话框 -->
  <ChartDisplayDialog v-model="chartDisplayVisible" :chart-config="chartConfig" :chart-data="chartData" />
</template>
