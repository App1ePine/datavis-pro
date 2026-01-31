<script lang="ts" setup>
import { Delete, Document, Download, RefreshLeft, Upload } from '@element-plus/icons-vue';
import { getName, getVersion } from '@tauri-apps/api/app';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';
import { relaunch } from '@tauri-apps/plugin-process';
import { check } from '@tauri-apps/plugin-updater';
import { ElMessage, ElMessageBox, ElNotification } from 'element-plus';
import { computed, onMounted, onUnmounted, ref } from 'vue';
import AboutDialog from '@/components/AboutDialog.vue';
import DataGrid from '@/components/DataGrid.vue';
import ExportDialog from '@/components/dialogs/ExportDialog.vue';
import RightSidebar from '@/components/RightSidebar.vue';
import Sidebar from '@/components/Sidebar.vue';
import { useDataStore } from '@/stores/dataStore';

const dataStore = useDataStore();

// 软件信息
const appVersion = ref('');
const appName = ref('');

// 导出对话框状态
const exportDialogVisible = ref(false);
// 关于对话框状态
const aboutDialogVisible = ref(false);
let unlistenCheckUpdate: UnlistenFn | null = null;
let unlistenShowAbout: UnlistenFn | null = null;
let updateChecking = false;
let updateInstalling = false;

// 键盘快捷键处理
function handleKeyDown(event: KeyboardEvent) {
  // Ctrl+Z (Windows/Linux) 或 Cmd+Z (Mac) - Undo
  if ((event.ctrlKey || event.metaKey) && event.key === 'z' && !event.shiftKey) {
    event.preventDefault();
    handleUndo();
  }
  // Ctrl+Shift+Z (Windows/Linux) 或 Cmd+Shift+Z (Mac) - Redo
  else if ((event.ctrlKey || event.metaKey) && event.key === 'z' && event.shiftKey) {
    event.preventDefault();
    handleRedo();
  }
  // Ctrl+Y (Windows/Linux) - Redo 的替代快捷键
  else if (event.ctrlKey && event.key === 'y' && !event.metaKey) {
    event.preventDefault();
    handleRedo();
  }
}

onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  void listen('app://check-update', () => {
    handleCheckUpdate();
  }).then((unlisten) => {
    unlistenCheckUpdate = unlisten;
  });
  void listen('app://show-about', () => {
    aboutDialogVisible.value = true;
  }).then((unlisten) => {
    unlistenShowAbout = unlisten;
  });
});

onMounted(async () => {
  const version = await getVersion();
  appVersion.value = `v${version}`;
  appName.value = await getName();
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  if (unlistenCheckUpdate) {
    unlistenCheckUpdate();
    unlistenCheckUpdate = null;
  }
  if (unlistenShowAbout) {
    unlistenShowAbout();
    unlistenShowAbout = null;
  }
});

const hasData = computed(() => dataStore.currentData !== null);

// 加载提示文本
const loadingText = computed(() => {
  if (dataStore.loading) {
    return '正在处理数据，请稍候...';
  }
  return '';
});

async function handleImportData() {
  try {
    const file = await open({
      multiple: false,
      filters: [
        {
          name: '所有支持的格式',
          extensions: ['csv', 'tsv', 'xlsx', 'xls', 'parquet'],
        },
        {
          name: 'CSV 文件',
          extensions: ['csv'],
        },
        {
          name: 'TSV 文件',
          extensions: ['tsv'],
        },
        {
          name: 'Excel 文件',
          extensions: ['xlsx', 'xls'],
        },
        {
          name: 'Parquet 文件',
          extensions: ['parquet'],
        },
      ],
    });

    if (!file) return;

    if (file.endsWith('.csv') || file.endsWith('.tsv')) {
      await dataStore.importCSV(file);
      ElMessage.success(file.endsWith('.tsv') ? 'TSV 文件导入成功' : 'CSV 文件导入成功');
    } else if (file.endsWith('.xlsx') || file.endsWith('.xls')) {
      await dataStore.importExcel(file);
      ElMessage.success('Excel 文件导入成功');
    } else if (file.endsWith('.parquet')) {
      await dataStore.importParquet(file);
      ElMessage.success('Parquet 文件导入成功');
    }
  } catch (e) {
    console.error('导入失败:', e);
    ElMessage.error(`导入失败: ${e}`);
  }
}

async function handleExportData() {
  if (!dataStore.currentDataset) {
    ElMessage.warning('请先导入数据');
    return;
  }

  try {
    // 显示导出对话框
    exportDialogVisible.value = true;
  } catch (e) {
    console.error('导出数据失败:', e);
    ElMessage.error(`导出数据失败: ${e}`);
  }
}

