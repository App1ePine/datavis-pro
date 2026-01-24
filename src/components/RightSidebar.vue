<template>
  <div class="right-sidebar">
    <div class="sidebar-header">
      <span class="header-title">列统计信息</span>
      <el-tag v-if="columnStats" size="small" type="primary" class="column-tag" :title="columnStats.name">
        {{ columnStats.name }}
      </el-tag>
    </div>

    <el-scrollbar class="sidebar-content">
      <div v-if="columnStats" class="stats-section">
        <!-- 基础统计 - 根据实际数据判断显示 -->
        <template v-if="hasBasicStats">
          <h4 class="section-title">
            基础统计
            <span class="dtype-suffix">{{ columnStats.dtype }}</span>
          </h4>

          <!-- 数值类型的基础统计 -->
          <el-descriptions v-if="hasNumericStats" direction="vertical" :column="1" border size="small">
            <el-descriptions-item label="最大值">
              {{ columnStats.max !== null ? columnStats.max.toFixed(2) : 'N/A' }}
            </el-descriptions-item>
            <el-descriptions-item label="最小值">
              {{ columnStats.min !== null ? columnStats.min.toFixed(2) : 'N/A' }}
            </el-descriptions-item>
            <el-descriptions-item label="平均值">
              {{ columnStats.mean !== null ? columnStats.mean.toFixed(2) : 'N/A' }}
            </el-descriptions-item>
            <el-descriptions-item label="标准差">
              {{ columnStats.std !== null ? columnStats.std.toFixed(2) : 'N/A' }}
            </el-descriptions-item>
          </el-descriptions>

          <!-- 日期时间类型的基础统计 -->
          <el-descriptions v-else-if="hasDatetimeStats" direction="vertical" :column="1" border size="small">
            <el-descriptions-item label="最早时间">
              {{ columnStats.min_datetime || 'N/A' }}
            </el-descriptions-item>
            <el-descriptions-item label="最晚时间">
              {{ columnStats.max_datetime || 'N/A' }}
            </el-descriptions-item>
            <el-descriptions-item label="时间跨度 (天)">
              {{ columnStats.datetime_range_days !== null ? columnStats.datetime_range_days.toFixed(2) : 'N/A' }}
            </el-descriptions-item>
          </el-descriptions>

          <!-- 布尔类型的基础统计 -->
          <el-descriptions v-else-if="hasBooleanStats" direction="vertical" :column="1" border size="small">
            <el-descriptions-item label="True 数量">
              {{ columnStats.true_count !== null ? columnStats.true_count.toLocaleString() : 'N/A' }}
            </el-descriptions-item>
            <el-descriptions-item label="False 数量">
              {{ columnStats.false_count !== null ? columnStats.false_count.toLocaleString() : 'N/A' }}
            </el-descriptions-item>
          </el-descriptions>

          <div class="spacer"></div>
        </template>

        <!-- 分布概览 - 仅当有分位数数据时显示 -->
        <template v-if="hasDistributionStats">
          <h4 class="section-title distribution">分布概览</h4>
          <el-descriptions direction="vertical" :column="1" border size="small">
            <el-descriptions-item label="25% 分位值">
              {{ columnStats.q25 !== null ? columnStats.q25.toFixed(2) : 'N/A' }}
            </el-descriptions-item>
            <el-descriptions-item label="50% 分位值">
              {{ columnStats.q50 !== null ? columnStats.q50.toFixed(2) : 'N/A' }}
            </el-descriptions-item>
            <el-descriptions-item label="75% 分位值">
              {{ columnStats.q75 !== null ? columnStats.q75.toFixed(2) : 'N/A' }}
            </el-descriptions-item>
          </el-descriptions>

          <div class="spacer"></div>
        </template>

        <!-- 数据质量 - 始终显示（基于 total_count, null_count, unique_count） -->
        <template v-if="hasQualityStats">
          <h4 class="section-title quality">
            数据质量
            <span v-if="!hasBasicStats" class="dtype-suffix">{{ columnStats.dtype }}</span>
          </h4>
          <div class="quality-card">
            <div class="quality-item">
              <span class="quality-label">总数目:</span>
              <span class="quality-value">{{ columnStats.total_count.toLocaleString() }}</span>
            </div>
            <div class="quality-item">
              <span class="quality-label">缺失值 (Null):</span>
              <span class="quality-value" :class="columnStats.null_count === 0 ? 'success' : 'error'">
                {{ columnStats.null_count }} ({{ ((columnStats.null_count / columnStats.total_count) * 100).toFixed(1) }}%)
              </span>
            </div>
            <div class="quality-item">
              <span class="quality-label">唯一值 (Unique):</span>
              <span class="quality-value">{{ columnStats.unique_count }}</span>
            </div>
          </div>
        </template>
      </div>

      <el-empty v-else description="请点击表格中的任意单元格查看列统计信息" :image-size="100" />
    </el-scrollbar>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useDataStore } from '@/stores/dataStore';

