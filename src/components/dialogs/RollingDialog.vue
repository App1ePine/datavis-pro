<template>
  <el-dialog
    :model-value="visible"
    title="滑动窗口 - Rolling Window"
    width="600px"
    @update:model-value="(val: boolean) => $emit('update:visible', val)"
    @close="handleClose"
    align-center
  >
    <div class="rolling-content">
      <!-- 说明 -->
      <el-alert title="计算滑动平均、滑动中位数等" type="info" :closable="false" show-icon>
        <template #default>
          <div class="alert-content">
            <p>滑动窗口常用于时间序列分析，可以平滑数据波动。</p>
            <p><strong>示例：</strong>窗口大小为 7 的滑动平均可以计算每连续7天的平均值。</p>
          </div>
        </template>
      </el-alert>

      <!-- 操作类型选择 -->
      <div class="form-section">
        <h4 class="section-title">操作类型</h4>
        <el-select v-model="operationType" placeholder="选择操作类型" style="width: 100%">
          <el-option label="移动求和 (Sum)" value="sum" />
          <el-option label="移动平均 (Average)" value="average" />
          <el-option label="移动中位数 (Median)" value="median" />
          <el-option label="移动最小值 (Min)" value="min" />
          <el-option label="移动最大值 (Max)" value="max" />
          <el-option label="移动标准差 (Std)" value="std" />
          <el-option label="移动方差 (Var)" value="var" />
          <el-option label="移动分位数 (Quantile)" value="quantile" />
        </el-select>
      </div>

      <!-- 分位数设置 (仅在选择分位数时显示) -->
      <div v-if="operationType === 'quantile'" class="form-section">
        <h4 class="section-title">分位数值 (0.0 - 1.0)</h4>
        <el-input-number
          v-model="quantileValue"
          :min="0.01"
          :max="0.99"
          :step="0.01"
          :precision="2"
          style="width: 100%"
        />
      </div>

      <!-- 列选择 -->
      <div class="form-section">
        <h4 class="section-title">选择列</h4>
        <el-select v-model="selectedColumn" placeholder="选择要计算的数值列" style="width: 100%">
          <el-option v-for="col in numericColumns" :key="col.name" :label="col.name" :value="col.name">
            <span style="float: left">{{ col.name }}</span>
            <span style="float: right; color: #8492a6; font-size: 13px; margin-left: 10px">
              <el-tag size="small" type="info">{{ col.dtype }}</el-tag>
            </span>
          </el-option>
        </el-select>
      </div>

      <!-- 窗口大小 -->
      <div class="form-section">
        <h4 class="section-title">窗口配置</h4>
        <div class="window-config-grid">
          <div class="config-item">
            <span class="label">窗口大小</span>
            <el-input-number v-model="windowSize" :min="1" :max="1000" :step="1" />
          </div>
          <div class="config-item">
            <span class="label">最小窗口</span>
            <el-input-number v-model="minPeriods" :min="1" :max="windowSize" :step="1" />
          </div>
          <div class="config-item switch-item">
            <span class="label">窗口居中</span>
            <el-switch v-model="center" />
          </div>
        </div>
        <div class="hint">窗口大小决定计算范围。最小窗口指计算所需的最小非空值数量。</div>
      </div>

      <!-- 结果预览 -->
      <div v-if="selectedColumn && windowSize" class="result-preview">
        <el-alert type="success" :closable="false">
          <template #default>
            将添加新列：
            <strong>{{ resultColumnName }}</strong>
          </template>
        </el-alert>
      </div>
    </div>

    <template #footer>
      <span class="dialog-footer">
        <el-button @click="handleClose">取消</el-button>
        <el-button type="primary" :loading="loading" :disabled="!isValid" @click="handleConfirm"> 确认操作 </el-button>
      </span>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ElMessage } from 'element-plus';
import { computed, ref } from 'vue';
import { useDataStore } from '@/stores/dataStore';
import type { ColumnInfo } from '@/types/dataset';
import {
  rollingAverage,
  rollingMax,
  rollingMedian,
  rollingMin,
  rollingQuantile,
  rollingStd,
  rollingSum,
  rollingVar,
} from '@/utils/tauri-commands';

// Props
defineProps<{
  visible: boolean;
}>();

// Emits
const emit = defineEmits<{
  'update:visible': [value: boolean];
  success: [];
}>();

// Store
const dataStore = useDataStore();

// 状态
type OperationType = 'average' | 'median' | 'sum' | 'min' | 'max' | 'std' | 'var' | 'quantile';
const operationType = ref<OperationType>('average');
const selectedColumn = ref<string>('');
const windowSize = ref<number>(7);
const quantileValue = ref<number>(0.5);
const center = ref(false);
const minPeriods = ref<number>(1);
const loading = ref(false);

