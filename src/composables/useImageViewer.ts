import { Window } from '@tauri-apps/api/window';
import { emit } from '@tauri-apps/api/event';
import type { ImageData } from '../types/imageViewer';

/**
 * 根据图片尺寸计算窗口大小
 */
function calculateWindowSize(imageWidth: number, imageHeight: number): { width: number; height: number } {
  const screenWidth = window.screen.width;
  const screenHeight = window.screen.height;
  const maxWidth = screenWidth * 0.8;
  const maxHeight = screenHeight * 0.8;

  let width = imageWidth;
  let height = imageHeight;

  if (width > maxWidth || height > maxHeight) {
    const ratio = Math.min(maxWidth / width, maxHeight / height);
    width = Math.floor(width * ratio);
    height = Math.floor(height * ratio);
  }

  // 最小尺寸限制
  width = Math.max(400, width);
  height = Math.max(300, height);

  // 添加标题栏和控制按钮的高度（约 80px）
  height = height + 80;

  return { width, height };
}

/**
 * 获取图片尺寸
 */
async function getImageDimensions(src: string): Promise<{ width: number; height: number }> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => {
      resolve({ width: img.naturalWidth, height: img.naturalHeight });
    };
    img.onerror = () => {
      reject(new Error('Failed to load image'));
    };
    img.src = src;
  });
}

/**
 * 打开图片查看器窗口
 */
export async function openImageViewer(images: ImageData[], currentIndex: number): Promise<void> {
  try {
    console.log('开始打开图片查看器');

    // 获取预配置的图片查看器窗口（Window.getByLabel 返回 Promise，需要 await）
    const imageViewerWindow = await Window.getByLabel('image-viewer');

    if (!imageViewerWindow) {
      console.error('找不到图片查看器窗口，请检查 tauri.conf.json 配置');
      throw new Error('图片查看器窗口未配置');
    }

    console.log('找到图片查看器窗口');

    // 显示窗口（尺寸调整将在 ImageViewer.vue 内部完成）
    console.log('显示窗口');
    await imageViewerWindow.show();

    // 等待一小段时间，确保窗口完全显示和 Vue 组件挂载
    await new Promise(resolve => setTimeout(resolve, 200));

    // 发送图片数据（包含窗口尺寸计算所需的信息）
    console.log('发送图片数据');
    await emit('image-viewer-data', { images, currentIndex });
    console.log('图片数据已发送');
  } catch (error) {
    console.error('打开图片查看器失败:', error);
    throw error;
  }
}

