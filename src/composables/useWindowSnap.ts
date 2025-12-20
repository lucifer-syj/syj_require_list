import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'
import { PhysicalPosition, PhysicalSize } from '@tauri-apps/api/window'
import { useWindowStore } from '../stores/windowStore'
import type { SnapEdge, ScreenInfo, WindowSize, WindowPosition, SnapConfig } from '../types/window'

const appWindow = getCurrentWindow()

/**
 * 获取当前显示器信息
 */
export async function getCurrentScreenInfo(): Promise<ScreenInfo> {
  return await invoke<ScreenInfo>('get_current_screen_info')
}

/**
 * 判断应该吸附到哪个边缘
 */
export async function determineSnapEdge(
  position: WindowPosition,
  size: WindowSize,
  screenInfo: ScreenInfo
): Promise<SnapEdge> {
  return await invoke<SnapEdge>('determine_snap_edge', {
    position,
    size,
    screenInfo
  })
}

/**
 * 计算吸附后的位置和尺寸
 */
export async function calculateSnapPosition(
  edge: SnapEdge,
  screenInfo: ScreenInfo,
  currentSize: WindowSize,
  currentPosition: WindowPosition
): Promise<SnapConfig | null> {
  return await invoke<SnapConfig | null>('calculate_snap_position', {
    edge,
    screenInfo,
    currentSize,
    currentPosition
  })
}

/**
 * 执行吸附操作
 */
export async function snapToEdge(
  edge: SnapEdge,
  currentAlwaysOnTop: boolean
): Promise<boolean> {
  try {
    const windowStore = useWindowStore()
    const screenInfo = await getCurrentScreenInfo()
    const currentSize = await appWindow.outerSize()
    const currentPosition = await appWindow.outerPosition()

    const windowSize: WindowSize = {
      width: currentSize.width,
      height: currentSize.height
    }

    const windowPosition: WindowPosition = {
      x: currentPosition.x,
      y: currentPosition.y
    }

    // 保存原始状态
    windowStore.saveOriginalState(windowSize, windowPosition, currentAlwaysOnTop)

    // 执行吸附
    await invoke('snap_to_edge', {
      edge,
      screenInfo,
      windowSize,
      windowPosition
    })

    // 更新状态
    windowStore.setSnap(edge)
    windowStore.setScreenInfo(screenInfo)

    return true
  } catch (error) {
    console.error('吸附失败:', error)
    return false
  }
}

/**
 * 还原窗口到原始状态
 * @param restorePosition 是否还原位置，默认为 false（只还原尺寸）
 */
export async function restoreOriginalState(restorePosition: boolean = false): Promise<boolean> {
  try {
    const windowStore = useWindowStore()

    if (!windowStore.originalSize) {
      return false
    }

    // 还原尺寸
    await appWindow.setSize(
      new PhysicalSize(windowStore.originalSize.width, windowStore.originalSize.height)
    )

    // 只在需要时还原位置
    if (restorePosition && windowStore.originalPosition) {
      await appWindow.setPosition(
        new PhysicalPosition(windowStore.originalPosition.x, windowStore.originalPosition.y)
      )
    }

    // 还原置顶状态
    await appWindow.setAlwaysOnTop(windowStore.originalAlwaysOnTop)

    // 清除吸附状态
    windowStore.clearSnap()

    return true
  } catch (error) {
    console.error('还原失败:', error)
    return false
  }
}

/**
 * 缓动函数：easeInOutCubic
 */
function easeInOutCubic(t: number): number {
  return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2
}

// 隐藏时保留的可见区域宽度（px）
const VISIBLE_AREA_WIDTH = 10

// 防止重复调用的标志
let isHiding = false
let isShowing = false

/**
 * 隐藏窗口（移动到屏幕边缘，只保留10px可见）
 */
