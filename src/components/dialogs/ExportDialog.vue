<template>
  <el-dialog
    :model-value="visible"
    title="导出数据 - Export Data"
    width="500px"
    @update:model-value="(val: boolean) => $emit('update:visible', val)"
    @close="handleClose"
    align-center
  >
    <div class="export-content">
      <!-- 格式选择 -->
      <div class="format-section">
        <h4 class="section-title">选择导出格式</h4>
        <div class="format-grid">
          <div class="format-card" :class="{ active: selectedFormat === 'csv' }" @click="selectedFormat = 'csv'">
            <div class="format-icon">📄</div>
            <div class="format-name">CSV</div>
            <div class="format-desc">通用表格格式</div>
          </div>
          <div
            class="format-card"
            :class="{ active: selectedFormat === 'parquet' }"
            @click="selectedFormat = 'parquet'"
          >
            <div class="format-icon">📦</div>
            <div class="format-name">Parquet</div>
            <div class="format-desc">高效列式存储</div>
          </div>
        </div>
      </div>

      <!-- 格式说明 -->
      <el-alert v-if="selectedFormat === 'csv'" type="info" :closable="false" show-icon>
        <template #default>
          <strong>CSV 格式：</strong>纯文本格式，兼容性好，可用 Excel 直接打开，但文件较大。
        </template>
      </el-alert>
      <el-alert v-if="selectedFormat === 'parquet'" type="info" :closable="false" show-icon>
        <template #default>
          <strong>Parquet 格式：</strong>列式存储，压缩率高，适合大数据处理，需要专业工具读取。
        </template>
      </el-alert>
    </div>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" @click="handleConfirm"> 确认导出 </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref } from 'vue';

interface Props {
  visible: boolean;
}

defineProps<Props>();
const emit = defineEmits<{
  'update:visible': [value: boolean];
  confirm: [format: 'csv' | 'parquet'];
}>();

// 状态
const selectedFormat = ref<'csv' | 'parquet'>('csv');

// 关闭对话框
function handleClose() {
  emit('update:visible', false);
  // 重置状态
  selectedFormat.value = 'csv';
}

// 确认操作
function handleConfirm() {
  emit('confirm', selectedFormat.value);
  handleClose();
}
</script>

<style scoped>
.export-content {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 12px 0;
}

/* 格式选择 */
.format-section {
  padding-bottom: 20px;
  border-bottom: 1px solid #e4e7ed;
}

.format-grid {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 16px;
}

.format-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 24px 16px;
  border: 2px solid #e4e7ed;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s;
  background-color: #fff;
}

.format-card:hover {
  border-color: #409eff;
  background-color: #f0f9ff;
}

.format-card.active {
  border-color: #409eff;
  background-color: #ecf5ff;
  box-shadow: 0 2px 8px rgba(64, 158, 255, 0.2);
}

.format-icon {
  font-size: 48px;
  margin-bottom: 12px;
}

.format-name {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin-bottom: 4px;
}

.format-desc {
  font-size: 12px;
  color: #909399;
  text-align: center;
}
</style>
