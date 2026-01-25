// ============================================================================
// data/store.rs - 内存数据存储（历史栈版本）
// ============================================================================
// 这个文件实现了基于操作历史的数据存储管理
// 使用历史栈来存储每次操作后的完整状态，支持 undo/redo 功能

use crate::error::DataAnalystError;
use crate::models::{DatasetInfo, HistoryEntry, HistoryEntryInfo};
use polars::prelude::*;
use std::sync::{Arc, Mutex};

// ============================================================================
// 数据存储结构体
// ============================================================================
/// DataStore - 基于历史栈的数据存储
///
/// 新架构：
/// - 不再存储多个数据集
/// - 使用历史栈存储每次操作后的完整状态
/// - 支持 undo/redo 功能
/// - 线性历史：回退后新操作会丢弃后续历史
pub struct DataStore {
    /// 操作历史栈
    ///
    /// Vec<HistoryEntry> 存储所有历史状态
    /// 每个 HistoryEntry 包含：
    /// - 操作类型
    /// - 操作后的完整 DataFrame
    /// - 元信息
    /// - 时间戳
    history: Vec<HistoryEntry>,

    /// 当前历史索引
    ///
    /// Option<usize> 表示当前位置：
    /// - None: 没有数据（历史为空）
    /// - Some(index): 当前在历史栈中的位置
    ///
    /// 例如：
    /// - history.len() = 3, current_index = Some(2): 在最新状态
    /// - history.len() = 3, current_index = Some(1): 已经 undo 一次
    /// - history.len() = 3, current_index = Some(0): 在最早状态
    current_index: Option<usize>,

    /// 历史栈最大深度
    ///
    /// 限制历史记录数量，防止内存溢出
    /// 默认值：50
    max_history: usize,
}

// ============================================================================
// DataStore 方法实现
// ============================================================================
impl DataStore {
    /// 创建新的数据存储
    ///
    /// 默认最大历史深度为 50
    pub fn new() -> Self {
        Self {
            history: Vec::new(),
            current_index: None,
            max_history: 50,
        }
    }

    /// 创建指定最大历史深度的数据存储
    pub fn with_max_history(max_history: usize) -> Self {
        Self {
            history: Vec::new(),
            current_index: None,
            max_history,
        }
    }

    /// 添加新操作到历史栈
    ///
    /// 这是核心方法，实现线性历史管理：
    /// 1. 如果当前不在最新状态（已经 undo），丢弃后续历史
    /// 2. 添加新的历史条目
    /// 3. 如果超过最大深度，移除最早的历史
    /// 4. 更新 current_index 指向新状态
    ///
    /// 参数：
    /// - entry: 新的历史条目
    pub fn push_operation(&mut self, entry: HistoryEntry) {
        // 线性历史：如果当前不在最新状态，丢弃后续历史
        if let Some(index) = self.current_index {
            // 保留 0..=index 的历史，丢弃 index+1.. 的历史
            self.history.truncate(index + 1);
        } else {
            // 如果 current_index 为 None，清空历史
            self.history.clear();
        }

        // 添加新的历史条目
        self.history.push(entry);

        // 如果超过最大深度，移除最早的历史
        if self.history.len() > self.max_history {
            self.history.remove(0);
            // 注意：移除第一个元素后，索引需要调整
            self.current_index = Some(self.history.len() - 1);
        } else {
            // 更新 current_index 指向新状态
            self.current_index = Some(self.history.len() - 1);
        }
    }

    /// 获取当前 DataFrame
    ///
    /// 返回当前状态的 DataFrame 引用
    ///
    /// 返回：
    /// - Some(&DataFrame): 当前 DataFrame
    /// - None: 没有数据
    pub fn get_current(&self) -> Option<&DataFrame> {
        self.current_index
            .and_then(|index| self.history.get(index))
            .map(|entry| &entry.dataframe)
    }

    /// 获取当前数据集元信息
    ///
    /// 返回：
    /// - Some(&DatasetInfo): 当前元信息
    /// - None: 没有数据
    pub fn get_current_info(&self) -> Option<&DatasetInfo> {
        self.current_index
            .and_then(|index| self.history.get(index))
            .map(|entry| &entry.metadata)
    }

    /// 获取当前历史条目
    ///
    /// 返回：
    /// - Some(&HistoryEntry): 当前历史条目
    /// - None: 没有数据
    pub fn get_current_entry(&self) -> Option<&HistoryEntry> {
        self.current_index.and_then(|index| self.history.get(index))
    }

    /// 撤销操作（Undo）
    ///
    /// 回退到上一个状态
    ///
    /// 返回：
    /// - Ok(()): 成功撤销
    /// - Err: 无法撤销（已经在最早状态或没有数据）
    pub fn undo(&mut self) -> Result<(), DataAnalystError> {
        match self.current_index {
            Some(index) if index > 0 => {
                // 回退到上一个状态
                self.current_index = Some(index - 1);
                Ok(())
            }
            Some(_) => {
                // 已经在最早状态
                Err(DataAnalystError::InvalidOperation(
                    "已经在最早状态，无法撤销".to_string(),
                ))
            }
            None => {
                // 没有数据
                Err(DataAnalystError::InvalidOperation("没有数据".to_string()))
            }
        }
    }

    /// 重做操作（Redo）
    ///
    /// 前进到下一个状态
    ///
    /// 返回：
    /// - Ok(()): 成功重做
    /// - Err: 无法重做（已经在最新状态或没有数据）
    pub fn redo(&mut self) -> Result<(), DataAnalystError> {
        match self.current_index {
            Some(index) if index < self.history.len() - 1 => {
                // 前进到下一个状态
                self.current_index = Some(index + 1);
                Ok(())
            }
            Some(_) => {
                // 已经在最新状态
                Err(DataAnalystError::InvalidOperation(
                    "已经在最新状态，无法重做".to_string(),
                ))
            }
            None => {
                // 没有数据
                Err(DataAnalystError::InvalidOperation("没有数据".to_string()))
            }
        }
    }

