<template>
  <div class="right-sidebar">
    <div class="sidebar-header">
      <el-icon class="header-icon"><DataBoard /></el-icon>
      <div class="header-text">
        <div class="header-title">列统计</div>
        <div v-if="columnStats" class="header-subtitle">{{ columnStats.name }}</div>
        <div v-else class="header-subtitle">点击单元格查看</div>
      </div>
    </div>

    <el-scrollbar class="sidebar-content">
      <div v-if="columnStats" class="stats-section">
        <!-- 基础统计 - 根据实际数据判断显示 -->
        <template v-if="hasBasicStats">
          <div class="stats-card">
            <div class="card-header">
              <h4 class="card-title">基础统计</h4>
              <el-tag size="small" type="info">{{ columnStats.dtype }}</el-tag>
            </div>

            <!-- 数值类型的基础统计 -->
            <div v-if="hasNumericStats" class="stats-grid">
              <div class="stat-item">
                <span class="stat-label">最大值</span>
                <span class="stat-value">{{ columnStats.max !== null ? columnStats.max.toFixed(2) : 'N/A' }}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">最小值</span>
                <span class="stat-value">{{ columnStats.min !== null ? columnStats.min.toFixed(2) : 'N/A' }}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">平均值</span>
                <span class="stat-value">{{ columnStats.mean !== null ? columnStats.mean.toFixed(2) : 'N/A' }}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">标准差</span>
                <span class="stat-value">{{ columnStats.std !== null ? columnStats.std.toFixed(2) : 'N/A' }}</span>
              </div>
            </div>

            <!-- 日期时间类型的基础统计 -->
            <div v-else-if="hasDatetimeStats" class="stats-grid">
              <div class="stat-item">
                <span class="stat-label">最早时间</span>
                <span class="stat-value">{{ columnStats.min_datetime || 'N/A' }}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">最晚时间</span>
                <span class="stat-value">{{ columnStats.max_datetime || 'N/A' }}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">时间跨度 (天)</span>
                <span class="stat-value">{{
                  columnStats.datetime_range_days !== null ? columnStats.datetime_range_days.toFixed(2) : 'N/A'
                }}</span>
              </div>
            </div>

            <!-- 布尔类型的基础统计 -->
            <div v-else-if="hasBooleanStats" class="stats-grid">
              <div class="stat-item">
                <span class="stat-label">True 数量</span>
                <span class="stat-value">{{
                  columnStats.true_count !== null ? columnStats.true_count.toLocaleString() : 'N/A'
                }}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">False 数量</span>
                <span class="stat-value">{{
                  columnStats.false_count !== null ? columnStats.false_count.toLocaleString() : 'N/A'
                }}</span>
              </div>
            </div>
          </div>
        </template>

        <!-- 分布概览 - 仅当有分位数数据时显示 -->
        <template v-if="hasDistributionStats">
          <div class="stats-card">
            <div class="card-header">
              <h4 class="card-title">分布概览</h4>
            </div>
            <div class="stats-grid">
              <div class="stat-item">
                <span class="stat-label">25% 分位值</span>
                <span class="stat-value">{{ columnStats.q25 !== null ? columnStats.q25.toFixed(2) : 'N/A' }}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">50% 分位值</span>
                <span class="stat-value">{{ columnStats.q50 !== null ? columnStats.q50.toFixed(2) : 'N/A' }}</span>
              </div>
              <div class="stat-item">
                <span class="stat-label">75% 分位值</span>
                <span class="stat-value">{{ columnStats.q75 !== null ? columnStats.q75.toFixed(2) : 'N/A' }}</span>
              </div>
            </div>
          </div>
        </template>

        <!-- 数据质量 - 始终显示（基于 total_count, null_count, unique_count） -->
        <template v-if="hasQualityStats">
          <div class="stats-card quality">
            <div class="card-header">
              <h4 class="card-title">数据质量</h4>
              <el-tag v-if="!hasBasicStats" size="small" type="info">{{ columnStats.dtype }}</el-tag>
            </div>
            <div class="quality-items">
              <div class="quality-item">
                <div class="quality-label">总数目</div>
                <div class="quality-value">{{ columnStats.total_count.toLocaleString() }}</div>
              </div>
              <div class="quality-item">
                <div class="quality-label">缺失值</div>
                <div :class="columnStats.null_count === 0 ? 'success' : 'error'" class="quality-value">
                  {{ columnStats.null_count }} ({{
                    ((columnStats.null_count / columnStats.total_count) * 100).toFixed(1)
                  }}%)
                </div>
              </div>
              <div class="quality-item">
                <div class="quality-label">唯一值</div>
                <div class="quality-value">{{ columnStats.unique_count }}</div>
              </div>
            </div>
          </div>
        </template>
      </div>

      <div v-else class="empty-container">
        <el-empty v-if="!dataStore.hasData" :image-size="100" description="暂无数据" />
        <el-empty v-else :image-size="100" description="请点击表格中的任意单元格查看列统计信息" />
      </div>
    </el-scrollbar>
  </div>
</template>

<script setup lang="ts">
import { DataBoard } from '@element-plus/icons-vue';
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
  padding: 20px;
  background-color: #ffffff;
  border-bottom: 1px solid #e4e7ed;
  display: flex;
  align-items: center;
  gap: 12px;
}

.header-icon {
  font-size: 28px;
  line-height: 1;
}

.header-text {
  flex: 1;
  min-width: 0;
}

.header-title {
  font-size: 16px;
  font-weight: 700;
  color: #303133;
  line-height: 1.3;
  margin-bottom: 2px;
}

.header-subtitle {
  font-size: 12px;
  color: #909399;
  line-height: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.sidebar-content {
  flex: 1;
  height: 0;
  background-color: #f5f7fa;
}

.sidebar-content :deep(.el-scrollbar__view) {
  height: 100%;
}

.empty-container {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
}

.empty-content {
  text-align: center;
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 16px;
  opacity: 0.3;
}

.empty-text {
  font-size: 14px;
  color: #606266;
  margin-bottom: 8px;
  font-weight: 500;
}

.empty-subtext {
  font-size: 12px;
  color: #909399;
}

.stats-section {
  padding: 16px;
}

.stats-card {
  background: #ffffff;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
  border: 1px solid #ebeef5;
}

.stats-card.quality {
  background: linear-gradient(135deg, #fff5f5 0%, #ffffff 100%);
}

.card-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 16px;
  padding-bottom: 12px;
  border-bottom: 2px solid #f0f0f0;
}

.card-title {
  font-size: 14px;
  font-weight: 600;
  color: #303133;
  margin: 0;
}

.stats-grid {
  display: grid;
  grid-template-columns: 1fr;
  gap: 12px;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 12px;
  background: #f5f7fa;
  border-radius: 4px;
  transition: all 0.2s;
}

.stat-item:hover {
  background: #ecf5ff;
}

.stat-label {
  font-size: 13px;
  color: #606266;
  font-weight: 500;
}

.stat-value {
  font-family: 'Courier New', monospace;
  font-size: 14px;
  font-weight: 700;
  color: #303133;
}

.quality-items {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.quality-item {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  background: rgba(255, 255, 255, 0.8);
  border-radius: 6px;
  border: 1px solid #f0f0f0;
}

.quality-label {
  font-size: 12px;
  color: #909399;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.quality-value {
  font-family: 'Courier New', monospace;
  font-size: 18px;
  font-weight: 700;
  color: #303133;
}

.quality-value.error {
  color: #f56c6c;
}

.quality-value.success {
  color: #67c23a;
}
</style>
