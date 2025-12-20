/**
 * 窗口吸附边缘类型
 */
export type SnapEdge = 'left' | 'right' | 'top' | 'none'

/**
 * 屏幕信息
 */
export interface ScreenInfo {
  x: number
  y: number
  width: number
  height: number
  scaleFactor: number
}

/**
 * 窗口尺寸
 */
export interface WindowSize {
  width: number
  height: number
}

/**
 * 窗口位置
 */
export interface WindowPosition {
  x: number
  y: number
}

/**
 * 吸附配置
 */
export interface SnapConfig {
  edge: SnapEdge
  position: WindowPosition
  size: WindowSize
}