// 处理导出确认
async function handleExportConfirm(format: 'csv' | 'parquet') {
  try {
    // 获取文件名并移除已有的后缀
    const baseName = (dataStore.currentDataset?.name || 'export').replace(/\.csv$/i, '').replace(/\.parquet$/i, '');

    // 根据选择的格式设置文件扩展名
    const extension = format === 'csv' ? 'csv' : 'parquet';
    const filterName = format === 'csv' ? 'CSV 文件' : 'Parquet 文件';

    const filePath = await save({
      filters: [
        {
          name: filterName,
          extensions: [extension],
        },
      ],
      defaultPath: `${baseName}.${extension}`,
    });

    if (!filePath) return;

    // 根据格式调用对应的导出函数
    if (format === 'parquet') {
      await dataStore.exportDatasetParquet(filePath);
    } else {
      await dataStore.exportDataset(filePath);
    }

    ElMessage.success('数据导出成功');
  } catch (e) {
    console.error('导出数据失败:', e);
    ElMessage.error(`导出数据失败: ${e}`);
  }
}

async function handleUndo() {
  if (!dataStore.canUndoFlag) {
    ElMessage.info('没有可撤销的操作');
    return;
  }

  try {
    await dataStore.undo();
    ElMessage.success('已撤销上一步操作');
  } catch (e) {
    console.error('撤销失败:', e);
    ElMessage.error(`撤销失败: ${e}`);
  }
}

async function handleRedo() {
  if (!dataStore.canRedoFlag) {
    ElMessage.info('没有可重做的操作');
    return;
  }

  try {
    await dataStore.redo();
    ElMessage.success('已重做操作');
  } catch (e) {
    console.error('重做失败:', e);
    ElMessage.error(`重做失败: ${e}`);
  }
}

async function handleResetData() {
  if (!dataStore.currentDataset) {
    ElMessage.warning('请先导入数据');
    return;
  }

  try {
    await ElMessageBox.confirm('确定要重置到刚导入时的数据吗？所有操作历史将被清除。', '确认重置', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    });

    await dataStore.resetToInitial();
    ElMessage.success('已重置到初始数据');
  } catch (e) {
    if (e !== 'cancel') {
      console.error('重置数据失败:', e);
      ElMessage.error(`重置数据失败: ${e}`);
    }
  }
}

async function handleClearData() {
  if (!dataStore.currentDataset) {
    ElMessage.warning('请先导入数据');
    return;
  }

  try {
    await ElMessageBox.confirm('确定要清空所有数据和历史吗？此操作不可恢复。', '确认清空', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning',
    });

    await dataStore.clearData();
    ElMessage.success('已清空所有数据');
  } catch (e) {
    if (e !== 'cancel') {
      console.error('清空数据失败:', e);
      ElMessage.error(`清空数据失败: ${e}`);
    }
  }
}

async function handleCheckUpdate() {
  if (updateChecking) return;
  updateChecking = true;

  try {
    const update = await check();

    if (!update) {
      ElNotification({
        title: '已是最新版本',
        message: '当前已是最新版本。',
        type: 'info',
        duration: 3500,
      });
      return;
    }

    const versionText = update.version ? `v${update.version}` : '新版本';
    const noticeText = update.body
      ? `${versionText} 已发布，点击此通知开始更新。`
      : `检测到 ${versionText}，点击此通知开始更新。`;

    ElNotification({
      title: '发现新版本',
      message: noticeText,
      type: 'success',
      duration: 0,
      onClick: async () => {
        if (updateInstalling) return;
        updateInstalling = true;

        ElNotification({
          title: '正在下载更新',
          message: '更新包下载中，请稍候...',
          type: 'info',
          duration: 3000,
        });

        try {
          await update.downloadAndInstall();
          ElNotification({
            title: '更新完成',
            message: '应用即将重启以完成更新。',
            type: 'success',
            duration: 2500,
          });
          await relaunch();
        } catch (e) {
          console.error('更新失败:', e);
          ElNotification({
            title: '更新失败',
            message: `更新过程中出现问题: ${e}`,
            type: 'error',
            duration: 4500,
          });
        } finally {
          updateInstalling = false;
        }
      },
    });
  } catch (e) {
    console.error('检查更新失败:', e);
    ElNotification({
      title: '检查更新失败',
      message: `无法检查更新: ${e}`,
      type: 'error',
      duration: 4500,
    });
  } finally {
    updateChecking = false;
  }
}
</script>

