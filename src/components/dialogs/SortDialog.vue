<template>
  <el-dialog
    :model-value="visible"
    align-center
    title="排序"
    width="520px"
    @close="handleClose"
    @update:model-value="(val: boolean) => $emit('update:visible', val)"
  >
    <el-form :model="formData" label-width="100px">
      <el-form-item label="排序列" required>
        <el-select v-model="formData.column" class="sort-select" clearable placeholder="选择排序列">
          <el-option v-for="col in columns" :key="col.name" :label="col.name" :value="col.name" />
        </el-select>
      </el-form-item>

      <el-form-item label="排序方向">
        <el-radio-group v-model="formData.descending">
          <el-radio-button :label="false">升序</el-radio-button>
          <el-radio-button :label="true">降序</el-radio-button>
        </el-radio-group>
      </el-form-item>

      <el-form-item label="空值位置">
        <el-switch v-model="formData.nullsLast" />
        <div class="sort-hint">开启后空值排在末尾</div>
      </el-form-item>
    </el-form>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button :disabled="!formData.column" type="primary" @click="handleConfirm">确认排序</el-button>
    </template>
  </el-dialog>
</template>

<script lang="ts" setup>
import { ref } from 'vue';
import type { ColumnInfo } from '@/types/dataset';

interface Props {
  visible: boolean;
  columns: ColumnInfo[];
}

defineProps<Props>();
const emit = defineEmits<{
  'update:visible': [value: boolean];
  confirm: [payload: { column: string; descending: boolean; nullsLast: boolean }];
}>();

const formData = ref({
  column: '',
  descending: false,
  nullsLast: false,
});

function handleClose() {
  emit('update:visible', false);
  formData.value = {
    column: '',
    descending: false,
    nullsLast: false,
  };
}

function handleConfirm() {
  if (!formData.value.column) {
    return;
  }

  emit('confirm', {
    column: formData.value.column,
    descending: formData.value.descending,
    nullsLast: formData.value.nullsLast,
  });
  handleClose();
}
</script>

<style scoped>
.sort-select {
  width: 100%;
}

.sort-hint {
  margin-left: 12px;
  color: var(--el-text-color-secondary);
  font-size: 12px;
}
</style>
