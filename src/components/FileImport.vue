<template>
  <el-button
    type="primary"
    @click="selectFile"
    :loading="loading"
    :icon="Upload"
  >
    导入 CSV/Excel
  </el-button>
</template>

<script setup lang="ts">
import { Upload } from '@element-plus/icons-vue';
import { open } from '@tauri-apps/plugin-dialog';
import { ElMessage } from 'element-plus';
import { ref } from 'vue';
import { useDataStore } from '@/stores/dataStore';

const dataStore = useDataStore();
const loading = ref(false);

async function selectFile() {
  try {
    const file = await open({
      multiple: false,
      filters: [
        {
          name: '数据文件',
          extensions: ['csv', 'xlsx', 'xls'],
        },
      ],
    });

    if (!file) return;

    loading.value = true;

    // file 是字符串路径
    const filePath = file;

    if (filePath.endsWith('.csv')) {
      await dataStore.importCSV(filePath);
      ElMessage.success('CSV 文件导入成功');
    } else if (filePath.endsWith('.xlsx') || filePath.endsWith('.xls')) {
      await dataStore.importExcel(filePath);
      ElMessage.success('Excel 文件导入成功');
    } else {
      ElMessage.error('不支持的文件格式');
    }
  } catch (e) {
    console.error('导入失败:', e);
    ElMessage.error(`导入失败: ${e}`);
  } finally {
    loading.value = false;
  }
}
</script>