<template>
  <el-container
    v-loading="dataStore.loading"
    :element-loading-text="loadingText"
    class="app-container"
    element-loading-background="rgba(0, 0, 0, 0.7)"
  >
    <!-- Header -->
    <el-header class="app-header" height="56px">
      <div class="header-content">
        <div class="header-left">
          <div class="app-branding">
            <img alt="DataVis Pro" class="app-icon" src="/app-icon.svg" />
            <div class="app-title">{{ appName || 'DataVis Pro' }}</div>
            <el-tag size="small" type="info">{{ appVersion || 'v1.0.0' }}</el-tag>
          </div>
        </div>
        <div class="header-center">
          <div v-if="hasData" class="dataset-info">
            <el-icon class="dataset-icon"><Document /></el-icon>
            <span class="dataset-name">{{ dataStore.currentDataset?.name || '未命名数据集' }}</span>
            <el-divider direction="vertical" />
            <span class="dataset-stats">{{ dataStore.currentDataset?.rows.toLocaleString() || 0 }} 行</span>
            <el-divider direction="vertical" />
            <span class="dataset-stats">{{ dataStore.currentDataset?.columns.length || 0 }} 列</span>
          </div>
        </div>
        <div class="header-right">
          <el-button :icon="Upload" plain type="primary" @click="handleImportData">导入</el-button>
          <el-button :disabled="!hasData" :icon="Download" plain type="success" @click="handleExportData">
            导出
          </el-button>
          <el-button :disabled="!hasData" :icon="RefreshLeft" plain type="warning" @click="handleResetData">
            重置
          </el-button>
          <el-button :disabled="!hasData" :icon="Delete" plain type="danger" @click="handleClearData"> 清空 </el-button>
        </div>
      </div>
    </el-header>

    <el-container class="main-container">
      <!-- Left Sidebar -->
      <el-aside class="left-sidebar" width="280px">
        <Sidebar />
      </el-aside>

      <!-- Main Content -->
      <el-main class="content-area">
        <DataGrid v-if="hasData" />
        <div v-else class="empty-state">
          <div class="empty-container">
            <!-- 主要内容 -->
            <div class="welcome-section">
              <div class="welcome-header">
                <h1 class="welcome-title">欢迎使用 {{ appName }}</h1>
                <p class="welcome-subtitle">强大的数据分析工具，支持多种数据格式和丰富的数据处理功能</p>
              </div>

              <!-- 快速开始 -->
              <div class="quick-start">
                <el-button :icon="Upload" size="large" type="primary" @click="handleImportData">
                  导入数据文件
                </el-button>
                <div class="supported-formats">
                  <span class="format-label">支持格式：</span>
                  <el-tag effect="plain" size="small">CSV</el-tag>
                  <el-tag effect="plain" size="small">TSV</el-tag>
                  <el-tag effect="plain" size="small">Excel</el-tag>
                  <el-tag effect="plain" size="small">Parquet</el-tag>
                </div>
              </div>

              <!-- 功能特性 -->
              <div class="features-section">
                <h3 class="features-title">核心功能</h3>
                <div class="features-grid">
                  <div class="feature-item">
                    <div class="feature-icon">⚡</div>
                    <div class="feature-content">
                      <h4>高性能处理</h4>
                      <p>基于 Rust + Polars 引擎，轻松处理千万级数据</p>
                    </div>
                  </div>
                  <div class="feature-item">
                    <div class="feature-icon">🔍</div>
                    <div class="feature-content">
                      <h4>SQL 查询</h4>
                      <p>支持标准 SQL 语法进行数据筛选和分析</p>
                    </div>
                  </div>
                  <div class="feature-item">
                    <div class="feature-icon">🧹</div>
                    <div class="feature-content">
                      <h4>数据清洗</h4>
                      <p>提供 7 种空值填充策略和多种数据转换工具</p>
                    </div>
                  </div>
                  <div class="feature-item">
                    <div class="feature-icon">↶</div>
                    <div class="feature-content">
                      <h4>历史回溯</h4>
                      <p>完整的 Undo/Redo 支持，随时回退操作</p>
                    </div>
                  </div>
                  <div class="feature-item">
                    <div class="feature-icon">📊</div>
                    <div class="feature-content">
                      <h4>数据可视化</h4>
                      <p>内置多种图表类型，直观展示数据分析结果</p>
                    </div>
                  </div>
                  <div class="feature-item">
                    <div class="feature-icon">🔄</div>
                    <div class="feature-content">
                      <h4>数据转换</h4>
                      <p>支持长宽表转换、滑动窗口等高级操作</p>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </el-main>

      <!-- Right Sidebar -->
      <el-aside class="right-sidebar" width="300px">
        <RightSidebar />
      </el-aside>
    </el-container>

    <!-- 导出对话框 -->
    <ExportDialog v-model:visible="exportDialogVisible" @confirm="handleExportConfirm" />

    <!-- 关于对话框 -->
    <AboutDialog v-model="aboutDialogVisible" />
  </el-container>
