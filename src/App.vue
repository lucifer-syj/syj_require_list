<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from 'vue'
import { getCurrentWindow, PhysicalPosition, PhysicalSize, Window } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import TodoList from './components/TodoList.vue'
import SettingsDialog from './components/SettingsDialog.vue'
import { useWindowStore } from './stores/windowStore'
import { useTodoStore } from './stores/todoStore'
import {
  getCurrentScreenInfo,
  determineSnapEdge,
  snapToEdge,
  restoreOriginalState,
  calculateSnapPosition,
  hideWindow,
  showWindow
} from './composables/useWindowSnap'
import type { SnapEdge } from './types/window'

// 获取当前窗口实例
const appWindow = getCurrentWindow()
const windowStore = useWindowStore()
const todoStore = useTodoStore()

// 事件监听器清理函数
let unlistenDeleted: (() => void) | null = null
let unlistenRestored: (() => void) | null = null
let unlistenPermanentDeleted: (() => void) | null = null

// 获取预览窗口实例
let previewWindow: Window | null = null
const getPreviewWindow = async () => {
  if (!previewWindow) {
    const { Window } = await import('@tauri-apps/api/window')
    previewWindow = Window.getByLabel('preview')
  }
  return previewWindow
}

// 预览框状态
let isPreviewVisible = false
let lastDetectedEdge: SnapEdge = 'none'
let lastPreviewUpdate = 0
const PREVIEW_UPDATE_INTERVAL = 100 // 100ms 更新一次预览框

// 置顶状态（初始值与配置一致）
const isAlwaysOnTop = ref(true)

// 设置对话框显示状态
const showSettings = ref(false)

// 拖动状态
let isDragging = false
let dragStartPos = { x: 0, y: 0 }
let windowStartPos = { x: 0, y: 0 }
let wasSnappedBeforeDrag = false
let previousAlwaysOnTopBeforeDrag = true

// 位置更新节流
let isUpdatingPosition = false
let pendingPosition: { x: number; y: number } | null = null

// 防抖标志
let isSnapping = false
let lastSnapTime = 0
const SNAP_COOLDOWN = 500 // 吸附后500ms内不允许拖动

// 自动隐藏定时器
let autoHideTimer: number | null = null
const AUTO_HIDE_DELAY = 1000 // 吸附后1秒自动隐藏

/**
 * 启动自动隐藏定时器
 */
const startAutoHideTimer = () => {
  // 清除已有的定时器
  cancelAutoHideTimer()

  console.log('启动自动隐藏定时器，1秒后隐藏窗口')
  autoHideTimer = window.setTimeout(async () => {
    console.log('自动隐藏定时器触发')
    await hideWindow()
  }, AUTO_HIDE_DELAY)
}

/**
 * 取消自动隐藏定时器
 */
const cancelAutoHideTimer = () => {
  if (autoHideTimer !== null) {
    console.log('取消自动隐藏定时器')
    window.clearTimeout(autoHideTimer)
    autoHideTimer = null
  }
}

// 开始拖拽窗口
const startDrag = async (e: MouseEvent) => {
  console.log('>>> startDrag 开始')
  if (e.button !== 0) return // 只响应左键

  // 取消自动隐藏定时器
  cancelAutoHideTimer()

  // 如果窗口处于隐藏状态，先显示窗口
  if (windowStore.isHidden) {
    console.log('窗口处于隐藏状态，先显示窗口')
    await showWindow()
    // 显示后需要短暂延迟，让用户看到窗口已显示
    await new Promise(resolve => setTimeout(resolve, 100))
  }

  // 防抖：正在吸附或刚吸附完成，忽略拖动
  if (isSnapping) {
    console.log('正在吸附中，忽略拖动')
    return
  }

  const now = Date.now()
  if (now - lastSnapTime < SNAP_COOLDOWN) {
    console.log(`吸附后 ${SNAP_COOLDOWN}ms 内，忽略拖动`)
    return
  }

  try {
    // 记录拖动开始时的鼠标位置和窗口位置
    isDragging = true
    dragStartPos = { x: e.screenX, y: e.screenY }
    const pos = await appWindow.outerPosition()
    windowStartPos = { x: pos.x, y: pos.y }

    // 保存拖动前的吸附状态
    wasSnappedBeforeDrag = windowStore.isSnapped
    previousAlwaysOnTopBeforeDrag = isAlwaysOnTop.value

    console.log('开始拖动，初始位置:', windowStartPos)
    console.log('拖动前是否吸附:', wasSnappedBeforeDrag)

    // 监听全局鼠标事件
    document.addEventListener('mousemove', onMouseMove)
    document.addEventListener('mouseup', onMouseUp)
  } catch (error) {
    console.error('Failed to start dragging:', error)
  }
}

