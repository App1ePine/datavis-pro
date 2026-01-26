<script lang="ts" setup>
import { Loading } from '@element-plus/icons-vue';
import type { EChartsOption } from 'echarts';
import { BarChart, LineChart, PieChart, ScatterChart } from 'echarts/charts';
import {
  DatasetComponent,
  DataZoomComponent,
  GridComponent,
  LegendComponent,
  TitleComponent,
  TooltipComponent,
} from 'echarts/components';
import { use } from 'echarts/core';
import { CanvasRenderer } from 'echarts/renderers';
import { computed, nextTick, ref, watch } from 'vue';
import VChart from 'vue-echarts';
import type { ChartConfig, ChartData } from '@/types/dataset';

// 注册 ECharts 组件
use([
  CanvasRenderer,
  LineChart,
  BarChart,
  ScatterChart,
  PieChart,
  TitleComponent,
  TooltipComponent,
  LegendComponent,
  GridComponent,
  DataZoomComponent,
  DatasetComponent,
]);

interface Props {
  modelValue: boolean;
  chartData: ChartData | null;
  chartConfig: ChartConfig | null;
}

const props = defineProps<Props>();

const emit = defineEmits<{
  'update:modelValue': [value: boolean];
}>();

// 对话框可见性
const dialogVisible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value),
});

const hasData = computed(() => (props.chartData?.data_count ?? 0) > 0);

// ECharts 配置
const chartOption = ref<EChartsOption>({});
const chartReady = ref(false);

// 监听对话框打开，等待 DOM 渲染后再显示图表
watch(dialogVisible, async (visible) => {
  if (visible && props.chartData && hasData.value) {
    const chartData = props.chartData;
    chartReady.value = false;
    await nextTick();
    setTimeout(() => {
      chartOption.value = buildChartOption(chartData, props.chartConfig ?? null);
      console.log('Chart Option:', chartOption.value);
      console.log('Dataset:', chartData.dataset);
      chartReady.value = true;
    }, 100);
  } else if (visible && props.chartData && !hasData.value) {
    chartOption.value = {};
    chartReady.value = false;
  }
});

// 构建 ECharts option
function buildChartOption(data: ChartData, config: ChartConfig | null): EChartsOption {
  // 根据图表类型添加特定配置
  switch (data.chart_type) {
    case 'line':
      return buildLineChartOption(data, config);
    case 'bar':
      return buildBarChartOption(data);
    case 'scatter':
      return buildScatterChartOption(data);
    case 'pie':
      return buildPieChartOption(data);
    case 'histogram':
      return buildHistogramChartOption(data);
    default:
      return {};
  }
}

// 构建折线图配置
function buildLineChartOption(data: ChartData, config: ChartConfig | null): EChartsOption {
  const header = data.dataset[0] as Array<string | number | boolean | null>;
  const dimensions = header.map((item) => String(item));
  const yColumns = dimensions.slice(1);

  const lineStyle = config?.line_style ?? 'line';
  const rightAxisColumns = new Set(config?.y_axis_right_columns ?? []);
  const useDualAxis = rightAxisColumns.size > 0;
  type YAxisOption = NonNullable<EChartsOption['yAxis']>;
  const yAxisOption: YAxisOption = useDualAxis
    ? [{ type: 'value' as const }, { type: 'value' as const, position: 'right' }]
    : { type: 'value' as const };

  // 将 Vue Proxy 转换为普通数组
  const plainDataset = JSON.parse(JSON.stringify(data.dataset));

  return {
    dataset: {
      source: plainDataset,
      dimensions,
    },
    legend: {},
    tooltip: {
      trigger: 'axis',
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '15%',
      top: '10%',
    },
    xAxis: {
      type: 'category',
    },
    yAxis: yAxisOption,
    dataZoom: [{ type: 'slider', start: 0, end: 100 }, { type: 'inside' }],
    series: yColumns.map((name, index) => {
      const seriesItem: Record<string, unknown> = {
        name,
        type: 'line',
        smooth: true,
        encode: {
          x: dimensions[0],
          y: dimensions[index + 1],
        },
      };

      if (useDualAxis) {
        seriesItem.yAxisIndex = rightAxisColumns.has(name) ? 1 : 0;
      }

      if (lineStyle === 'area' || lineStyle === 'stack_area') {
        seriesItem.areaStyle = {};
      }

      if (lineStyle === 'stack_area') {
        seriesItem.stack = 'total';
      }

      return seriesItem;
    }),
  };
}

