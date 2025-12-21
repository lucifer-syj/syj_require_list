// 引入模块
mod database;
mod commands;
mod window_snap;
mod config;
mod migration;


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
                commands::read_image,
                window_snap::get_current_screen_info,
                window_snap::determine_snap_edge,
                window_snap::calculate_snap_position,
                window_snap::snap_to_edge,
                window_snap::check_should_unsnap,
                config::get_config,
                config::save_config_only,
                migration::migrate_data
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
    });
}

/// 初始化数据库
async fn init_database() -> Result<sqlx::SqlitePool, sqlx::Error> {
    // 加载配置
    let config = config::load_config().unwrap_or_else(|_| config::AppConfig::default());

    // 获取数据库路径
    let db_path = PathBuf::from(config.database_path());

    // 确保目录存在
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).ok();
    }

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

    // 创建 todo_images 关联表
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS todo_images (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            todo_id INTEGER NOT NULL,
            image_path TEXT NOT NULL,
            order_index INTEGER DEFAULT 0,
            created_at TEXT NOT NULL,
            FOREIGN KEY (todo_id) REFERENCES todos (id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await?;

    // 创建索引优化查询性能
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_todo_images_todo_id ON todo_images(todo_id)",
    )
    .execute(&pool)
    .await?;

    // 迁移旧数据：将 todos.image_path 迁移到 todo_images 表
    sqlx::query(
        "INSERT INTO todo_images (todo_id, image_path, order_index, created_at)
         SELECT id, image_path, 0, created_at
         FROM todos
         WHERE image_path IS NOT NULL
           AND image_path != ''
           AND id NOT IN (SELECT DISTINCT todo_id FROM todo_images)",
    )
    .execute(&pool)
    .await
    .ok(); // 忽略错误，避免重复迁移时失败

    Ok(pool)
}