// 鼠标移动时更新窗口位置（节流优化）
const onMouseMove = (e: MouseEvent) => {
  if (!isDragging) return

  const deltaX = e.screenX - dragStartPos.x
  const deltaY = e.screenY - dragStartPos.y

  // 保存最新的目标位置
  pendingPosition = {
    x: windowStartPos.x + deltaX,
    y: windowStartPos.y + deltaY
  }

  // 更新预览框（节流，100ms）
  const now = Date.now()
  if (now - lastPreviewUpdate > PREVIEW_UPDATE_INTERVAL) {
    lastPreviewUpdate = now
    updatePreview()
  }

  // 如果正在更新位置，跳过本次调用
  if (isUpdatingPosition) return

  // 标记正在更新
  isUpdatingPosition = true

  // 使用 RAF 确保在下一帧更新
  requestAnimationFrame(async () => {
    if (pendingPosition) {
      await appWindow.setPosition(
        new PhysicalPosition(pendingPosition.x, pendingPosition.y)
      )
      pendingPosition = null
    }
    isUpdatingPosition = false
  })
}

// 鼠标松开时执行边缘检测
const onMouseUp = async (e: MouseEvent) => {
  console.log('>>> 鼠标松开')
  isDragging = false

  // 移除事件监听
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)

  // 隐藏预览框
  await hidePreview()

  // 检测边缘并决定是否吸附
  await checkAndSnap(wasSnappedBeforeDrag, previousAlwaysOnTopBeforeDrag)
}

// 显示预览框
const showPreview = async (
  edge: SnapEdge,
  screenInfo: any,
  currentSize: any,
  currentPosition: any
) => {
  try {
    const preview = await getPreviewWindow()
    if (!preview) return

    // 计算预览框的位置和尺寸
    const snapConfig = await calculateSnapPosition(
      edge,
      screenInfo,
      currentSize,
      currentPosition
    )

    if (!snapConfig) return

    // 设置预览框的位置和尺寸
    await preview.setPosition(
      new PhysicalPosition(snapConfig.position.x, snapConfig.position.y)
    )
    await preview.setSize(new PhysicalSize(snapConfig.size.width, snapConfig.size.height))

    // 显示预览框
    if (!isPreviewVisible) {
      await preview.show()
      isPreviewVisible = true
    }
  } catch (error) {
    console.error('显示预览框失败:', error)
  }
}

// 隐藏预览框
const hidePreview = async () => {
  try {
    if (!isPreviewVisible) return

    const preview = await getPreviewWindow()
    if (!preview) return

    await preview.hide()
    isPreviewVisible = false
    lastDetectedEdge = 'none'
  } catch (error) {
    console.error('隐藏预览框失败:', error)
  }
}

// 更新预览框
const updatePreview = async () => {
  try {
    if (!pendingPosition) return

    // 保存副本，避免在异步操作期间被修改
    const position = { ...pendingPosition }

    // 获取当前窗口尺寸和屏幕信息
    const currentSize = await appWindow.outerSize()
    const screenInfo = await getCurrentScreenInfo()

    // 检测边缘
    const edge = await determineSnapEdge(
      position,
      { width: currentSize.width, height: currentSize.height },
      screenInfo
    )

    // 如果边缘状态没有变化，跳过
    if (edge === lastDetectedEdge) return

    lastDetectedEdge = edge

    if (edge !== 'none') {
      // 显示预览框
      await showPreview(edge, screenInfo, currentSize, position)
    } else {
      // 隐藏预览框
      await hidePreview()
    }
  } catch (error) {
    console.error('更新预览框失败:', error)
  }
}

