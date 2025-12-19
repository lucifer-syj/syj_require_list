use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;
use tauri::State;

/// Todo 数据结构
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: Option<i64>,
    pub content: String,
    pub source: Option<String>,
    pub image_path: Option<String>,
    pub ocr_text: Option<String>,
    pub created_at: String,
    pub expected_finish_at: Option<String>,
    pub finished_at: Option<String>,
    pub is_completed: bool,
    pub order_index: Option<i64>,
}

/// 添加待办事项的输入参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTodoInput {
    pub content: String,
    pub source: Option<String>,
    pub image_path: Option<String>,
    pub ocr_text: Option<String>,
    pub expected_finish_at: Option<String>,
}

/// 更新待办事项的输入参数
#[derive(Debug, Deserialize)]
pub struct UpdateTodoInput {
    pub id: i64,
    pub content: Option<String>,
    pub expected_finish_at: Option<String>,
}

/// 数据库连接池包装器
pub struct DbState {
    pub pool: Arc<Pool<Sqlite>>,
}

/// 获取所有待办事项
#[tauri::command]
pub async fn get_todos(db: State<'_, DbState>) -> Result<Vec<Todo>, String> {
    // 查询所有待办,按完成状态和 ID 排序(未完成在前,完成在后)
    let query = "SELECT id, content, source, image_path, ocr_text, created_at,
                 expected_finish_at, finished_at, is_completed, order_index
                 FROM todos
                 ORDER BY is_completed ASC, id DESC";

    let result: Vec<Todo> = sqlx::query_as(query)
        .fetch_all(db.pool.as_ref())
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

    Ok(result)
}

/// 添加待办事项
#[tauri::command]
pub async fn add_todo(db: State<'_, DbState>, input: AddTodoInput) -> Result<Todo, String> {
    // 获取当前时间
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // 插入新待办
    let query = "INSERT INTO todos (content, source, image_path, ocr_text, created_at, expected_finish_at, is_completed, order_index)
                 VALUES (?, ?, ?, ?, ?, ?, 0, 0)";

    let result = sqlx::query(query)
        .bind(&input.content)
        .bind(input.source.as_deref().unwrap_or(""))
        .bind(input.image_path.as_deref().unwrap_or(""))
        .bind(input.ocr_text.as_deref().unwrap_or(""))
        .bind(&now)
        .bind(input.expected_finish_at.as_deref().unwrap_or(""))
        .execute(db.pool.as_ref())
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

    let id = result.last_insert_rowid();

    // 返回新创建的 Todo
    Ok(Todo {
        id: Some(id),
        content: input.content,
        source: input.source,
        image_path: input.image_path,
        ocr_text: input.ocr_text,
        created_at: now,
        expected_finish_at: input.expected_finish_at,
        finished_at: None,
        is_completed: false,
        order_index: Some(0),
    })
}

/// 更新待办事项
#[tauri::command]
pub async fn update_todo(db: State<'_, DbState>, input: UpdateTodoInput) -> Result<(), String> {
    // 构建更新语句
    let mut query_builder = sqlx::QueryBuilder::new("UPDATE todos SET ");

    let mut has_updates = false;

    if let Some(content) = &input.content {
        if has_updates {
            query_builder.push(", ");
        }
        query_builder.push("content = ");
        query_builder.push_bind(content);
        has_updates = true;
    }

    if let Some(expected_finish_at) = &input.expected_finish_at {
        if has_updates {
            query_builder.push(", ");
        }
        query_builder.push("expected_finish_at = ");
        query_builder.push_bind(expected_finish_at);
        has_updates = true;
    }

    if !has_updates {
        return Ok(());
    }

    query_builder.push(" WHERE id = ");
    query_builder.push_bind(input.id);

    query_builder
        .build()
        .execute(db.pool.as_ref())
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

    Ok(())
}

/// 删除待办事项
#[tauri::command]
pub async fn delete_todo(db: State<'_, DbState>, id: i64) -> Result<(), String> {
    // 先查询待办的图片路径
    let query = "SELECT image_path FROM todos WHERE id = ?";
    let result: Option<(Option<String>,)> = sqlx::query_as(query)
        .bind(id)
        .fetch_optional(db.pool.as_ref())
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

    // 如果有图片路径，则删除图片文件
    if let Some((Some(image_path),)) = result {
        if !image_path.is_empty() {
            // 尝试删除图片文件（忽略错误，即使删除失败也继续删除待办）
            let _ = fs::remove_file(&image_path);
            println!("已删除图片文件: {}", image_path);
        }
    }

    // 删除数据库记录
    sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(db.pool.as_ref())
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

    Ok(())
}

/// 切换待办事项完成状态
#[tauri::command]
pub async fn toggle_todo(db: State<'_, DbState>, id: i64) -> Result<(), String> {
    // 获取当前时间
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // 切换完成状态,如果变为完成则设置完成时间,否则清空完成时间
    let query = "UPDATE todos
                 SET is_completed = CASE WHEN is_completed = 0 THEN 1 ELSE 0 END,
                     finished_at = CASE WHEN is_completed = 0 THEN ? ELSE NULL END
                 WHERE id = ?";

    sqlx::query(query)
        .bind(&now)
        .bind(id)
        .execute(db.pool.as_ref())
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

    Ok(())
}

/// 保存图片文件到本地目录
#[tauri::command]
pub async fn save_image(file_data: Vec<u8>, file_name: String) -> Result<String, String> {
    // 获取应用数据目录
    let app_data_dir = dirs::data_local_dir()
        .ok_or("Failed to get local data directory")?;

    // 创建图片存储目录: AppData/Local/syj_require_list/images/
    let images_dir = app_data_dir.join("syj_require_list").join("images");

    fs::create_dir_all(&images_dir)
        .map_err(|e| format!("Failed to create images directory: {}", e))?;

    // 生成唯一文件名 (使用时间戳 + 随机数)
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();

    // 获取文件扩展名
    let path = PathBuf::from(&file_name);
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("jpg");

    // 创建一个简单的随机数作为文件名的一部分
    let random_num: u32 = (timestamp.len() * file_name.len()) as u32;
    let unique_name = format!("{}_{}.{}", timestamp, random_num, ext);

    let file_path = images_dir.join(&unique_name);

    // 保存文件
    fs::write(&file_path, file_data)
        .map_err(|e| format!("Failed to save image file: {}", e))?;

    // 返回文件路径
    Ok(file_path.to_string_lossy().to_string())
}

/// 读取图片文件为 base64
#[tauri::command]
pub async fn read_image(file_path: String) -> Result<Vec<u8>, String> {
    // 读取文件
    let data = fs::read(&file_path)
        .map_err(|e| format!("Failed to read image file: {}", e))?;

    Ok(data)
}
