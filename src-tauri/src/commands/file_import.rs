// ============================================================================
// commands/file_import.rs - 文件导入命令
// ============================================================================
// 这个文件定义了文件导入相关的 Tauri 命令
// 前端可以调用这些命令来导入 CSV 和 Excel 文件

// ============================================================================
// 导入依赖
// ============================================================================
use crate::data::{create_dataset_info, load_csv, load_excel, load_parquet};  // 数据加载函数
use crate::models::{DatasetInfo, HistoryEntry, OperationType};  // 数据集元信息类型
use crate::AppState;              // 应用状态
use uuid::Uuid;                   // UUID 生成器

// ============================================================================
// CSV 文件导入命令
// ============================================================================
/// 导入 CSV 文件
///
/// 这是一个 Tauri 命令，前端可以通过 invoke('import_csv', { filePath }) 调用
///
/// 工作流程：
/// 1. 接收文件路径
/// 2. 使用 Polars 读取 CSV 文件
/// 3. 生成唯一的数据集 ID（UUID）
/// 4. 提取数据集元信息
/// 5. 将 DataFrame 和元信息存储到 DataStore
/// 6. 返回元信息给前端
///
/// 参数：
/// - file_path: CSV 文件的完整路径（由前端传入）
/// - state: 应用状态（Tauri 自动注入，包含 DataStore）
///
/// 返回：
/// - Result<DatasetInfo, String>:
///   - 成功：返回数据集元信息（包含 ID、名称、行数、列信息等）
///   - 失败：返回错误消息字符串
///
/// 前端调用示例：
/// ```typescript
/// const info = await invoke('import_csv', {
///   filePath: '/path/to/data.csv'
/// });
/// console.log(`导入成功，数据集 ID: ${info.id}`);
/// ```
#[tauri::command]  // 这个宏将函数注册为 Tauri 命令
pub async fn import_csv(
    file_path: String,                    // 文件路径（前端传入）
    state: tauri::State<'_, AppState>,    // 应用状态（自动注入）
) -> Result<DatasetInfo, String> {        // 返回类型
    // ------------------------------------------------------------------------
    // 步骤 1: 读取 CSV 文件
    // ------------------------------------------------------------------------
    // load_csv: 使用 Polars 读取 CSV 文件，返回 DataFrame
    // map_err: 将错误转换为字符串（Tauri 要求错误类型是 String）
    // format!: 格式化错误消息
    // ?: 如果失败，立即返回错误
    let df = load_csv(&file_path)
        .map_err(|e| format!("Failed to load CSV: {}", e))?;

    // ------------------------------------------------------------------------
    // 步骤 2: 生成唯一 ID 和时间戳
    // ------------------------------------------------------------------------
    let id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    // ------------------------------------------------------------------------
    // 步骤 3: 创建数据集元信息
    // ------------------------------------------------------------------------
    let info = create_dataset_info(&id, &file_path, &df);

    // ------------------------------------------------------------------------
    // 步骤 4: 创建操作类型和描述
    // ------------------------------------------------------------------------
    let operation = OperationType::Import {
        file_path: file_path.clone(),
    };
    let description = operation.description();

    // ------------------------------------------------------------------------
    // 步骤 5: 创建历史条目
    // ------------------------------------------------------------------------
    let entry = HistoryEntry {
        id: id.clone(),
        operation,
        dataframe: df,
        metadata: info.clone(),
        timestamp,
        description,
    };

    // ------------------------------------------------------------------------
    // 步骤 6: 清空历史并添加新条目
    // ------------------------------------------------------------------------
    let mut store = state.data_store.lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    // 清空旧历史（导入新文件时重置）
    store.clear();

    // 添加导入操作到历史栈
    store.push_operation(entry);

    // ------------------------------------------------------------------------
    // 步骤 7: 返回元信息
    // ------------------------------------------------------------------------
    Ok(info)
}

// ============================================================================
// Excel 文件导入命令
// ============================================================================
/// 导入 Excel 文件
///
/// 这是一个 Tauri 命令，前端可以通过 invoke('import_excel', { filePath, sheetName }) 调用
///
/// 工作流程：
/// 1. 接收文件路径和可选的工作表名称
/// 2. 使用 Calamine 读取 Excel 文件
/// 3. 生成唯一的数据集 ID
/// 4. 提取数据集元信息
/// 5. 存储到 DataStore
/// 6. 返回元信息给前端
///
/// 参数：
/// - file_path: Excel 文件的完整路径
/// - sheet_name: 可选的工作表名称
///   - Some("Sheet1"): 读取指定的工作表
///   - None: 读取第一个工作表
/// - state: 应用状态（自动注入）
///
/// 返回：
/// - Result<DatasetInfo, String>: 成功返回元信息，失败返回错误
///
/// 前端调用示例：
/// ```typescript
/// // 读取第一个工作表
/// const info = await invoke('import_excel', {
///   filePath: '/path/to/data.xlsx'
/// });
///
/// // 读取指定工作表
/// const info = await invoke('import_excel', {
///   filePath: '/path/to/data.xlsx',
///   sheetName: '销售数据'
/// });
/// ```
#[tauri::command]
pub async fn import_excel(
    file_path: String,                    // 文件路径
    sheet_name: Option<String>,           // 可选的工作表名称
    state: tauri::State<'_, AppState>,    // 应用状态
) -> Result<DatasetInfo, String> {
    // ------------------------------------------------------------------------
    // 步骤 1: 读取 Excel 文件
    // ------------------------------------------------------------------------
    // load_excel: 使用 Calamine 读取 Excel 文件
    // sheet_name: 传递工作表名称（可能是 None）
    let df = load_excel(&file_path, sheet_name)
        .map_err(|e| format!("Failed to load Excel: {}", e))?;

    // ------------------------------------------------------------------------
    // 步骤 2-7: 与 CSV 导入相同
    // ------------------------------------------------------------------------
    let id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();
    let info = create_dataset_info(&id, &file_path, &df);

    let operation = OperationType::Import {
        file_path: file_path.clone(),
    };
    let description = operation.description();

    let entry = HistoryEntry {
        id: id.clone(),
        operation,
        dataframe: df,
        metadata: info.clone(),
        timestamp,
        description,
    };

    let mut store = state.data_store.lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;
    store.clear();
    store.push_operation(entry);

    Ok(info)
}

