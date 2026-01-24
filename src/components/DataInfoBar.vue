<template>
  <div class="data-info-bar">
    <span class="info-text">
      当前数据帧: <span class="info-value">{{ datasetName }}</span>
    </span>
    <span class="separator">|</span>
    <span class="info-text">
      行数: <span class="info-value">{{ rows }}</span>
    </span>
    <span class="separator">|</span>
    <span class="info-text">
      列数: <span class="info-value">{{ columns }}</span>
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useDataStore } from '@/stores/dataStore';

const dataStore = useDataStore();

const datasetName = computed(() => dataStore.currentDataset?.name || 'df_main');
const rows = computed(() => dataStore.currentDataset?.rows.toLocaleString() || '0');
const columns = computed(() => dataStore.currentDataset?.columns.length || 0);
</script>

<style scoped>
.data-info-bar {
  height: 40px;
  background-color: #f5f7fa;
  border-bottom: 1px solid #e4e7ed;
  display: flex;
  align-items: center;
  padding: 0 16px;
  font-size: 12px;
  color: #909399;
}

.info-text {
  display: flex;
  align-items: center;
  gap: 4px;
}

.info-value {
  font-family: 'Courier New', monospace;
  color: #303133;
  font-weight: 500;
}

.separator {
  margin: 0 12px;
  color: #dcdfe6;
}
</style>