// 构建柱状图配置
function buildBarChartOption(data: ChartData): EChartsOption {
  const header = data.dataset[0] as Array<string | number | boolean | null>;
  const dimensions = header.map((item) => String(item));
  const yColumns = dimensions.slice(1);

  // 将 Vue Proxy 转换为普通数组
  const plainDataset = JSON.parse(JSON.stringify(data.dataset));

  return {
    dataset: {
      source: plainDataset,
      dimensions,
    },
    legend: {},
    tooltip: {
      trigger: 'axis',
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '15%',
      top: '10%',
    },
    xAxis: {
      type: 'category',
    },
    yAxis: {
      type: 'value',
    },
    dataZoom: [{ type: 'slider', start: 0, end: 100 }, { type: 'inside' }],
    series: yColumns.map((name, index) => ({
      name,
      type: 'bar',
      encode: {
        x: dimensions[0],
        y: dimensions[index + 1],
      },
    })),
  };
}

// 构建散点图配置
function buildScatterChartOption(data: ChartData): EChartsOption {
  const header = data.dataset[0] as Array<string | number | boolean | null>;
  const dimensions = header.map((item) => String(item));
  const yColumns = dimensions.slice(1);

  // 将 Vue Proxy 转换为普通数组
  const plainDataset = JSON.parse(JSON.stringify(data.dataset));

  return {
    dataset: {
      source: plainDataset,
      dimensions,
    },
    legend: {},
    tooltip: {
      trigger: 'axis',
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '15%',
      top: '10%',
    },
    xAxis: {
      type: 'value',
    },
    yAxis: {
      type: 'value',
    },
    dataZoom: [
      { type: 'slider', xAxisIndex: 0, start: 0, end: 100 },
      { type: 'slider', yAxisIndex: 0, start: 0, end: 100 },
      { type: 'inside' },
    ],
    series: yColumns.map((name, index) => ({
      name,
      type: 'scatter',
      encode: {
        x: dimensions[0],
        y: dimensions[index + 1],
      },
    })),
  };
}

// 构建直方图配置
function buildHistogramChartOption(data: ChartData): EChartsOption {
  const header = data.dataset[0] as Array<string | number | boolean | null>;
  const dimensions = header.map((item) => String(item));

  // 将 Vue Proxy 转换为普通数组
  const plainDataset = JSON.parse(JSON.stringify(data.dataset));

  return {
    dataset: {
      source: plainDataset,
      dimensions,
    },
    legend: {},
    tooltip: {
      trigger: 'axis',
    },
    grid: {
      left: '3%',
      right: '4%',
      bottom: '15%',
      top: '10%',
    },
    xAxis: {
      type: 'category',
      axisLabel: {
        rotate: 30,
      },
    },
    yAxis: {
      type: 'value',
    },
    dataZoom: [{ type: 'slider', start: 0, end: 100 }, { type: 'inside' }],
    series: [
      {
        type: 'bar',
        encode: {
          x: dimensions[0],
          y: dimensions[1],
        },
      },
    ],
  };
}

// 构建饼图配置
function buildPieChartOption(data: ChartData): EChartsOption {
  // 将 Vue Proxy 转换为普通数组
  const plainDataset = JSON.parse(JSON.stringify(data.dataset));
  const header = data.dataset[0] as Array<string | number | boolean | null>;
  const dimensions = header.map((item) => String(item));

  return {
    dataset: {
      source: plainDataset,
      dimensions,
    },
    legend: {},
    tooltip: {
      trigger: 'item',
      formatter: '{b}: {c} ({d}%)',
    },
    series: [
      {
        type: 'pie',
        radius: '60%',
        center: ['50%', '50%'],
        encode: {
          itemName: dimensions[0],
          value: dimensions[1],
        },
        emphasis: {
          itemStyle: {
            shadowBlur: 10,
            shadowOffsetX: 0,
            shadowColor: 'rgba(0, 0, 0, 0.5)',
          },
        },
      },
    ],
  };
}
</script>

<template>
  <el-dialog
    v-model="dialogVisible"
    :close-on-click-modal="false"
    :title="
      chartData?.chart_type === 'line'
        ? '折线图'
        : chartData?.chart_type === 'bar'
          ? '柱状图'
          : chartData?.chart_type === 'scatter'
            ? '散点图'
            : chartData?.chart_type === 'histogram'
              ? '直方图'
              : '饼图'
    "
    destroy-on-close
    width="80%"
  >
    <div v-if="chartData && hasData && chartReady" class="chart-canvas">
      <VChart :option="chartOption" autoresize />
    </div>
    <div v-else-if="chartData && hasData && !chartReady" class="chart-loading">
      <el-icon :size="40" class="is-loading">
        <Loading />
      </el-icon>
      <div class="chart-loading-text">图表加载中...</div>
    </div>
    <div v-else class="chart-empty">暂无数据</div>

    <template #footer>
      <el-button @click="dialogVisible = false">关闭</el-button>
    </template>
  </el-dialog>
</template>

<style scoped>
.chart-canvas {
  width: 100%;
  height: 600px;
}

.chart-loading {
  text-align: center;
  padding: 40px;
}

.chart-loading-text {
  margin-top: 16px;
  color: var(--el-text-color-secondary);
}

.chart-empty {
  text-align: center;
  padding: 40px;
  color: var(--el-text-color-secondary);
}
</style>
