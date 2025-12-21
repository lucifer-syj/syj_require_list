// 图片查看器相关类型定义

/**
 * 图片数据
 */
export interface ImageData {
  path: string;  // 图片文件路径
  src: string;   // Blob URL
}

/**
 * 图片查看器数据（通过事件传递）
 */
export interface ImageViewerData {
  images: ImageData[];  // 图片列表
  currentIndex: number; // 当前显示的图片索引
}
