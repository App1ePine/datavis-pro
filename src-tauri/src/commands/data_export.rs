// ============================================================================
// commands/data_export.rs - 数据导出命令（新架构）
// ============================================================================
// 这个文件定义了数据导出相关的 Tauri 命令
// 新架构：只导出当前数据集

// ============================================================================
// 导入依赖
// ============================================================================
use crate::AppState;
use polars::prelude::*;

// ============================================================================
// 导出当前数据集为 CSV 文件
// ============================================================================
/// 导出当前数据集为 CSV 文件
///
/// 参数：
/// - output_path: 输出文件的完整路径
/// - state: 应用状态（自动注入）
///
/// 返回：
/// - Result<String, String>: 成功返回文件路径，失败返回错误
#[tauri::command]
pub async fn export_csv(
    output_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    // 获取当前 DataFrame
    let store = state.data_store.lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let df = store
        .get_current()
        .ok_or("没有数据")?;

    // 创建输出文件
    let mut file = std::fs::File::create(&output_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    // 写入 CSV 数据
    CsvWriter::new(&mut file)
        .include_header(true)
        .with_separator(b',')
        .finish(&mut df.clone())
        .map_err(|e| format!("Failed to write CSV: {}", e))?;

    Ok(output_path)
}

// ============================================================================
// 导出当前数据集为 Parquet 文件
// ============================================================================
/// 导出当前数据集为 Parquet 文件
///
/// 参数：
/// - output_path: 输出文件的完整路径
/// - state: 应用状态（自动注入）
///
/// 返回：
/// - Result<String, String>: 成功返回文件路径，失败返回错误
#[tauri::command]
pub async fn export_parquet(
    output_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    // 获取当前 DataFrame
    let store = state.data_store.lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    let df = store
        .get_current()
        .ok_or("没有数据")?;

    // 创建输出文件
    let mut file = std::fs::File::create(&output_path)
        .map_err(|e| format!("Failed to create file: {}", e))?;

    // 写入 Parquet 数据
    ParquetWriter::new(&mut file)
        .finish(&mut df.clone())
        .map_err(|e| format!("Failed to write Parquet: {}", e))?;

    Ok(output_path)
}

// ============================================================================
// 清空所有数据
// ============================================================================
/// 清空所有数据和历史
///
/// 这会清空内存中的所有数据和操作历史
#[tauri::command]
pub async fn clear_data(
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let mut store = state.data_store.lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    store.clear();
    Ok(())
}

// ============================================================================
// 使用说明
// ============================================================================
//
// 前端调用示例：
//
// ```typescript
// import { save } from '@tauri-apps/plugin-dialog';
// import { invoke } from '@tauri-apps/api/core';
//
// // 1. 打开保存文件对话框
// const filePath = await save({
//   filters: [{ name: 'CSV', extensions: ['csv'] }],
//   defaultPath: 'export.csv'
// });
//
// if (filePath) {
//   // 2. 调用导出命令
//   const result = await invoke('export_csv', {
//     outputPath: filePath
//   });
//   console.log(`导出成功: ${result}`);
// }
//
// // 3. 清空所有数据
// await invoke('clear_data');
// ```
//
// ============================================================================
