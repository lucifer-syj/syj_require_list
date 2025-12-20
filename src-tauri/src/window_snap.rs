use serde::{Deserialize, Serialize};
use tauri::{PhysicalPosition, PhysicalSize, Window};

// 常量定义
const SNAP_THRESHOLD: i32 = 10; // 吸附触发距离（像素）
const UNSNAP_THRESHOLD: i32 = 50; // 取消吸附距离（像素）

// 数据结构定义

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenInfo {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub scale_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SnapEdge {
    Left,
    Right,
    Top,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapConfig {
    pub edge: SnapEdge,
    pub position: WindowPosition,
    pub size: WindowSize,
}

// Tauri 命令实现

/// 获取当前显示器信息
#[tauri::command]
pub async fn get_current_screen_info(window: Window) -> Result<ScreenInfo, String> {
    let monitor = window
        .current_monitor()
        .map_err(|e| format!("获取显示器失败: {}", e))?
        .ok_or("无法获取当前显示器")?;

    Ok(ScreenInfo {
        x: monitor.position().x,
        y: monitor.position().y,
        width: monitor.size().width,
        height: monitor.size().height,
        scale_factor: monitor.scale_factor(),
    })
}

/// 判断应该吸附到哪个边缘
#[tauri::command]
pub fn determine_snap_edge(
    position: WindowPosition,
    size: WindowSize,
    screen_info: ScreenInfo,
) -> SnapEdge {
    let distance_to_left = position.x - screen_info.x;
    let distance_to_right =
        (screen_info.x + screen_info.width as i32) - (position.x + size.width as i32);
    let distance_to_top = position.y - screen_info.y;

    // 按优先级检查：左 > 右 > 上
    if distance_to_left < SNAP_THRESHOLD && distance_to_left > -20 {
        return SnapEdge::Left;
    }
    if distance_to_right < SNAP_THRESHOLD && distance_to_right > -20 {
        return SnapEdge::Right;
    }
    if distance_to_top < SNAP_THRESHOLD && distance_to_top > -20 {
        return SnapEdge::Top;
    }

    SnapEdge::None
}

/// 计算吸附后的位置和尺寸
#[tauri::command]
pub fn calculate_snap_position(
    edge: SnapEdge,
    screen_info: ScreenInfo,
    current_size: WindowSize,
    current_position: WindowPosition,
) -> Option<SnapConfig> {
    match edge {
        SnapEdge::Left => Some(SnapConfig {
            edge: SnapEdge::Left,
            position: WindowPosition {
                x: screen_info.x,
                y: screen_info.y,
            },
            size: WindowSize {
                width: current_size.width,
                height: screen_info.height,
            },
        }),
        SnapEdge::Right => Some(SnapConfig {
            edge: SnapEdge::Right,
            position: WindowPosition {
                x: screen_info.x + screen_info.width as i32 - current_size.width as i32,
                y: screen_info.y,
            },
            size: WindowSize {
                width: current_size.width,
                height: screen_info.height,
            },
        }),
        SnapEdge::Top => Some(SnapConfig {
            edge: SnapEdge::Top,
            position: WindowPosition {
                x: current_position.x,
                y: screen_info.y,
            },
            size: current_size,
        }),
        SnapEdge::None => None,
    }
}

/// 执行吸附操作
#[tauri::command]
pub async fn snap_to_edge(
    window: Window,
    edge: SnapEdge,
    screen_info: ScreenInfo,
    window_size: WindowSize,
    window_position: WindowPosition,
) -> Result<(), String> {
    // 计算吸附位置
    let snap_config = calculate_snap_position(edge, screen_info, window_size, window_position)
        .ok_or("无法计算吸附位置")?;

    println!("=== 执行吸附操作 ===");
    println!("目标位置: {:?}", snap_config.position);
    println!("目标尺寸: {:?}", snap_config.size);

    // 先设置位置，再设置尺寸（避免尺寸改变导致位置偏移）
    window
        .set_position(PhysicalPosition::new(
            snap_config.position.x,
            snap_config.position.y,
        ))
        .map_err(|e| format!("设置窗口位置失败: {}", e))?;

    window
        .set_size(PhysicalSize::new(
            snap_config.size.width,
            snap_config.size.height,
        ))
        .map_err(|e| format!("设置窗口尺寸失败: {}", e))?;

    // 强制开启置顶
    window
        .set_always_on_top(true)
        .map_err(|e| format!("设置置顶失败: {}", e))?;

    println!("吸附操作完成");

    Ok(())
}

/// 检查是否应该取消吸附
#[tauri::command]
pub fn check_should_unsnap(
    current_position: WindowPosition,
    current_size: WindowSize,
    snap_edge: SnapEdge,
    screen_info: ScreenInfo,
) -> bool {
    let distance_from_edge = match snap_edge {
        SnapEdge::Left => current_position.x - screen_info.x,
        SnapEdge::Right => {
            (screen_info.x + screen_info.width as i32)
                - (current_position.x + current_size.width as i32)
        }
        SnapEdge::Top => current_position.y - screen_info.y,
        SnapEdge::None => return false,
    };

    distance_from_edge > UNSNAP_THRESHOLD
}