// ============================================================================
// Parquet 文件导入命令
// ============================================================================
/// 导入 Parquet 文件
///
/// 这是一个 Tauri 命令，前端可以通过 invoke('import_parquet', { filePath }) 调用
///
/// Parquet 是一种高效的列式存储格式：
/// - 读取速度快（比 CSV 快 10-100 倍）
/// - 文件体积小（压缩率高）
/// - 保留完整的数据类型信息（不需要类型推断）
///
/// 参数：
/// - file_path: Parquet 文件的完整路径
/// - state: 应用状态（自动注入）
///
/// 返回：
/// - Result<DatasetInfo, String>: 成功返回元信息，失败返回错误
///
/// 前端调用示例：
/// ```typescript
/// const info = await invoke('import_parquet', {
///   filePath: '/path/to/data.parquet'
/// });
/// ```
#[tauri::command]
pub async fn import_parquet(
    file_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<DatasetInfo, String> {
    // 读取 Parquet 文件
    let df = load_parquet(&file_path)
        .map_err(|e| format!("Failed to load Parquet: {}", e))?;

    // 生成 UUID 和时间戳
    let id = Uuid::new_v4().to_string();
    let timestamp = chrono::Utc::now().to_rfc3339();

    // 创建元信息
    let info = create_dataset_info(&id, &file_path, &df);

    // 创建操作和历史条目
    let operation = OperationType::Import {
        file_path: file_path.clone(),
    };
    let description = operation.description();

    let entry = HistoryEntry {
        id: id.clone(),
        operation,
        dataframe: df,
        metadata: info.clone(),
        timestamp,
        description,
    };

    // 清空历史并添加新条目
    let mut store = state.data_store.lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;
    store.clear();
    store.push_operation(entry);

    Ok(info)
}

// ============================================================================
// 错误处理说明
// ============================================================================
//
// 1. map_err 的作用：
//    - 将各种错误类型转换为 String
//    - Tauri 命令的错误类型必须是 String
//    - format! 宏用于创建格式化的错误消息
//
// 2. ? 操作符的作用：
//    - 如果 Result 是 Err，立即返回错误
//    - 如果 Result 是 Ok，提取值并继续执行
//    - 简化了错误处理代码
//
// 3. 错误传播链：
//    Polars/Calamine 错误 → DataAnalystError → String → 前端
//
// ============================================================================

// ============================================================================
// 线程安全说明
// ============================================================================
//
// 1. 为什么需要 lock()？
//    - DataStore 被包装在 Arc<Mutex<...>> 中
//    - 多个 Tauri 命令可能同时执行
//    - Mutex 确保同一时间只有一个命令可以修改数据
//
// 2. 锁的生命周期：
//    - lock() 返回 MutexGuard
//    - MutexGuard 在离开作用域时自动释放锁
//    - 在这个函数中，锁在函数结束时释放
//
// 3. 死锁风险：
//    - 当前实现不会死锁（每个命令只获取一次锁）
//    - 如果需要同时访问多个资源，需要注意锁的顺序
//
// ============================================================================

// ============================================================================
// 性能考虑
// ============================================================================
//
// 1. 文件读取：
//    - CSV: Polars 使用多线程，速度很快
//    - Excel: Calamine 是纯 Rust 实现，也很快
//    - 大文件（>100MB）可能需要几秒钟
//
// 2. 内存占用：
//    - DataFrame 完全加载到内存中
//    - 10MB CSV ≈ 20-30MB 内存
//    - 100MB CSV ≈ 200-300MB 内存
//
// 3. 并发导入：
//    - 多个文件可以并发导入（Tauri 是多线程的）
//    - 但同一时间只有一个命令可以修改 DataStore（Mutex）
//
// ============================================================================

// ============================================================================
// 前端集成示例
// ============================================================================
//
// TypeScript 封装：
//
// ```typescript
// // utils/tauri-commands.ts
// import { invoke } from '@tauri-apps/api/core';
// import type { DatasetInfo } from '@/types/dataset';
//
// export async function importCSV(filePath: string): Promise<DatasetInfo> {
//   return await invoke<DatasetInfo>('import_csv', { filePath });
// }
//
// export async function importExcel(
//   filePath: string,
//   sheetName?: string
// ): Promise<DatasetInfo> {
//   return await invoke<DatasetInfo>('import_excel', {
//     filePath,
//     sheetName
//   });
// }
// ```
//
// Vue 组件使用：
//
// ```vue
// <script setup lang="ts">
// import { importCSV } from '@/utils/tauri-commands';
// import { ElMessage } from 'element-plus';
//
// async function handleImport(filePath: string) {
//   try {
//     const info = await importCSV(filePath);
//     ElMessage.success(`导入成功：${info.name}`);
//     console.log(`行数：${info.rows}，列数：${info.columns.length}`);
//   } catch (error) {
//     ElMessage.error(`导入失败：${error}`);
//   }
// }
// </script>
// ```
//
// ============================================================================