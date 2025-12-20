import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { SnapEdge, ScreenInfo, WindowSize, WindowPosition } from '../types/window'

export const useWindowStore = defineStore('window', () => {
  // 吸附状态
  const isSnapped = ref(false)
  const snapEdge = ref<SnapEdge>('none')

  // 原始状态（吸附前保存）
  const originalSize = ref<WindowSize | null>(null)
  const originalPosition = ref<WindowPosition | null>(null)
  const originalAlwaysOnTop = ref(true)

  // 屏幕信息
  const screenInfo = ref<ScreenInfo | null>(null)

  // 隐藏状态
  const isHidden = ref(false)
  const snappedPosition = ref<WindowPosition | null>(null) // 吸附位置（用于从隐藏状态恢复）

  /**
   * 设置吸附状态
   */
  function setSnap(edge: SnapEdge) {
    isSnapped.value = edge !== 'none'
    snapEdge.value = edge
  }

  /**
   * 清除吸附状态
   */
  function clearSnap() {
    isSnapped.value = false
    snapEdge.value = 'none'
    originalSize.value = null
    originalPosition.value = null
    screenInfo.value = null
    isHidden.value = false
    snappedPosition.value = null
  }

  /**
   * 保存原始状态
   */
  function saveOriginalState(
    size: WindowSize,
    position: WindowPosition,
    alwaysOnTop: boolean
  ) {
    originalSize.value = size
    originalPosition.value = position
    originalAlwaysOnTop.value = alwaysOnTop
  }

  /**
   * 设置屏幕信息
   */
  function setScreenInfo(info: ScreenInfo) {
    screenInfo.value = info
  }

  /**
   * 设置隐藏状态
   */
  function setHidden(hidden: boolean) {
    isHidden.value = hidden
  }

  /**
   * 保存吸附位置
   */
  function saveSnappedPosition(position: WindowPosition) {
    snappedPosition.value = position
  }

  return {
    // 状态
    isSnapped,
    snapEdge,
    originalSize,
    originalPosition,
    originalAlwaysOnTop,
    screenInfo,
    isHidden,
    snappedPosition,
    // 方法
    setSnap,
    clearSnap,
    saveOriginalState,
    setScreenInfo,
    setHidden,
    saveSnappedPosition
  }
})
