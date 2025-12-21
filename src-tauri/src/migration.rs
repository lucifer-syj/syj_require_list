use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use crate::config::{self, AppConfig};


/// 迁移结果结构体
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationResult {
    pub success: bool,
    pub message: String,
    pub database_copied: bool,
    pub images_copied: i32,
    pub paths_updated: i32,
}

/// 验证路径有效性（可写权限）
fn validate_path(path: &str) -> Result<(), String> {
    let path_buf = PathBuf::from(path);

    // 检查父目录是否存在
    let parent = if path_buf.is_file() || path.ends_with(".db") {
        path_buf.parent()
    } else {
        Some(path_buf.as_path())
    };

    if let Some(parent_dir) = parent {
        // 如果父目录不存在，尝试创建
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir)
                .map_err(|e| format!("无法创建目录 {}: {}", parent_dir.display(), e))?;
        }

        // 测试写入权限
        let test_file = parent_dir.join(".write_test");
        fs::write(&test_file, "test")
            .map_err(|e| format!("目录 {} 没有写入权限: {}", parent_dir.display(), e))?;
        fs::remove_file(&test_file).ok();
    }

    Ok(())
}

/// Tauri 命令：执行完整的数据迁移
#[tauri::command]
pub async fn migrate_data(
    new_config: AppConfig,
) -> Result<MigrationResult, String> {
    println!("=== 开始数据迁移 ===");

    // 加载当前配置
    let old_config = config::load_config()?;

    // 检查路径是否有变化
    if old_config.root_dir == new_config.root_dir {
        return Ok(MigrationResult {
            success: true,
            message: "路径未变化，无需迁移".to_string(),
            database_copied: false,
            images_copied: 0,
            paths_updated: 0,
        });
    }

    // 验证新路径
    validate_path(&new_config.root_dir)?;

    let old_root = PathBuf::from(&old_config.root_dir);
    let new_root = PathBuf::from(&new_config.root_dir);

    // 确保新根目录存在
    fs::create_dir_all(&new_root)
        .map_err(|e| format!("创建新根目录失败: {}", e))?;

    // 统计复制的文件数量
    let mut total_files = 0;

    // 复制整个目录内容
    if old_root.exists() {
        copy_dir_recursive(&old_root, &new_root, &mut total_files)?;
    }

    // 保存新配置
    config::save_config(&new_config)?;

    println!("=== 数据迁移完成 ===");

    Ok(MigrationResult {
        success: true,
        message: format!(
            "迁移成功！已复制 {} 个文件到新目录\n\n旧目录: {}\n新目录: {}\n\n请重启应用以使用新路径。",
            total_files,
            old_config.root_dir,
            new_config.root_dir
        ),
        database_copied: true,
        images_copied: total_files,
        paths_updated: 0,
    })
}

/// 递归复制目录
fn copy_dir_recursive(src: &Path, dest: &Path, file_count: &mut i32) -> Result<(), String> {
    if !src.exists() {
        return Ok(());
    }

    // 确保目标目录存在
    fs::create_dir_all(dest)
        .map_err(|e| format!("创建目标目录失败: {}", e))?;

    // 遍历源目录
    let entries = fs::read_dir(src)
        .map_err(|e| format!("读取源目录失败: {}", e))?;

    for entry in entries {
        if let Ok(entry) = entry {
            let src_path = entry.path();
            let file_name = entry.file_name();
            let dest_path = dest.join(&file_name);

            if src_path.is_dir() {
                // 递归复制子目录
                copy_dir_recursive(&src_path, &dest_path, file_count)?;
            } else if src_path.is_file() {
                // 复制文件
                if let Err(e) = fs::copy(&src_path, &dest_path) {
                    println!("警告: 复制文件失败: {} -> {}: {}",
                        src_path.display(), dest_path.display(), e);
                } else {
                    *file_count += 1;
                }
            }
        }
    }

    Ok(())
}

