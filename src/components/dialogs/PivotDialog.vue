<template>
  <el-dialog
    :model-value="visible"
    title="长表转宽表 - Pivot"
    width="700px"
    @update:model-value="(val: boolean) => $emit('update:visible', val)"
    @close="handleClose"
    align-center
  >
    <div class="pivot-content">
      <!-- 说明 -->
      <el-alert title="将长格式数据转为宽格式" type="info" :closable="false" show-icon>
        <template #default>
          <div class="alert-content">
            <p>透视表可以将长格式数据重组为宽格式，便于分析和展示。</p>
            <p><strong>示例：</strong></p>
            <p>输入：| name | year | sales |</p>
            <p>输出：| name | 2020 | 2021 | 2022 |</p>
          </div>
        </template>
      </el-alert>

      <!-- 索引列选择 -->
      <div class="form-section">
        <h4 class="section-title">索引列（行标识）</h4>
        <el-select v-model="indexColumns" multiple placeholder="选择作为行索引的列" style="width: 100%">
          <el-option v-for="col in availableColumns" :key="col.name" :label="col.name" :value="col.name">
            <span style="float: left">{{ col.name }}</span>
            <span style="float: right; color: #8492a6; font-size: 13px; margin-left: 10px">
              <el-tag size="small" type="info">{{ col.dtype }}</el-tag>
            </span>
          </el-option>
        </el-select>
      </div>

      <!-- 列名来源列 -->
      <div class="form-section">
        <h4 class="section-title">列名来源列</h4>
        <el-select v-model="columnsColumn" placeholder="选择作为新列名的列" style="width: 100%">
          <el-option v-for="col in availableColumns" :key="col.name" :label="col.name" :value="col.name">
            <span style="float: left">{{ col.name }}</span>
            <span style="float: right; color: #8492a6; font-size: 13px; margin-left: 10px">
              <el-tag size="small" type="info">{{ col.dtype }}</el-tag>
            </span>
          </el-option>
        </el-select>
      </div>

      <!-- 值来源列 -->
      <div class="form-section">
        <h4 class="section-title">值来源列</h4>
        <el-select v-model="valuesColumn" placeholder="选择作为值的列" style="width: 100%">
          <el-option v-for="col in availableColumns" :key="col.name" :label="col.name" :value="col.name">
            <span style="float: left">{{ col.name }}</span>
            <span style="float: right; color: #8492a6; font-size: 13px; margin-left: 10px">
              <el-tag size="small" type="info">{{ col.dtype }}</el-tag>
            </span>
          </el-option>
        </el-select>
      </div>

      <!-- 聚合函数 -->
      <div class="form-section">
        <h4 class="section-title">聚合函数</h4>
        <el-select v-model="aggregateFunc" placeholder="选择聚合函数" style="width: 100%">
          <el-option label="First（第一个值）" value="first" />
          <el-option label="Last（最后一个值）" value="last" />
          <el-option label="Sum（求和）" value="sum" />
          <el-option label="Mean（平均值）" value="mean" />
          <el-option label="Min（最小值）" value="min" />
          <el-option label="Max（最大值）" value="max" />
          <el-option label="Count（计数）" value="count" />
        </el-select>
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
import { pivotData } from '@/utils/tauri-commands';

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
const indexColumns = ref<string[]>([]);
const columnsColumn = ref<string>('');
const valuesColumn = ref<string>('');
const aggregateFunc = ref<string>('first');
const loading = ref(false);

// 可用列
const availableColumns = computed<ColumnInfo[]>(() => {
  return dataStore.currentInfo?.columns || [];
});

// 验证
const isValid = computed(() => {
  return indexColumns.value.length > 0 && columnsColumn.value && valuesColumn.value && aggregateFunc.value;
});

// 关闭对话框
const handleClose = () => {
  emit('update:visible', false);
  // 重置表单
  indexColumns.value = [];
  columnsColumn.value = '';
  valuesColumn.value = '';
  aggregateFunc.value = 'first';
};

// 确认执行
const handleConfirm = async () => {
  if (!isValid.value) {
    ElMessage.warning('请填写所有必填项');
    return;
  }

  loading.value = true;
  try {
    await pivotData(indexColumns.value, columnsColumn.value, valuesColumn.value, aggregateFunc.value);

    ElMessage.success('透视表操作成功');
    emit('success');
    handleClose();

    // 刷新数据
    await dataStore.refreshCurrentData();
  } catch (error) {
    console.error('透视表操作失败:', error);
    ElMessage.error(`透视表操作失败: ${error}`);
  } finally {
    loading.value = false;
  }
};
</script>

<style scoped>
.pivot-content {
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

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>
