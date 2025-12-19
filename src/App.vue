<script setup lang="ts">
import { ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import TodoList from './components/TodoList.vue';

// 获取当前窗口实例
const appWindow = getCurrentWindow();

// 置顶状态（初始值与配置一致）
const isAlwaysOnTop = ref(true);

// 开始拖拽窗口
const startDrag = async () => {
  try {
    await appWindow.startDragging();
  } catch (error) {
    console.error('Failed to start dragging:', error);
  }
};

// 切换置顶状态
const toggleAlwaysOnTop = async () => {
  try {
    isAlwaysOnTop.value = !isAlwaysOnTop.value;
    await appWindow.setAlwaysOnTop(isAlwaysOnTop.value);
  } catch (error) {
    console.error('Failed to toggle always on top:', error);
    // 如果失败，恢复状态
    isAlwaysOnTop.value = !isAlwaysOnTop.value;
  }
};

// 关闭窗口
const closeWindow = async () => {
  try {
    await appWindow.close();
  } catch (error) {
    console.error('Failed to close window:', error);
    alert(`关闭失败: ${error}`);
  }
};
</script>

<template>
  <div class="app">
    <!-- 可拖拽的标题栏 -->
    <div class="title-bar" @mousedown="startDrag">
      <div class="title-bar-content">
        <span class="app-title">📝 Todo List</span>
        <div class="title-bar-buttons">
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
    </div>
  </div>
</template>

<style scoped>
.app {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #f5f5f5;
}

/* 标题栏样式 */
.title-bar {
  width: 100%;
  height: 36px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  display: flex;
  align-items: center;
  padding: 0 12px;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  cursor: move;
  user-select: none;
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
