// 引入模块
mod database;
mod commands;

use commands::DbState;
use std::sync::Arc;
use std::path::PathBuf;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 使用 tokio 运行异步代码初始化数据库
    tauri::async_runtime::block_on(async {
        // 初始化数据库连接池
        let pool = init_database().await.expect("Failed to initialize database");

        tauri::Builder::default()
            .plugin(tauri_plugin_opener::init())
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_fs::init())
            .manage(DbState {
                pool: Arc::new(pool),
            })
            .invoke_handler(tauri::generate_handler![
                greet,
                commands::get_todos,
                commands::add_todo,
                commands::update_todo,
                commands::delete_todo,
                commands::toggle_todo,
                commands::save_image,
                commands::read_image
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}

/// 初始化数据库
async fn init_database() -> Result<sqlx::SqlitePool, sqlx::Error> {
    // 获取应用数据目录
    let app_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("syj_require_list");

    // 确保目录存在
    std::fs::create_dir_all(&app_dir).ok();

    // 数据库文件路径
    let db_path = app_dir.join("todos.db");
    let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

    println!("数据库路径: {}", db_url);

    // 创建数据库连接池
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    // 运行迁移
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS todos (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            source TEXT,
            image_path TEXT,
            ocr_text TEXT,
            created_at TEXT NOT NULL,
            expected_finish_at TEXT,
            finished_at TEXT,
            is_completed INTEGER DEFAULT 0,
            order_index INTEGER
        )",
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
