<script setup lang="ts">
import { Delete, Download, RefreshLeft, Upload } from '@element-plus/icons-vue';
import { getName, getVersion } from '@tauri-apps/api/app';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';
import { relaunch } from '@tauri-apps/plugin-process';
import { check } from '@tauri-apps/plugin-updater';
import { ElMessage, ElMessageBox, ElNotification } from 'element-plus';
import { computed, onMounted, onUnmounted, ref } from 'vue';
import DataGrid from '@/components/DataGrid.vue';
import DataInfoBar from '@/components/DataInfoBar.vue';
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
let unlistenCheckUpdate: UnlistenFn | null = null;
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
    class="app-container"
    v-loading="dataStore.loading"
    :element-loading-text="loadingText"
    element-loading-background="rgba(0, 0, 0, 0.7)"
  >
    <!-- Header -->
    <el-header class="app-header" height="56px">
      <div class="header-left">
        <div class="app-title">{{ appName || '...' }}</div>
        <el-tag type="info" size="small">{{ appVersion || 'v--' }}</el-tag>
      </div>
      <div class="header-right">
        <el-button type="primary" plain :icon="Upload" @click="handleImportData"> 导入数据 </el-button>
        <el-button type="success" plain :icon="Download" @click="handleExportData" :disabled="!hasData">
          导出数据
        </el-button>
        <el-button type="warning" plain :icon="RefreshLeft" @click="handleResetData" :disabled="!hasData">
          重置数据
        </el-button>
        <el-button type="danger" plain :icon="Delete" @click="handleClearData" :disabled="!hasData">
          清空数据
        </el-button>
      </div>
    </el-header>

    <el-container class="main-container">
      <!-- Left Sidebar -->
      <el-aside width="280px" class="left-sidebar">
        <Sidebar />
      </el-aside>

      <!-- Main Content -->
      <el-main class="content-area">
        <DataInfoBar v-if="hasData" />
        <div class="data-content">
          <div v-if="hasData" class="data-view">
            <DataGrid />
          </div>
          <div v-else class="empty-state">
            <div class="empty-content">
              <div class="empty-icon">▦</div>
              <div class="empty-title">AG Grid Vue 3 Component Area</div>
              <div class="empty-subtitle">Dataframe renders here</div>
              <div class="empty-hint">请点击顶部"导入数据"按钮开始</div>
            </div>
          </div>
        </div>
      </el-main>

      <!-- Right Sidebar -->
      <el-aside width="300px" class="right-sidebar">
        <RightSidebar />
      </el-aside>
    </el-container>

    <!-- 导出对话框 -->
    <ExportDialog v-model:visible="exportDialogVisible" @confirm="handleExportConfirm" />
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
  justify-content: space-between;
  padding: 0 16px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.05);
  z-index: 10;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.app-title {
  font-size: 20px;
  font-weight: bold;
  color: #303133;
  margin: 0;
}

.header-right {
  display: flex;
  gap: 8px;
}

.main-container {
  height: calc(100vh - 56px);
  overflow: hidden;
}

.left-sidebar {
  background-color: #f5f7fa;
  border-right: 1px solid #e4e7ed;
  overflow: hidden;
}

.content-area {
  background-color: #ffffff;
  padding: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.data-content {
  flex: 1;
  overflow: hidden;
  position: relative;
}

.data-view {
  height: 100%;
  padding: 16px;
}

.empty-state {
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.empty-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border: 2px dashed #dcdfe6;
  border-radius: 8px;
  background-color: #fafafa;
}

.empty-icon {
  font-size: 64px;
  color: #c0c4cc;
  margin-bottom: 16px;
}

.empty-title {
  font-size: 16px;
  font-weight: 500;
  color: #606266;
  margin: 0 0 8px 0;
}

.empty-subtitle {
  font-size: 14px;
  color: #909399;
  margin: 0 0 20px 0;
}

.empty-hint {
  font-size: 13px;
  color: #c0c4cc;
  margin: 0;
}

.right-sidebar {
  background-color: #ffffff;
  border-left: 1px solid #e4e7ed;
  box-shadow: -2px 0 8px rgba(0, 0, 0, 0.05);
  z-index: 10;
  overflow: hidden;
}
</style>
