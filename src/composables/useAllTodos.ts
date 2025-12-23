import { Window, PhysicalSize } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig } from '../types/config';

const MIN_WIDTH = 600;
const MIN_HEIGHT = 400;

/**
 * 打开全部待办窗口
 */
export async function openAllTodos(): Promise<void> {
  try {
    console.log('开始打开全部待办窗口');

    // 获取预配置的全部待办窗口
    const allTodosWindow = await Window.getByLabel('all-todos');

    if (!allTodosWindow) {
      console.error('找不到全部待办窗口，请检查 tauri.conf.json 配置');
      throw new Error('全部待办窗口未配置');
    }

    console.log('找到全部待办窗口');

    // 加载配置并应用保存的窗口尺寸
    try {
      const config = await invoke<AppConfig>('get_config');
      if (config.allTodosWindow) {
        const width = Math.max(config.allTodosWindow.width, MIN_WIDTH);
        const height = Math.max(config.allTodosWindow.height, MIN_HEIGHT);
        console.log('应用保存的全部待办窗口尺寸:', { width, height });
        await allTodosWindow.setSize(new PhysicalSize(width, height));
      }
    } catch (error) {
      console.error('加载全部待办窗口配置失败:', error);
    }

    // 显示窗口
    console.log('显示窗口');
    await allTodosWindow.show();
  } catch (error) {
    console.error('打开全部待办窗口失败:', error);
    throw error;
  }
}