// 检测边缘并执行吸附或还原
const checkAndSnap = async (wasSnapped: boolean, previousAlwaysOnTop: boolean) => {
  try {
    console.log('=== 鼠标松开，检测边缘 ===')
    // 获取当前窗口位置和尺寸
    const position = await appWindow.outerPosition()
    const size = await appWindow.outerSize()
    const screenInfo = await getCurrentScreenInfo()

    console.log('窗口位置:', { x: position.x, y: position.y })
    console.log('窗口尺寸:', { width: size.width, height: size.height })
    console.log('屏幕信息:', screenInfo)
    console.log('拖动前是否吸附:', wasSnapped)

    // 判断是否靠近边缘
    const edge = await determineSnapEdge(
      { x: position.x, y: position.y },
      { width: size.width, height: size.height },
      screenInfo
    )

    console.log('检测到的边缘:', edge)

    // 如果靠近边缘，执行吸附
    if (edge !== 'none') {
      console.log('靠近边缘，执行吸附到:', edge)

      // 设置防抖标志
      isSnapping = true

      const success = await snapToEdge(edge, isAlwaysOnTop.value)

      // 记录吸附时间
      lastSnapTime = Date.now()
      isSnapping = false

      if (success) {
        console.log('✅ 吸附成功')
        // 立即隐藏窗口
        await hideWindow()
      } else {
        console.log('❌ 吸附失败')
      }
    } else {
      // 不在边缘附近
      console.log('不在边缘附近')

      // 如果之前处于吸附状态，现在需要还原尺寸（但不还原位置）
      if (wasSnapped) {
        console.log('从吸附状态拖动到非边缘位置，只还原尺寸，保持当前位置')
        await restoreOriginalState(false) // 只还原尺寸，不还原位置
        isAlwaysOnTop.value = previousAlwaysOnTop
        // 取消自动隐藏定时器
        cancelAutoHideTimer()
      } else {
        console.log('保持当前位置，不做任何操作')
      }
    }
  } catch (error) {
    console.error('检测边缘失败:', error)
    isSnapping = false
  }
}

// 切换置顶状态
const toggleAlwaysOnTop = async () => {
  // 如果处于吸附状态，不允许手动切换置顶
  if (windowStore.isSnapped) {
    return
  }

  try {
    isAlwaysOnTop.value = !isAlwaysOnTop.value
    await appWindow.setAlwaysOnTop(isAlwaysOnTop.value)
  } catch (error) {
    console.error('Failed to toggle always on top:', error)
    // 如果失败，恢复状态
    isAlwaysOnTop.value = !isAlwaysOnTop.value
  }
}

// 最小化窗口
const minimizeWindow = async () => {
  try {
    await appWindow.minimize()
  } catch (error) {
    console.error('Failed to minimize window:', error)
  }
}

// 打开设置对话框
const openSettings = () => {
  showSettings.value = true
}

// 关闭设置对话框
const closeSettings = () => {
  showSettings.value = false
}

// 关闭窗口（关闭所有窗口以退出应用）
const closeWindow = async () => {
  try {
    // 手动获取并关闭所有窗口
    const previewWindow = await Window.getByLabel('preview')
    const imageViewerWindow = await Window.getByLabel('image-viewer')

    // 先关闭其他窗口
    if (imageViewerWindow) {
      await imageViewerWindow.close()
    }
    if (previewWindow) {
      await previewWindow.close()
    }

    // 最后关闭主窗口
    await appWindow.close()
  } catch (error) {
    console.error('Failed to close windows:', error)
    alert(`关闭失败: ${error}`)
  }
}

// 打开"所有待办"窗口
const openAllTodosWindow = async () => {
  try {
    const allTodosWindow = await Window.getByLabel('all-todos')
    if (allTodosWindow) {
      await allTodosWindow.show()
    }
  } catch (error) {
    console.error('Failed to open all todos window:', error)
  }
}

/**
 * 鼠标进入窗口时的处理
 */
const onMouseEnter = async () => {
  // 只有在窗口隐藏且吸附状态下才触发显示
  if (windowStore.isHidden && windowStore.isSnapped) {
    console.log('鼠标进入隐藏窗口，显示窗口')
    await showWindow()
    // 取消自动隐藏定时器（用户正在交互）
    cancelAutoHideTimer()
  }
}

/**
 * 鼠标离开窗口时的处理
 */
const onMouseLeave = async () => {
  // 如果窗口处于吸附状态且未隐藏，立即隐藏
  if (windowStore.isSnapped && !windowStore.isHidden && !isDragging) {
    console.log('鼠标离开窗口，立即隐藏')
    await hideWindow()
  }
}

// 初始化
onMounted(async () => {
  console.log('=== 应用初始化 ===')

  // 监听数据变更事件，实现窗口间同步
  unlistenDeleted = await listen('todo-deleted', async () => {
    console.log('收到 todo-deleted 事件，重新加载待办列表')
    await todoStore.fetchTodos()
  })

  unlistenRestored = await listen('todo-restored', async () => {
    console.log('收到 todo-restored 事件，重新加载待办列表')
    await todoStore.fetchTodos()
  })

  unlistenPermanentDeleted = await listen('todo-permanent-deleted', async () => {
    console.log('收到 todo-permanent-deleted 事件，重新加载待办列表')
    await todoStore.fetchTodos()
  })
})

