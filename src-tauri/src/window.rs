use mouse_position::mouse_position::Mouse;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Mutex;

// 全局鼠标追踪器状态
pub struct MouseTracker {
    pub is_tracking: Arc<Mutex<bool>>,
}

impl MouseTracker {
    pub fn new() -> Self {
        Self {
            is_tracking: Arc::new(Mutex::new(false)),
        }
    }
}

/// 检查鼠标是否在触发区域（屏幕边缘）
fn check_hover_area(
    mouse_x: i32,
    mouse_y: i32,
    edge: &str,
    screen_info: &ScreenInfo,
) -> bool {
    const TRIGGER_ZONE: i32 = 10; // 触发区域10px

    match edge {
        "left" => {
            mouse_x >= screen_info.x && mouse_x <= screen_info.x + TRIGGER_ZONE
                && mouse_y >= screen_info.y
                && mouse_y <= screen_info.y + screen_info.height
        }
        "right" => {
            mouse_x >= screen_info.x + screen_info.width - TRIGGER_ZONE
                && mouse_x <= screen_info.x + screen_info.width
                && mouse_y >= screen_info.y
                && mouse_y <= screen_info.y + screen_info.height
        }
        "top" => {
            mouse_y >= screen_info.y && mouse_y <= screen_info.y + TRIGGER_ZONE
                && mouse_x >= screen_info.x
                && mouse_x <= screen_info.x + screen_info.width
        }
        _ => false,
    }
}

#[derive(Clone, serde::Serialize)]
struct ScreenInfo {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

/// 启动鼠标追踪
#[tauri::command]
pub async fn start_mouse_tracking(
    app: AppHandle,
    edge: String,
    screen_x: i32,
    screen_y: i32,
    screen_width: i32,
    screen_height: i32,
) -> Result<(), String> {
    let tracker = app.state::<MouseTracker>();
    let mut is_tracking = tracker.is_tracking.lock().await;

    // 如果已经在追踪，先停止
    if *is_tracking {
        return Ok(());
    }

    *is_tracking = true;
    drop(is_tracking);

    let is_tracking_clone = Arc::clone(&tracker.is_tracking);
    let app_clone = app.clone();
    let edge_clone = edge.clone();

    let screen_info = ScreenInfo {
        x: screen_x,
        y: screen_y,
        width: screen_width,
        height: screen_height,
    };

    tokio::spawn(async move {
        let mut was_hovering = false;

        while *is_tracking_clone.lock().await {
            let position = Mouse::get_mouse_position();
            if let Mouse::Position { x, y } = position {
                let is_hovering = check_hover_area(x, y, &edge_clone, &screen_info);

                // 只在状态变化时发送事件
                if is_hovering != was_hovering {
                    if let Some(window) = app_clone.get_webview_window("main") {
                        let _ = window.emit("mouse-hover", is_hovering);
                    }
                    was_hovering = is_hovering;
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
    });

    Ok(())
}

/// 停止鼠标追踪
#[tauri::command]
pub async fn stop_mouse_tracking(app: AppHandle) -> Result<(), String> {
    let tracker = app.state::<MouseTracker>();
    let mut is_tracking = tracker.is_tracking.lock().await;
    *is_tracking = false;
    Ok(())
}
