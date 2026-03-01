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
  background: linear-gradient(135deg, #ffffff 0%, #f8faff 100%);
  border-bottom: 1px solid rgba(0, 0, 0, 0.06);
  display: flex;
  align-items: center;
  padding: 0;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.04), 0 4px 12px rgba(0, 0, 0, 0.02);
  z-index: 10;
  /* backdrop-filter removed to prevent text blur in WebView */
}

.header-content {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;
  gap: 20px;
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
  gap: 10px;
}

.app-icon {
  width: 36px;
  height: 36px;
  display: block;
  filter: drop-shadow(0 2px 4px rgba(64, 158, 255, 0.2));
  transition: transform 0.3s ease;
}

.app-icon:hover {
  transform: scale(1.08) rotate(-3deg);
}

.app-title {
  font-size: 18px;
  font-weight: 700;
  color: #1d2129;
  margin: 0;
  letter-spacing: -0.02em;
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
  gap: 10px;
  padding: 6px 18px;
  background: linear-gradient(135deg, #f0f5ff 0%, #e8f0fe 100%);
  border: 1px solid rgba(64, 158, 255, 0.15);
  border-radius: 20px;
  color: #4e5969;
  font-size: 13px;
  transition: all 0.3s ease;
  box-shadow: 0 1px 3px rgba(64, 158, 255, 0.06);
}

.dataset-info:hover {
  border-color: rgba(64, 158, 255, 0.3);
  box-shadow: 0 2px 8px rgba(64, 158, 255, 0.12);
}

.dataset-icon {
  font-size: 15px;
  color: #409eff;
}

.dataset-name {
  font-weight: 600;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #1d2129;
}

.dataset-stats {
  font-family: 'SF Mono', 'Menlo', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  color: #4e5969;
  font-weight: 500;
}

.dataset-info .el-divider {
  background-color: rgba(64, 158, 255, 0.2);
  margin: 0;
}

.header-right {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.header-right .el-button {
  border-radius: 8px;
  font-weight: 500;
  font-size: 13px;
  padding: 8px 14px;
  transition: all 0.25s ease;
}

.header-right .el-button:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.main-container {
  height: calc(100vh - 56px);
  overflow: hidden;
}

.left-sidebar {
  background-color: #fafbfc;
  border-right: 1px solid rgba(0, 0, 0, 0.06);
  overflow: hidden;
  transition: box-shadow 0.3s ease;
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
  background: linear-gradient(165deg, #f6f8fc 0%, #eef2ff 40%, #ffffff 100%);
}

.empty-container {
  min-height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 40px 60px;
}

.welcome-section {
  max-width: 860px;
  width: 100%;
  animation: welcomeFadeIn 0.6s ease-out;
}

@keyframes welcomeFadeIn {
  from {
    opacity: 0;
    transform: translateY(16px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.welcome-header {
  text-align: center;
  margin-bottom: 40px;
}

.welcome-title {
  font-size: 34px;
  font-weight: 700;
  background: linear-gradient(135deg, #1d2129 0%, #409eff 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin: 0 0 14px 0;
  line-height: 1.3;
  letter-spacing: -0.02em;
}

.welcome-subtitle {
  font-size: 16px;
  color: #86909c;
  margin: 0;
  line-height: 1.7;
  font-weight: 400;
}

.quick-start {
  text-align: center;
  margin-bottom: 56px;
  padding: 36px 32px;
  background: linear-gradient(135deg, #ffffff 0%, #f7f9ff 100%);
  border-radius: 16px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.06), 0 1px 2px rgba(0, 0, 0, 0.04);
  border: 1px solid rgba(64, 158, 255, 0.08);
  transition: all 0.3s ease;
}

.quick-start:hover {
  box-shadow: 0 8px 32px rgba(64, 158, 255, 0.1), 0 2px 4px rgba(0, 0, 0, 0.04);
  border-color: rgba(64, 158, 255, 0.15);
}

.quick-start .el-button {
  margin-bottom: 20px;
  border-radius: 10px;
  padding: 12px 28px;
  font-size: 15px;
  font-weight: 600;
  box-shadow: 0 4px 14px rgba(64, 158, 255, 0.3);
  transition: all 0.3s ease;
}

.quick-start .el-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(64, 158, 255, 0.4);
}

.supported-formats {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  flex-wrap: wrap;
}

.supported-formats .el-tag {
  border-radius: 6px;
  font-weight: 500;
  letter-spacing: 0.02em;
}

.format-label {
  font-size: 13px;
  color: #86909c;
  margin-right: 4px;
}

.features-section {
  margin-top: 0;
}

.features-title {
  font-size: 18px;
  font-weight: 600;
  color: #1d2129;
  margin: 0 0 20px 0;
  text-align: center;
  letter-spacing: -0.01em;
}

.features-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
  gap: 14px;
}

.feature-item {
  display: flex;
  gap: 14px;
  padding: 20px;
  background: rgba(255, 255, 255, 0.85);
  border-radius: 12px;
  border: 1px solid rgba(0, 0, 0, 0.05);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  /* backdrop-filter removed to prevent text blur in WebView */
}

.feature-item:hover {
  border-color: rgba(64, 158, 255, 0.2);
  box-shadow: 0 8px 24px rgba(64, 158, 255, 0.1);
  transform: translateY(-3px);
  background: #ffffff;
}

.feature-icon {
  font-size: 28px;
  line-height: 1;
  flex-shrink: 0;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: linear-gradient(135deg, #f0f5ff 0%, #e8f4ff 100%);
  border-radius: 12px;
}

.feature-content {
  flex: 1;
}

.feature-content h4 {
  font-size: 15px;
  font-weight: 600;
  color: #1d2129;
  margin: 0 0 6px 0;
  letter-spacing: -0.01em;
}

.feature-content p {
  font-size: 13px;
  color: #86909c;
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
  border-left: 1px solid rgba(0, 0, 0, 0.06);
  box-shadow: -1px 0 6px rgba(0, 0, 0, 0.03);
  z-index: 10;
  overflow: hidden;
}
</style>