export async function hideWindow(): Promise<boolean> {
  try {
    const windowStore = useWindowStore()

    // 检查是否正在隐藏
    if (isHiding) {
      console.log('窗口正在隐藏中，忽略重复调用')
      return false
    }

    // 检查是否已经隐藏
    if (windowStore.isHidden) {
      console.log('窗口已经隐藏，无需再次隐藏')
      return false
    }

    if (!windowStore.isSnapped || !windowStore.screenInfo) {
      console.log('窗口未吸附，无法隐藏')
      return false
    }

    // 设置标志
    isHiding = true

    // 获取当前位置
    const currentPosition = await appWindow.outerPosition()
    const currentSize = await appWindow.outerSize()

    console.log('=== 开始隐藏窗口 ===')
    console.log('当前位置:', { x: currentPosition.x, y: currentPosition.y })
    console.log('当前尺寸:', { width: currentSize.width, height: currentSize.height })

    // 保存吸附位置（用于恢复）
    windowStore.saveSnappedPosition({ x: currentPosition.x, y: currentPosition.y })

    // 计算隐藏后的位置（保留10px可见）
    let targetX = currentPosition.x
    let targetY = currentPosition.y

    const edge = windowStore.snapEdge
    const screen = windowStore.screenInfo

    console.log('吸附边缘:', edge)
    console.log('屏幕信息:', screen)

    if (edge === 'left') {
      targetX = screen.x - currentSize.width + VISIBLE_AREA_WIDTH
    } else if (edge === 'right') {
      targetX = screen.x + screen.width - VISIBLE_AREA_WIDTH
    } else if (edge === 'top') {
      targetY = screen.y - currentSize.height + VISIBLE_AREA_WIDTH
    }

    console.log('目标位置:', { x: targetX, y: targetY })
    console.log('移动距离:', {
      deltaX: targetX - currentPosition.x,
      deltaY: targetY - currentPosition.y
    })

    // 执行动画（优化：150ms，15fps）
    const duration = 150 // ms（进一步优化为 150ms）
    const fps = 15 // 目标帧率（进一步优化为 15fps）
    const frameInterval = 1000 / fps
    const startTime = Date.now()
    const startX = currentPosition.x
    const startY = currentPosition.y
    let lastFrameTime = startTime

    return new Promise((resolve) => {
      const animate = async () => {
        const now = Date.now()
        const elapsed = now - startTime
        const progress = Math.min(elapsed / duration, 1)

        // 帧率限制：只在距离上一帧足够时间后才更新
        if (now - lastFrameTime >= frameInterval || progress >= 1) {
          lastFrameTime = now

          const easedProgress = easeInOutCubic(progress)
          const newX = startX + (targetX - startX) * easedProgress
          const newY = startY + (targetY - startY) * easedProgress

          // 使用 await 等待位置更新完成，避免调用堆积
          await appWindow.setPosition(new PhysicalPosition(Math.round(newX), Math.round(newY)))
        }

        if (progress < 1) {
          requestAnimationFrame(animate)
        } else {
          // 动画完成，更新状态
          windowStore.setHidden(true)
          isHiding = false // 重置标志
          console.log('窗口已隐藏')
          resolve(true)
        }
      }

      requestAnimationFrame(animate)
    })
  } catch (error) {
    console.error('隐藏窗口失败:', error)
    isHiding = false // 重置标志
    return false
  }
}

/**
 * 显示窗口（从隐藏状态恢复到吸附位置）
 */
export async function showWindow(): Promise<boolean> {
  try {
    const windowStore = useWindowStore()

    // 检查是否正在显示
    if (isShowing) {
      console.log('窗口正在显示中，忽略重复调用')
      return false
    }

    // 检查是否已经显示
    if (!windowStore.isHidden) {
      console.log('窗口已经显示，无需再次显示')
      return false
    }

    if (!windowStore.isHidden || !windowStore.snappedPosition) {
      console.log('窗口未隐藏，无需显示')
      return false
    }

    // 设置标志
    isShowing = true

    // 获取当前位置
    const currentPosition = await appWindow.outerPosition()

    // 目标位置是保存的吸附位置
    const targetX = windowStore.snappedPosition.x
    const targetY = windowStore.snappedPosition.y

    // 执行动画（优化：150ms，15fps）
    const duration = 150 // ms（进一步优化为 150ms）
    const fps = 15 // 目标帧率（进一步优化为 15fps）
    const frameInterval = 1000 / fps
    const startTime = Date.now()
    const startX = currentPosition.x
    const startY = currentPosition.y
    let lastFrameTime = startTime

    return new Promise((resolve) => {
      const animate = async () => {
        const now = Date.now()
        const elapsed = now - startTime
        const progress = Math.min(elapsed / duration, 1)

        // 帧率限制：只在距离上一帧足够时间后才更新
        if (now - lastFrameTime >= frameInterval || progress >= 1) {
          lastFrameTime = now

          const easedProgress = easeInOutCubic(progress)
          const newX = startX + (targetX - startX) * easedProgress
          const newY = startY + (targetY - startY) * easedProgress

          // 使用 await 等待位置更新完成，避免调用堆积
          await appWindow.setPosition(new PhysicalPosition(Math.round(newX), Math.round(newY)))
        }

        if (progress < 1) {
          requestAnimationFrame(animate)
        } else {
          // 动画完成，更新状态
          windowStore.setHidden(false)
          isShowing = false // 重置标志
          console.log('窗口已显示')
          resolve(true)
        }
      }

      requestAnimationFrame(animate)
    })
  } catch (error) {
    console.error('显示窗口失败:', error)
    isShowing = false // 重置标志
    return false
  }
}
