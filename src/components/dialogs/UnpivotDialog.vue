<template>
  <el-dialog
    :model-value="visible"
    title="宽表转长表 - Unpivot"
    width="700px"
    @update:model-value="(val: boolean) => $emit('update:visible', val)"
    @close="handleClose"
    align-center
  >
    <div class="unpivot-content">
      <!-- 说明 -->
      <el-alert title="将宽格式数据转为长格式" type="info" :closable="false" show-icon>
        <template #default>
          <div class="alert-content">
            <p>反透视可以将宽格式数据转换为长格式，便于进一步分析。</p>
            <p><strong>示例：</strong></p>
            <p>输入：| name | 2020 | 2021 | 2022 |</p>
            <p>输出：| name | year | sales |</p>
          </div>
        </template>
      </el-alert>

      <!-- 索引列选择 -->
      <div class="form-section">
        <h4 class="section-title">索引列（保持不变的列）</h4>
        <p class="hint-text">这些列会保留在结果中，不会被转换</p>
        <el-select v-model="idVars" multiple placeholder="选择作为索引的列" style="width: 100%">
          <el-option v-for="col in availableColumns" :key="col.name" :label="col.name" :value="col.name">
            <span style="float: left">{{ col.name }}</span>
            <span style="float: right; color: #8492a6; font-size: 13px; margin-left: 10px">
              <el-tag size="small" type="info">{{ col.dtype }}</el-tag>
            </span>
          </el-option>
        </el-select>
      </div>

      <!-- 值列选择 -->
      <div class="form-section">
        <h4 class="section-title">值列（需要转换的列）</h4>
        <p class="hint-text">这些列的列名会变成新的"变量"列，列值会变成新的"值"列</p>
        <el-select v-model="valueVars" multiple placeholder="选择需要转换的列" style="width: 100%">
          <el-option v-for="col in availableColumns" :key="col.name" :label="col.name" :value="col.name">
            <span style="float: left">{{ col.name }}</span>
            <span style="float: right; color: #8492a6; font-size: 13px; margin-left: 10px">
              <el-tag size="small" type="info">{{ col.dtype }}</el-tag>
            </span>
          </el-option>
        </el-select>
      </div>

      <!-- 变量列名和值列名 -->
      <div class="names-grid">
        <div class="form-section">
          <h4 class="section-title">变量列名（可选）</h4>
          <el-input v-model="variableName" placeholder="默认为 'variable'" />
        </div>
        <div class="form-section">
          <h4 class="section-title">值列名（可选）</h4>
          <el-input v-model="valueName" placeholder="默认为 'value'" />
        </div>
      </div>

      <!-- 排序设置 -->
      <div class="form-section">
        <h4 class="section-title">结果排序（可选）</h4>
        <p class="hint-text">选择一列对结果进行排序</p>
        <el-select v-model="sortColumn" placeholder="选择排序列" clearable style="width: 100%">
          <el-option v-for="col in sortableColumns" :key="col" :label="col" :value="col" />
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
import { unpivotData } from '@/utils/tauri-commands';

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
const idVars = ref<string[]>([]);
const valueVars = ref<string[]>([]);
const variableName = ref<string>('');
const valueName = ref<string>('');
const sortColumn = ref<string>('');
const loading = ref(false);

// 可用列
const availableColumns = computed<ColumnInfo[]>(() => {
  return dataStore.currentInfo?.columns || [];
});

// 可排序的列（结果包含的列）
const sortableColumns = computed<string[]>(() => {
  const vars = [...idVars.value];
  vars.push(variableName.value || 'variable');
  vars.push(valueName.value || 'value');
  return vars;
});

// 验证
const isValid = computed(() => {
  return idVars.value.length > 0 && valueVars.value.length > 0;
});

// 关闭对话框
const handleClose = () => {
  emit('update:visible', false);
  // 重置表单
  idVars.value = [];
  valueVars.value = [];
  variableName.value = '';
  valueName.value = '';
  sortColumn.value = '';
};

// 确认执行
const handleConfirm = async () => {
  if (!isValid.value) {
    ElMessage.warning('请至少选择一个标识列和一个值列');
    return;
  }

  loading.value = true;
  try {
    await unpivotData(
      idVars.value,
      valueVars.value,
      variableName.value || undefined,
      valueName.value || undefined,
      sortColumn.value || undefined
    );

    ElMessage.success('反透视操作成功');
    emit('success');
    handleClose();

    // 刷新数据
    await dataStore.refreshCurrentData();
  } catch (error) {
    console.error('反透视操作失败:', error);
    ElMessage.error(`反透视操作失败: ${error}`);
  } finally {
    loading.value = false;
  }
};
</script>

<style scoped>
.unpivot-content {
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

.hint-text {
  font-size: 12px;
  color: #909399;
  margin-top: 5px;
  line-height: 1.5;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.names-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
}
</style>
