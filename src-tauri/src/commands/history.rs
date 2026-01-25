// ============================================================================
// commands/history.rs - 历史管理命令
// ============================================================================
// 这个文件实现了操作历史相关的 Tauri 命令
// 包括获取历史列表、undo、redo、跳转到指定历史等功能

use crate::AppState;
use crate::models::HistoryEntryInfo;
use tauri::State;

// ============================================================================
// 获取操作历史列表
// ============================================================================
/// 获取所有操作历史的元信息
///
/// 返回历史条目列表（不包含 DataFrame）
///
/// 参数：
/// - state: 应用状态
///
/// 返回：
/// - Result<Vec<HistoryEntryInfo>, String>: 历史列表或错误消息
#[tauri::command]
pub async fn get_history(state: State<'_, AppState>) -> Result<Vec<HistoryEntryInfo>, String> {
    // 获取数据存储的锁
    let store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    // 获取历史列表
    let history = store.get_history();

    Ok(history)
}

// ============================================================================
// 获取当前历史索引
// ============================================================================
/// 获取当前在历史栈中的位置
///
/// 返回：
/// - Result<Option<usize>, String>: 当前索引或 None（没有数据）
#[tauri::command]
pub async fn get_current_index(state: State<'_, AppState>) -> Result<Option<usize>, String> {
    let store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    Ok(store.get_current_index())
}

// ============================================================================
// 撤销操作
// ============================================================================
/// 撤销上一次操作（Undo）
///
/// 回退到上一个历史状态
///
/// 返回：
/// - Result<(), String>: 成功或错误消息
#[tauri::command]
pub async fn undo_operation(state: State<'_, AppState>) -> Result<(), String> {
    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    store.undo().map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// 重做操作
// ============================================================================
/// 重做下一次操作（Redo）
///
/// 前进到下一个历史状态
///
/// 返回：
/// - Result<(), String>: 成功或错误消息
#[tauri::command]
pub async fn redo_operation(state: State<'_, AppState>) -> Result<(), String> {
    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    store.redo().map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// 跳转到指定历史
// ============================================================================
/// 跳转到指定的历史节点
///
/// 参数：
/// - entry_id: 历史条目的 ID
///
/// 返回：
/// - Result<(), String>: 成功或错误消息
#[tauri::command]
pub async fn jump_to_history(entry_id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    store.jump_to(&entry_id).map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// 检查是否可以撤销/重做
// ============================================================================
/// 检查是否可以撤销
///
/// 返回：
/// - Result<bool, String>: 是否可以撤销
#[tauri::command]
pub async fn can_undo(state: State<'_, AppState>) -> Result<bool, String> {
    let store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    Ok(store.can_undo())
}

/// 检查是否可以重做
///
/// 返回：
/// - Result<bool, String>: 是否可以重做
#[tauri::command]
pub async fn can_redo(state: State<'_, AppState>) -> Result<bool, String> {
    let store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    Ok(store.can_redo())
}

/// 重置到初始状态
///
/// 截断历史栈，只保留第一个节点（刚导入时的状态）
/// 这会删除所有后续操作历史
///
/// 返回：
/// - Result<(), String>: 成功或错误消息
#[tauri::command]
pub async fn reset_to_initial(state: State<'_, AppState>) -> Result<(), String> {
    let mut store = state
        .data_store
        .lock()
        .map_err(|e| format!("Failed to lock data store: {}", e))?;

    store.reset_to_initial().map_err(|e| e.to_string())?;

    Ok(())
}

// ============================================================================
// 使用说明
// ============================================================================
//
// 前端调用示例：
//
// ```typescript
// import { invoke } from '@tauri-apps/api/core';
//
// // 1. 获取历史列表
// const history = await invoke<HistoryEntryInfo[]>('get_history');
//
// // 2. 获取当前索引
// const index = await invoke<number | null>('get_current_index');
//
// // 3. 撤销操作
// await invoke('undo_operation');
//
// // 4. 重做操作
// await invoke('redo_operation');
//
// // 5. 跳转到指定历史
// await invoke('jump_to_history', { index: 2 });
//
// // 6. 检查是否可以撤销/重做
// const canUndo = await invoke<boolean>('can_undo');
// const canRedo = await invoke<boolean>('can_redo');
// ```
//
// ============================================================================