const dataStore = useDataStore();

const columnStats = computed(() => dataStore.currentColumnStats);

// ==================== 基于数据的判断逻辑 ====================

// 判断是否有数值类型的基础统计（max, min, mean, std）
const hasNumericStats = computed(() => {
  if (!columnStats.value) return false;
  return (
    columnStats.value.max !== null ||
    columnStats.value.min !== null ||
    columnStats.value.mean !== null ||
    columnStats.value.std !== null
  );
});

// 判断是否有日期时间类型的基础统计
const hasDatetimeStats = computed(() => {
  if (!columnStats.value) return false;
  return (
    columnStats.value.min_datetime !== null ||
    columnStats.value.max_datetime !== null ||
    columnStats.value.datetime_range_days !== null
  );
});

// 判断是否有布尔类型的基础统计
const hasBooleanStats = computed(() => {
  if (!columnStats.value) return false;
  return columnStats.value.true_count !== null || columnStats.value.false_count !== null;
});

// 判断是否有基础统计（任意一种）
const hasBasicStats = computed(() => {
  return hasNumericStats.value || hasDatetimeStats.value || hasBooleanStats.value;
});

// 判断是否有分布概览（分位数统计）
const hasDistributionStats = computed(() => {
  if (!columnStats.value) return false;
  return columnStats.value.q25 !== null || columnStats.value.q50 !== null || columnStats.value.q75 !== null;
});

// 判断是否有数据质量统计（总是有，因为 total_count, null_count, unique_count 总是存在）
const hasQualityStats = computed(() => {
  return columnStats.value !== null;
});
</script>

<style scoped>
.right-sidebar {
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: #ffffff;
}

.sidebar-header {
  padding: 12px 16px;
  background-color: #ecf5ff;
  border-bottom: 1px solid #d9ecff;
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-title {
  color: #409eff;
  font-weight: 500;
  font-size: 14px;
  flex-shrink: 0;
  white-space: nowrap;
}

.column-tag {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-content {
  flex: 1;
  height: 0;
}

.stats-section {
  padding: 16px;
}

.section-title {
  font-size: 15px;
  font-weight: bold;
  color: #303133;
  margin: 0 0 12px 0;
  padding-left: 8px;
  border-left: 4px solid #409eff;
}

.dtype-suffix {
  font-size: 12px;
  font-weight: normal;
  color: #909399;
  margin-left: 8px;
}

.section-title.distribution {
  border-left-color: #67c23a;
}

.section-title.quality {
  border-left-color: #e6a23c;
}

.spacer {
  height: 16px;
}

.quality-card {
  background-color: #f5f7fa;
  border-radius: 4px;
  padding: 12px;
  font-size: 13px;
}

.quality-item {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
}

.quality-item:last-child {
  margin-bottom: 0;
}

.quality-label {
  color: #909399;
}

.quality-value {
  font-family: 'Courier New', monospace;
  font-weight: 500;
  color: #303133;
}

.quality-value.error {
  color: #f56c6c;
  font-weight: bold;
}

.quality-value.success {
  color: #67c23a;
  font-weight: bold;
}
</style>