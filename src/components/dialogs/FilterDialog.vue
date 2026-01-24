<template>
  <el-dialog
    :model-value="visible"
    title="筛选过滤"
    width="700px"
    @update:model-value="(val: boolean) => $emit('update:visible', val)"
    @close="handleClose"
  >
    <div class="filter-content">
      <!-- 说明 -->
      <el-alert
        title="使用 SQL WHERE 条件进行数据筛选"
        type="info"
        :closable="false"
        show-icon
      >
        <template #default>
          <div class="alert-content">
            <p>支持标准 SQL WHERE 语法，示例：</p>
            <ul>
              <li><code>age > 18</code> - 年龄大于 18</li>
              <li><code>name = 'John'</code> - 名字等于 John</li>
              <li><code>score >= 60 AND score <= 100</code> - 分数在 60-100 之间</li>
              <li><code>status = 'active' OR status = 'pending'</code> - 状态为 active 或 pending</li>
              <li><code>value IS NOT NULL</code> - 值不为空</li>
              <li><code>name LIKE '%test%'</code> - 名字包含 test</li>
            </ul>
          </div>
        </template>
      </el-alert>

      <!-- 表达式输入 -->
      <div class="expression-section">
        <h4 class="section-title">WHERE 条件</h4>
        <el-input
          v-model="expression"
          type="textarea"
          :rows="4"
          placeholder='请输入 SQL WHERE 条件，例如：age > 18 AND status = "active"'
          class="expression-input"
        />
      </div>

      <!-- 可用列参考 -->
      <div class="columns-reference">
        <h4 class="section-title">
          可用列
          <span class="subtitle">（点击列名快速插入）</span>
        </h4>

        <!-- 搜索框 -->
        <el-input
          v-model="searchText"
          placeholder="搜索列名..."
          :prefix-icon="Search"
          clearable
          class="search-input"
        />

        <!-- 列列表 -->
        <div class="columns-list">
          <div v-for="col in filteredColumns" :key="col.name" class="column-item" @click="insertColumn(col.name)">
            <span class="column-name">{{ col.name }}</span>
            <el-tag size="small" type="info">{{ col.dtype }}</el-tag>
          </div>
        </div>
      </div>

      <!-- 常用操作符参考 -->
      <div class="operators-reference">
        <h4 class="section-title">常用操作符</h4>
        <div class="operators-grid">
          <el-button size="small" @click="insertOperator(' > ')">大于 ></el-button>
          <el-button size="small" @click="insertOperator(' < ')">小于 &lt;</el-button>
          <el-button size="small" @click="insertOperator(' >= ')">大于等于 >=</el-button>
          <el-button size="small" @click="insertOperator(' <= ')">小于等于 &lt;=</el-button>
          <el-button size="small" @click="insertOperator(' = ')">等于 =</el-button>
          <el-button size="small" @click="insertOperator(' != ')">不等于 !=</el-button>
          <el-button size="small" @click="insertOperator(' AND ')">AND (且)</el-button>
          <el-button size="small" @click="insertOperator(' OR ')">OR (或)</el-button>
          <el-button size="small" @click="insertOperator(' IS NULL')">IS NULL</el-button>
          <el-button size="small" @click="insertOperator(' IS NOT NULL')">IS NOT NULL</el-button>
          <el-button size="small" @click="insertOperator(' LIKE ')">LIKE (模糊)</el-button>
          <el-button size="small" @click="insertOperator(' IN ')">IN (包含)</el-button>
        </div>
      </div>

      <!-- 预览提示 -->
      <el-alert v-if="expression" type="warning" :closable="false" show-icon>
        <template #default>
          <strong>注意：</strong>筛选操作会保留满足条件的行，不满足条件的行将被删除。此操作可以通过撤销功能恢复。
        </template>
      </el-alert>
    </div>

    <template #footer>
      <el-button @click="handleClose">取消</el-button>
      <el-button type="primary" @click="handleConfirm" :disabled="!expression.trim()">
        确定筛选
      </el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { Search } from '@element-plus/icons-vue';
import { computed, ref } from 'vue';
import type { ColumnInfo } from '@/types/dataset';

interface Props {
  visible: boolean;
  columns: ColumnInfo[];
}

const props = defineProps<Props>();
const emit = defineEmits<{
  'update:visible': [value: boolean];
  confirm: [expression: string];
}>();

// 状态
const expression = ref('');
const searchText = ref('');

// 过滤后的列
const filteredColumns = computed(() => {
  if (!searchText.value) return props.columns;
  const search = searchText.value.toLowerCase();
  return props.columns.filter((col) => col.name.toLowerCase().includes(search));
});

// 插入列名
function insertColumn(columnName: string) {
  // 检查列名是否包含空格或特殊字符，如果有则用双引号包裹
  const needsQuotes = /[\s\-.()[\]{}]/.test(columnName);
  const quotedName = needsQuotes ? `"${columnName}"` : columnName;

  // 如果表达式为空，直接插入列名
  if (!expression.value) {
    expression.value = quotedName;
    return;
  }

  // 在末尾插入列名
  expression.value += ` ${quotedName}`;
}

// 插入操作符
function insertOperator(operator: string) {
  if (!expression.value) {
    return;
  }
  expression.value += operator;
}

// 关闭对话框
function handleClose() {
  emit('update:visible', false);
  // 重置状态
  expression.value = '';
  searchText.value = '';
}

// 确认操作
function handleConfirm() {
  const trimmedExpression = expression.value.trim();
  if (!trimmedExpression) {
    return;
  }

  emit('confirm', trimmedExpression);
  handleClose();
}
</script>

<style scoped>
.filter-content {
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

.subtitle {
  font-size: 12px;
  font-weight: normal;
  color: #909399;
  margin-left: 8px;
}

/* 说明区域 */
.alert-content p {
  margin: 0 0 8px 0;
  font-weight: 500;
}

.alert-content ul {
  margin: 0;
  padding-left: 20px;
}

.alert-content li {
  margin: 4px 0;
  font-size: 13px;
}

.alert-content code {
  background-color: #f5f7fa;
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  color: #e6a23c;
}

/* 表达式输入 */
.expression-section {
  padding-bottom: 20px;
  border-bottom: 1px solid #e4e7ed;
}

.expression-input :deep(textarea) {
  font-family: 'Courier New', monospace;
  font-size: 13px;
}

/* 列参考 */
.columns-reference {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.search-input {
  margin-bottom: 8px;
}

.columns-list {
  max-height: 160px;
  overflow-y: auto;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  padding: 8px;
  background-color: #fafafa;
}

.column-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.column-item:hover {
  background-color: #e6f7ff;
}

.column-name {
  font-family: 'Courier New', monospace;
  font-size: 13px;
  color: #303133;
  flex: 1;
}

/* 操作符参考 */
.operators-reference {
  padding-bottom: 20px;
  border-bottom: 1px solid #e4e7ed;
}

.operators-grid {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
}

.operators-grid .el-button {
  width: 100%;
  margin-left: 0 !important;
}
</style>