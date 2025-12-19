use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::sync::Arc;
use std::fs;
use std::path::PathBuf;
use tauri::State;

/// 图片数据结构
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TodoImage {
    pub id: Option<i64>,
    pub todo_id: i64,
    pub image_path: String,
    pub order_index: i64,
    pub created_at: String,
}

/// Todo 数据结构
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Todo {
    pub id: Option<i64>,
    pub content: String,
    pub source: Option<String>,
    pub image_path: Option<String>,  // 保留用于向后兼容
    pub ocr_text: Option<String>,
    pub created_at: String,
    pub expected_finish_at: Option<String>,
    pub finished_at: Option<String>,
    pub is_completed: bool,
    pub order_index: Option<i64>,

    // 新增：关联的图片列表（查询时动态加载）
    #[sqlx(skip)]
    pub images: Option<Vec<TodoImage>>,
}

/// 添加待办事项的输入参数
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTodoInput {
    pub content: String,
    pub source: Option<String>,
    pub image_paths: Option<Vec<String>>,  // 改为数组
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

    let mut todos: Vec<Todo> = sqlx::query_as(query)
        .fetch_all(db.pool.as_ref())
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

    // 为每个待办加载关联的图片
    for todo in &mut todos {
        if let Some(todo_id) = todo.id {
            let images = get_todo_images(&db, todo_id).await?;

            // 向后兼容：如果 todo_images 表为空但 image_path 有值，则使用旧数据
            if images.is_empty() && todo.image_path.is_some() {
                if let Some(path) = &todo.image_path {
                    if !path.is_empty() {
                        todo.images = Some(vec![TodoImage {
                            id: None,
                            todo_id,
                            image_path: path.clone(),
                            order_index: 0,
                            created_at: todo.created_at.clone(),
                        }]);
                    }
                }
            } else {
                todo.images = Some(images);
            }
        }
    }

    Ok(todos)
}

/// 添加待办事项
#[tauri::command]
pub async fn add_todo(db: State<'_, DbState>, input: AddTodoInput) -> Result<Todo, String> {
    // 获取当前时间
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // 开始事务
    let mut tx = db.pool.begin()
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

    // 插入待办（image_path 字段留空，使用新的 todo_images 表）
    let query = "INSERT INTO todos (content, source, image_path, ocr_text, created_at, expected_finish_at, is_completed, order_index)
                 VALUES (?, ?, '', ?, ?, ?, 0, 0)";

    let result = sqlx::query(query)
        .bind(&input.content)
        .bind(input.source.as_deref().unwrap_or(""))
        .bind(input.ocr_text.as_deref().unwrap_or(""))
        .bind(&now)
        .bind(input.expected_finish_at.as_deref().unwrap_or(""))
        .execute(&mut *tx)
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

    let todo_id = result.last_insert_rowid();

    // 批量插入图片记录
    let mut images = Vec::new();
    if let Some(image_paths) = &input.image_paths {
        for (index, path) in image_paths.iter().enumerate() {
            let img_query = "INSERT INTO todo_images (todo_id, image_path, order_index, created_at)
                            VALUES (?, ?, ?, ?)";

            sqlx::query(img_query)
                .bind(todo_id)
                .bind(path)
                .bind(index as i64)
                .bind(&now)
                .execute(&mut *tx)
                .await
                .map_err(|e: sqlx::Error| e.to_string())?;

            images.push(TodoImage {
                id: None,
                todo_id,
                image_path: path.clone(),
                order_index: index as i64,
                created_at: now.clone(),
            });
        }
    }

    // 提交事务
    tx.commit()
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

    // 返回新创建的 Todo
    Ok(Todo {
        id: Some(todo_id),
        content: input.content,
        source: input.source,
        image_path: None,  // 不再使用旧的 image_path 字段
        ocr_text: input.ocr_text,
        created_at: now,
        expected_finish_at: input.expected_finish_at,
        finished_at: None,
        is_completed: false,
        order_index: Some(0),
        images: Some(images),
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
    // 1. 查询所有关联的图片路径
    let image_query = "SELECT image_path FROM todo_images WHERE todo_id = ?";
    let image_paths: Vec<(String,)> = sqlx::query_as(image_query)
        .bind(id)
        .fetch_all(db.pool.as_ref())
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

    // 2. 向后兼容：检查旧的 image_path 字段
    let legacy_query = "SELECT image_path FROM todos WHERE id = ?";
    let legacy_result: Option<(Option<String>,)> = sqlx::query_as(legacy_query)
        .bind(id)
        .fetch_optional(db.pool.as_ref())
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

    // 3. 删除所有图片文件
    for (path,) in image_paths {
        if !path.is_empty() {
            let _ = fs::remove_file(&path);
            println!("已删除图片文件: {}", path);
        }
    }

    // 删除旧字段的图片
    if let Some((Some(legacy_path),)) = legacy_result {
        if !legacy_path.is_empty() {
            let _ = fs::remove_file(&legacy_path);
            println!("已删除旧图片文件: {}", legacy_path);
        }
    }

    // 4. 删除数据库记录（外键约束会自动删除 todo_images 记录）
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

/// 获取指定待办的所有图片（辅助函数，不暴露为命令）
async fn get_todo_images(db: &State<'_, DbState>, todo_id: i64) -> Result<Vec<TodoImage>, String> {
    let query = "SELECT id, todo_id, image_path, order_index, created_at
                 FROM todo_images
                 WHERE todo_id = ?
                 ORDER BY order_index ASC";

    let images: Vec<TodoImage> = sqlx::query_as(query)
        .bind(todo_id)
        .fetch_all(db.pool.as_ref())
        .await
        .map_err(|e: sqlx::Error| e.to_string())?;

    Ok(images)
}