</template>

<style scoped>
.app-container {
  height: 100vh;
  width: 100vw;
}

.app-header {
  background-color: #ffffff;
  border-bottom: 1px solid #e4e7ed;
  display: flex;
  align-items: center;
  padding: 0;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.05);
  z-index: 10;
}

.header-content {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 24px;
  gap: 24px;
}

.header-left {
  flex-shrink: 0;
  display: flex;
  align-items: center;
  gap: 12px;
}

.app-branding {
  display: flex;
  align-items: center;
  gap: 12px;
}

.app-icon {
  width: 56px;
  height: 56px;
  display: block;
}

.app-title {
  font-size: 24px;
  font-weight: bold;
  color: #303133;
  margin: 0;
}

.header-center {
  flex: 1;
  display: flex;
  justify-content: center;
  min-width: 0;
}

.dataset-info {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 20px;
  background: #f5f7fa;
  border: 1px solid #e4e7ed;
  border-radius: 20px;
  color: #606266;
  font-size: 13px;
}

.dataset-icon {
  font-size: 16px;
  color: #909399;
}

.dataset-name {
  font-weight: 600;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #303133;
}

.dataset-stats {
  font-family: 'Courier New', monospace;
  font-size: 12px;
  color: #606266;
}

.dataset-info .el-divider {
  background-color: #dcdfe6;
  margin: 0;
}

.header-right {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

.main-container {
  height: calc(100vh - 56px);
  overflow: hidden;
}

.left-sidebar {
  background-color: #f5f7fa;
  border-right: 1px solid #dcdfe6;
  overflow: hidden;
}

.content-area {
  background-color: #ffffff;
  padding: 16px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* ========== 空状态首页设计 ========== */
.empty-state {
  height: 100%;
  overflow-y: auto;
  background: linear-gradient(to bottom, #f5f7fa 0%, #ffffff 100%);
}

.empty-container {
  min-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 60px;
}

.welcome-section {
  max-width: 900px;
  width: 100%;
}

.welcome-header {
  text-align: center;
  margin-bottom: 48px;
}

.welcome-title {
  font-size: 32px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 16px 0;
  line-height: 1.3;
}

.welcome-subtitle {
  font-size: 16px;
  color: #606266;
  margin: 0;
  line-height: 1.6;
}

.quick-start {
  text-align: center;
  margin-bottom: 64px;
  padding: 32px;
  background: #ffffff;
  border-radius: 8px;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.quick-start .el-button {
  margin-bottom: 20px;
}

.supported-formats {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  flex-wrap: wrap;
}

.format-label {
  font-size: 14px;
  color: #909399;
  margin-right: 4px;
}

.features-section {
  margin-top: 48px;
}

.features-title {
  font-size: 20px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 24px 0;
  text-align: center;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 20px;
}

.feature-item {
  display: flex;
  gap: 16px;
  padding: 24px;
  background: #ffffff;
  border-radius: 8px;
  border: 1px solid #ebeef5;
  transition: all 0.3s ease;
}

.feature-item:hover {
  border-color: #409eff;
  box-shadow: 0 4px 12px rgba(64, 158, 255, 0.15);
  transform: translateY(-2px);
}

.feature-icon {
  font-size: 32px;
  line-height: 1;
  flex-shrink: 0;
}

.feature-content {
  flex: 1;
}

.feature-content h4 {
  font-size: 16px;
  font-weight: 600;
  color: #303133;
  margin: 0 0 8px 0;
}

.feature-content p {
  font-size: 14px;
  color: #606266;
  margin: 0;
  line-height: 1.6;
}

@media (max-width: 768px) {
  .empty-container {
    padding: 30px 20px;
  }

  .welcome-title {
    font-size: 24px;
  }

  .welcome-subtitle {
    font-size: 14px;
  }

  .features-grid {
    grid-template-columns: 1fr;
  }
}

.right-sidebar {
  background-color: #ffffff;
  border-left: 1px solid #dcdfe6;
  box-shadow: -2px 0 8px rgba(0, 0, 0, 0.05);
  z-index: 10;
  overflow: hidden;
}
</style>