// 可用的数值列
const numericColumns = computed<ColumnInfo[]>(() => {
  const columns = dataStore.currentInfo?.columns || [];
  return columns.filter((col) => {
    const dtype = col.dtype.toLowerCase();
    return dtype.includes('int') || dtype.includes('float') || dtype.includes('decimal');
  });
});

// 结果列名
const resultColumnName = computed(() => {
  if (!selectedColumn.value || !windowSize.value) return '';
  let suffix = '';
  switch (operationType.value) {
    case 'average':
      suffix = 'rolling_avg';
      break;
    case 'median':
      suffix = 'rolling_median';
      break;
    case 'sum':
      suffix = 'rolling_sum';
      break;
    case 'min':
      suffix = 'rolling_min';
      break;
    case 'max':
      suffix = 'rolling_max';
      break;
    case 'std':
      suffix = 'rolling_std';
      break;
    case 'var':
      suffix = 'rolling_var';
      break;
    case 'quantile':
      suffix = `rolling_quantile_${quantileValue.value}`;
      break;
  }
  if (center.value) suffix += '_center';
  return `${selectedColumn.value}_${suffix}_${windowSize.value}`;
});

// 验证
const isValid = computed(() => {
  return selectedColumn.value && windowSize.value >= 1;
});

// 关闭对话框
const handleClose = () => {
  emit('update:visible', false);
  // 重置表单
  operationType.value = 'average';
  selectedColumn.value = '';
  windowSize.value = 7;
  quantileValue.value = 0.5;
  center.value = false;
  minPeriods.value = 1;
};

// 确认执行
const handleConfirm = async () => {
  if (!isValid.value) {
    ElMessage.warning('请选择列并设置窗口大小');
    return;
  }

  loading.value = true;
  try {
    switch (operationType.value) {
      case 'average':
        await rollingAverage(selectedColumn.value, windowSize.value, center.value, minPeriods.value);
        ElMessage.success('移动平均计算成功');
        break;
      case 'median':
        await rollingMedian(selectedColumn.value, windowSize.value, center.value, minPeriods.value);
        ElMessage.success('移动中位数计算成功');
        break;
      case 'sum':
        await rollingSum(selectedColumn.value, windowSize.value, center.value, minPeriods.value);
        ElMessage.success('移动求和计算成功');
        break;
      case 'min':
        await rollingMin(selectedColumn.value, windowSize.value, center.value, minPeriods.value);
        ElMessage.success('移动最小值计算成功');
        break;
      case 'max':
        await rollingMax(selectedColumn.value, windowSize.value, center.value, minPeriods.value);
        ElMessage.success('移动最大值计算成功');
        break;
      case 'std':
        await rollingStd(selectedColumn.value, windowSize.value, center.value, minPeriods.value);
        ElMessage.success('移动标准差计算成功');
        break;
      case 'var':
        await rollingVar(selectedColumn.value, windowSize.value, center.value, minPeriods.value);
        ElMessage.success('移动方差计算成功');
        break;
      case 'quantile':
        await rollingQuantile(
          selectedColumn.value,
          windowSize.value,
          quantileValue.value,
          center.value,
          minPeriods.value
        );
        ElMessage.success('移动分位数计算成功');
        break;
    }

    emit('success');
    handleClose();

    // 刷新数据
    await dataStore.refreshCurrentData();
  } catch (error) {
    console.error('移动窗口计算失败:', error);
    ElMessage.error(`移动窗口计算失败: ${error}`);
  } finally {
    loading.value = false;
  }
};
</script>

<style scoped>
.rolling-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.alert-content {
  font-size: 13px;
  line-height: 1.6;
}

.alert-content p {
  margin: 5px 0;
}

.alert-content strong {
  color: #409eff;
}

.form-section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.section-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: #303133;
}

.hint {
  font-size: 12px;
  color: #909399;
  line-height: 1.5;
}

.result-preview {
  padding: 10px;
  background-color: #f0f9ff;
  border-radius: 4px;
}

.result-preview strong {
  color: #409eff;
  font-family: 'Courier New', monospace;
}

.window-config-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  align-items: flex-start;
}

.config-item {
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: flex-start;
  gap: 10px;
}

.label {
  font-size: 13px;
  color: #606266;
  white-space: nowrap;
  min-width: 80px;
}

.switch-item {
  flex-direction: row;
  align-items: center;
  gap: 10px;
  padding-bottom: 5px;
}

.label {
  font-size: 13px;
  color: #606266;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>
