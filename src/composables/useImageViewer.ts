import { Window, PhysicalSize } from '@tauri-apps/api/window';
import { emit } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import type { ImageData } from '../types/imageViewer';
import type { AppConfig } from '../types/config';

const MIN_WIDTH = 600;
const MIN_HEIGHT = 400;

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

    // 加载配置并应用保存的窗口尺寸
    try {
      const config = await invoke<AppConfig>('get_config');
      if (config.imageViewerWindow) {
        const width = Math.max(config.imageViewerWindow.width, MIN_WIDTH);
        const height = Math.max(config.imageViewerWindow.height, MIN_HEIGHT);
        console.log('应用保存的图片查看器尺寸:', { width, height });
        await imageViewerWindow.setSize(new PhysicalSize(width, height));
      }
    } catch (error) {
      console.error('加载图片查看器配置失败:', error);
    }

    // 显示窗口
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