    /// 跳转到指定历史节点
    ///
    /// 参数：
    /// - entry_id: 历史条目的 ID
    ///
    /// 返回：
    /// - Ok(()): 成功跳转
    /// - Err: 找不到指定的历史节点
    pub fn jump_to(&mut self, entry_id: &str) -> Result<(), DataAnalystError> {
        // 查找指定 ID 的历史条目
        let index = self
            .history
            .iter()
            .position(|entry| entry.id == entry_id)
            .ok_or_else(|| DataAnalystError::InvalidOperation(format!("找不到历史节点: {}", entry_id)))?;

        // 更新 current_index
        self.current_index = Some(index);
        Ok(())
    }

    /// 获取操作历史列表
    ///
    /// 返回所有历史条目的元信息（不包含 DataFrame）
    ///
    /// 返回：
    /// - Vec<HistoryEntryInfo>: 历史条目信息列表
    pub fn get_history(&self) -> Vec<HistoryEntryInfo> {
        self.history.iter().map(|entry| entry.into()).collect()
    }

    /// 获取当前历史索引
    ///
    /// 返回：
    /// - Option<usize>: 当前索引
    pub fn get_current_index(&self) -> Option<usize> {
        self.current_index
    }

    /// 获取历史栈长度
    ///
    /// 返回：
    /// - usize: 历史条目数量
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// 检查是否可以撤销
    ///
    /// 返回：
    /// - bool: 是否可以撤销
    pub fn can_undo(&self) -> bool {
        matches!(self.current_index, Some(index) if index > 0)
    }

    /// 检查是否可以重做
    ///
    /// 返回：
    /// - bool: 是否可以重做
    pub fn can_redo(&self) -> bool {
        matches!(self.current_index, Some(index) if index < self.history.len() - 1)
    }

    /// 清空所有数据和历史
    ///
    /// 删除所有历史条目，重置状态
    pub fn clear(&mut self) {
        self.history.clear();
        self.current_index = None;
    }

    /// 重置到初始状态
    ///
    /// 截断历史栈，只保留第一个节点（刚导入时的状态）
    /// 这会删除所有后续操作历史
    pub fn reset_to_initial(&mut self) -> Result<(), DataAnalystError> {
        if self.history.is_empty() {
            return Err(DataAnalystError::InvalidOperation("没有历史记录".to_string()));
        }

        // 只保留第一个历史节点
        self.history.truncate(1);
        self.current_index = Some(0);
        Ok(())
    }

    /// 清理历史（保留最近的 N 条）
    ///
    /// 参数：
    /// - keep_count: 要保留的历史条目数量
    pub fn trim_history(&mut self, keep_count: usize) {
        if self.history.len() > keep_count {
            let remove_count = self.history.len() - keep_count;
            self.history.drain(0..remove_count);

            // 调整 current_index
            if let Some(index) = self.current_index {
                if index >= remove_count {
                    self.current_index = Some(index - remove_count);
                } else {
                    self.current_index = Some(0);
                }
            }
        }
    }
}

// ============================================================================
// Default trait 实现
// ============================================================================
impl Default for DataStore {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// 类型别名
// ============================================================================
/// SharedDataStore - 共享的数据存储类型
///
/// Arc<Mutex<DataStore>> 的含义：
/// - DataStore: 我们的数据存储结构体
/// - Mutex<DataStore>: 互斥锁包装，确保同一时间只有一个线程可以修改
/// - Arc<Mutex<DataStore>>: 原子引用计数，允许多个所有者共享数据
pub type SharedDataStore = Arc<Mutex<DataStore>>;

// ============================================================================
// 使用说明
// ============================================================================
//
// 1. 创建数据存储：
//    ```rust
//    let store = DataStore::new();
//    ```
//
// 2. 添加操作：
//    ```rust
//    let entry = HistoryEntry {
//        id: uuid::Uuid::new_v4().to_string(),
//        operation: OperationType::Import { file_path: "...".to_string() },
//        dataframe: df,
//        metadata: info,
//        timestamp: chrono::Utc::now().to_rfc3339(),
//        description: "导入文件".to_string(),
//    };
//    store.push_operation(entry);
//    ```
//
// 3. 获取当前数据：
//    ```rust
//    if let Some(df) = store.get_current() {
//        // 使用 DataFrame
//    }
//    ```
//
// 4. 撤销/重做：
//    ```rust
//    store.undo()?;  // 撤销
//    store.redo()?;  // 重做
//    ```
//
// 5. 获取历史列表：
//    ```rust
//    let history = store.get_history();
//    ```
//
// ============================================================================

// ============================================================================
// 内存管理说明
// ============================================================================
//
// 1. 内存占用：
//    - 每个历史条目存储完整的 DataFrame
//    - 内存占用 = 历史条目数量 × DataFrame 大小
//    - 默认最大 50 条历史，可以根据需要调整
//
// 2. 性能考虑：
//    - Undo/Redo 是 O(1) 操作（只需要更新索引）
//    - 添加操作是 O(1) 操作（Vec::push）
//    - 跳转到指定历史是 O(n) 操作（需要查找）
//
// 3. 线性历史：
//    - 回退后新操作会丢弃后续历史
//    - 类似 Git 的 detached HEAD 行为
//    - 简单直观，易于理解
//
// 4. 未来优化：
//    - 可以考虑使用增量存储（只存储操作，不存储完整 DataFrame）
//    - 可以考虑使用压缩（减少内存占用）
//    - 可以考虑使用磁盘缓存（大数据集）
//
// ============================================================================
