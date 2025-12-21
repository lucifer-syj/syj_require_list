use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// 应用配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub root_dir: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        // 获取默认根目录
        let root_dir = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("syj_require_list")
            .to_string_lossy()
            .to_string();

        Self { root_dir }
    }
}

impl AppConfig {
    /// 获取数据库文件路径
    pub fn database_path(&self) -> String {
        PathBuf::from(&self.root_dir)
            .join("todos.db")
            .to_string_lossy()
            .to_string()
    }

    /// 获取图片目录路径
    pub fn images_path(&self) -> String {
        PathBuf::from(&self.root_dir)
            .join("images")
            .to_string_lossy()
            .to_string()
    }
}

/// 获取配置文件路径
fn get_config_file_path() -> PathBuf {
    let app_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("syj_require_list");

    app_dir.join("config.json")
}

/// 加载配置文件
pub fn load_config() -> Result<AppConfig, String> {
    let config_path = get_config_file_path();

    // 如果配置文件不存在，返回默认配置
    if !config_path.exists() {
        println!("配置文件不存在，使用默认配置");
        return Ok(AppConfig::default());
    }

    // 读取配置文件
    let content = fs::read_to_string(&config_path)
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 解析 JSON
    let config: AppConfig = serde_json::from_str(&content)
        .map_err(|e| format!("解析配置文件失败: {}", e))?;

    println!("已加载配置: {:?}", config);
    Ok(config)
}

/// 保存配置文件
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let config_path = get_config_file_path();

    // 确保目录存在
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }

    // 序列化为 JSON
    let content = serde_json::to_string_pretty(config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;

    // 写入文件
    fs::write(&config_path, content)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;

    println!("配置已保存到: {:?}", config_path);
    Ok(())
}

/// Tauri 命令：获取配置
#[tauri::command]
pub async fn get_config() -> Result<AppConfig, String> {
    load_config()
}

/// Tauri 命令：仅保存配置（不执行迁移）
#[tauri::command]
pub async fn save_config_only(config: AppConfig) -> Result<(), String> {
    save_config(&config)
}