// 清理
onBeforeUnmount(() => {
  // 清除自动隐藏定时器
  cancelAutoHideTimer()

  // 清理事件监听器
  if (unlistenDeleted) unlistenDeleted()
  if (unlistenRestored) unlistenRestored()
  if (unlistenPermanentDeleted) unlistenPermanentDeleted()
})
</script>

<template>
  <div class="app" @mouseenter="onMouseEnter" @mouseleave="onMouseLeave">
    <!-- 可拖拽的标题栏 -->
    <div class="title-bar" @mousedown="startDrag">
      <div class="title-bar-content">
        <span class="app-title">📝 Todo List</span>
        <div class="title-bar-buttons">
          <button
            class="minimize-btn"
            @click.stop="minimizeWindow"
            @mousedown.stop
            title="最小化"
          >
            −
          </button>
          <button
            class="settings-btn"
            @click.stop="openSettings"
            @mousedown.stop
            title="设置"
          >
            ⚙️
          </button>
          <button
            class="pin-btn"
            :class="{ active: isAlwaysOnTop }"
            @click.stop="toggleAlwaysOnTop"
            @mousedown.stop
            :title="isAlwaysOnTop ? '取消固定' : '固定窗口'"
          >
            📌
          </button>
          <button class="close-btn" @click.stop="closeWindow" @mousedown.stop>×</button>
        </div>
      </div>
    </div>

    <!-- 主内容区域 -->
    <div class="main-content">
      <TodoList />
      <button class="view-all-btn" @click="openAllTodosWindow">
        查看所有
      </button>
    </div>

    <!-- 设置对话框 -->
    <SettingsDialog :show="showSettings" @close="closeSettings" />
  </div>
</template>

<style scoped>
.app {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: rgba(0, 0, 0, 0.5);
}

/* 标题栏样式 */
.title-bar {
  width: 100%;
  height: 36px;
  background: rgba(26, 26, 46, 0.5);
  display: flex;
  align-items: center;
  padding: 0 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
  cursor: move;
  user-select: none;
  backdrop-filter: blur(10px);
}

.title-bar-content {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.app-title {
  color: white;
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.5px;
}

/* 标题栏按钮容器 */
.title-bar-buttons {
  display: flex;
  gap: 6px;
  align-items: center;
}

/* 最小化按钮样式 */
.minimize-btn {
  width: 28px;
  height: 28px;
  border: none;
  background-color: rgba(255, 255, 255, 0.2);
  color: white;
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.minimize-btn:hover {
  background-color: rgba(255, 255, 255, 0.3);
  transform: scale(1.05);
}

.minimize-btn:active {
  transform: scale(0.95);
}

/* 设置按钮样式 */
.settings-btn {
  width: 28px;
  height: 28px;
  border: none;
  background-color: rgba(255, 255, 255, 0.2);
  color: white;
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.settings-btn:hover {
  background-color: rgba(100, 150, 255, 0.5);
  transform: scale(1.05);
}

.settings-btn:active {
  transform: scale(0.95);
}

/* 图钉按钮样式 */
.pin-btn {
  width: 28px;
  height: 28px;
  border: none;
  background-color: rgba(255, 255, 255, 0.2);
  color: white;
  font-size: 16px;
  line-height: 1;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  opacity: 0.6;
}

.pin-btn.active {
  background-color: rgba(255, 215, 0, 0.3);
  opacity: 1;
}

.pin-btn:hover {
  opacity: 1;
  transform: scale(1.05);
}

.pin-btn:active {
  transform: scale(0.95);
}

/* 关闭按钮样式 */
.close-btn {
  width: 28px;
  height: 28px;
  border: none;
  background-color: rgba(255, 255, 255, 0.2);
  color: white;
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
}

.close-btn:hover {
  background-color: #ff4444;
  transform: scale(1.05);
}

.close-btn:active {
  transform: scale(0.95);
}

/* 主内容区域 */
.main-content {
  flex: 1;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

/* 查看所有按钮 */
.view-all-btn {
  width: 100%;
  height: 40px;
  background-color: rgba(100, 150, 255, 0.3);
  color: white;
  border: none;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.2s;
  letter-spacing: 0.5px;
}

.view-all-btn:hover {
  background-color: rgba(100, 150, 255, 0.5);
  transform: translateY(-1px);
}

.view-all-btn:active {
  transform: translateY(0);
}
</style>

<style>
/* 全局样式重置 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
    'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
    sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  overflow: hidden;
}

#app {
  width: 100%;
  height: 100vh;
}
</style>
