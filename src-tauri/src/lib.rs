// ============================================================================
// lib.rs - Tauri 应用核心库文件
// ============================================================================
// 这个文件是整个 Tauri 应用的核心，负责：
// 1. 组织和导入所有模块
// 2. 定义应用状态（AppState）
// 3. 注册 Tauri 命令（前端可以调用的函数）
// 4. 初始化和运行 Tauri 应用

// ============================================================================
// 导入标准库
// ============================================================================
// std::sync 是 Rust 标准库中的同步原语模块
// Arc = Atomic Reference Counted（原子引用计数）：允许多个所有者共享数据
// Mutex = Mutual Exclusion（互斥锁）：确保同一时间只有一个线程可以访问数据
use std::sync::{Arc, Mutex};

// ============================================================================
// 声明模块
// ============================================================================
// mod 关键字用于声明模块，Rust 会在同名的文件或文件夹中查找模块内容
mod error;      // 错误处理模块（error.rs）
mod models;     // 数据模型模块（models/ 文件夹）
mod data;       // 数据处理模块（data/ 文件夹）
mod commands;   // Tauri 命令模块（commands/ 文件夹）

// ============================================================================
// 导入模块中的类型和函数
// ============================================================================
// use 关键字用于将模块中的内容引入当前作用域，类似于 import

// 从 data 模块导入数据存储相关类型
use data::{DataStore, SharedDataStore};

// 从 commands 模块导入所有 Tauri 命令函数
// 这些函数可以被前端 JavaScript/TypeScript 代码调用
use commands::{
    // 文件导入
    import_csv,
    import_excel,
    import_parquet,
    // 数据查询（新架构）
    get_current_data,
    get_current_info,
    get_column_stats,
    // 历史管理
    get_history,
    get_current_index,
    undo_operation,
    redo_operation,
    jump_to_history,
    can_undo,
    can_redo,
    reset_to_initial,
    // 数据操作
    drop_nulls,
    drop_all_nulls,
    select_columns,
    drop_columns,
    rename_columns,
    cast_types,
    filter_data,
    fill_null,
    // 数据导出
    export_csv,
    export_parquet,
    clear_data,
};

// ============================================================================
// 应用状态结构体
// ============================================================================
/// AppState - 应用的全局状态
///
/// 在 Tauri 中，AppState 用于在整个应用生命周期中共享数据
/// 所有的 Tauri 命令都可以访问这个状态
///
/// pub 关键字表示这个结构体是公开的，可以被其他模块访问
pub struct AppState {
    /// data_store: 共享的数据存储
    ///
    /// SharedDataStore 是 Arc<Mutex<DataStore>> 的类型别名
    /// - Arc: 允许多个 Tauri 命令同时持有数据存储的引用
    /// - Mutex: 确保同一时间只有一个命令可以修改数据
    ///
    /// 这种模式在 Rust 中很常见，用于在多线程环境中安全地共享可变数据
    pub data_store: SharedDataStore,
}

// ============================================================================
// 应用运行函数
// ============================================================================
/// run() - 启动 Tauri 应用
///
/// 这个函数负责：
/// 1. 创建应用状态
/// 2. 配置 Tauri 插件
/// 3. 注册命令处理器
/// 4. 启动应用
///
/// #[cfg_attr(...)] 是一个条件编译属性
/// 在移动平台上，这个函数会被标记为入口点
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // ------------------------------------------------------------------------
    // 1. 创建共享数据存储
    // ------------------------------------------------------------------------
    // DataStore::new() 创建一个新的数据存储实例
    // Mutex::new() 将数据存储包装在互斥锁中
    // Arc::new() 将互斥锁包装在原子引用计数中
    //
    // 这样做的好处：
    // - 多个命令可以同时持有数据存储的引用（Arc）
    // - 但同一时间只有一个命令可以修改数据（Mutex）
    // - 当最后一个引用被释放时，数据会自动清理（Arc 的引用计数）
    let data_store = Arc::new(Mutex::new(DataStore::new()));

    // ------------------------------------------------------------------------
    // 2. 构建并配置 Tauri 应用
    // ------------------------------------------------------------------------
    tauri::Builder::default()
        // 注册插件：opener 插件用于在默认浏览器中打开 URL
        .plugin(tauri_plugin_opener::init())

        // 注册插件：dialog 插件用于显示文件选择对话框
        .plugin(tauri_plugin_dialog::init())

        // 管理应用状态：将 AppState 注册到 Tauri
        // 这样所有的命令都可以通过 State 参数访问这个状态
        .manage(AppState { data_store })

        // 注册命令处理器：告诉 Tauri 哪些函数可以被前端调用
        .invoke_handler(tauri::generate_handler![
            // 文件导入命令
            import_csv,
            import_excel,
            import_parquet,

            // 数据查询命令（新架构）
            get_current_data,
            get_current_info,
            get_column_stats,

            // 历史管理命令
            get_history,
            get_current_index,
            undo_operation,
            redo_operation,
            jump_to_history,
            can_undo,
            can_redo,
            reset_to_initial,

            // 数据操作命令
            drop_nulls,
            drop_all_nulls,
            select_columns,
            drop_columns,
            rename_columns,
            cast_types,
            filter_data,
            fill_null,

            // 数据导出命令
            export_csv,
            export_parquet,
            clear_data,
        ])

        // 运行应用
        // tauri::generate_context!() 生成应用上下文（从 tauri.conf.json 读取配置）
        .run(tauri::generate_context!())

        // 如果运行失败，程序会 panic（崩溃）并显示错误信息
        .expect("error while running tauri application");
}

// ============================================================================
// 总结：前端如何调用这些命令
// ============================================================================
//
// 在前端 TypeScript 代码中：
//
// import { invoke } from '@tauri-apps/api/core';
//
// // 导入 CSV 文件
// const datasetInfo = await invoke('import_csv', {
//     filePath: '/path/to/file.csv'
// });
//
// // 获取数据集列表
// const datasets = await invoke('get_dataset_list');
//
// // 获取分页数据
// const data = await invoke('get_dataset_data', {
//     datasetId: 'some-uuid',
//     offset: 0,
//     limit: 100
// });
//
// ============================================================================