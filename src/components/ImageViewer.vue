<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import type { ImageData, ImageViewerData } from '../types/imageViewer';

// 获取当前窗口实例
const appWindow = getCurrentWindow();

// 图片数据
const images = ref<ImageData[]>([]);
const currentIndex = ref(0);
const isAlwaysOnTop = ref(true);

// 缩放和平移状态
const scale = ref(1.0);
const translateX = ref(0);
const translateY = ref(0);
const imageContainer = ref<HTMLElement | null>(null);

// 拖动状态
let isDragging = false;
let startX = 0;
let startY = 0;

// 监听图片数据事件
let unlisten: (() => void) | null = null;

onMounted(async () => {
  // 监听图片数据事件
  unlisten = await listen<ImageViewerData>('image-viewer-data', async (event) => {
    console.log('收到图片数据:', event.payload);
    images.value = event.payload.images;
    currentIndex.value = event.payload.currentIndex;
    resetTransform();

    // 根据图片尺寸调整窗口大小并居中
    if (images.value.length > 0 && images.value[currentIndex.value]) {
      try {
        const img = await getImageDimensions(images.value[currentIndex.value].src);
        const windowSize = calculateWindowSize(img.width, img.height);
        console.log('调整窗口尺寸:', windowSize);

        await appWindow.setSize({ width: windowSize.width, height: windowSize.height });
        await appWindow.center();
      } catch (error) {
        console.warn('无法调整窗口尺寸:', error);
      }
    }
  });

  // 监听键盘事件
  window.addEventListener('keydown', handleKeydown);
});

onBeforeUnmount(() => {
  // 清理事件监听
  if (unlisten) {
    unlisten();
  }
  window.removeEventListener('keydown', handleKeydown);
  // 清理 Blob URL
  images.value.forEach(img => {
    if (img.src) URL.revokeObjectURL(img.src);
  });
});

// 关闭窗口（实际上是隐藏，不销毁）
const closeWindow = async () => {
  try {
    await appWindow.hide();
  } catch (error) {
    console.error('隐藏窗口失败:', error);
  }
};

// 切换置顶状态
const toggleAlwaysOnTop = async () => {
  try {
    isAlwaysOnTop.value = !isAlwaysOnTop.value;
    await appWindow.setAlwaysOnTop(isAlwaysOnTop.value);
  } catch (error) {
    console.error('切换置顶状态失败:', error);
    isAlwaysOnTop.value = !isAlwaysOnTop.value;
  }
};

// 开始拖拽窗口
const startDrag = async (e: MouseEvent) => {
  if (e.button !== 0) return;
  try {
    await appWindow.startDragging();
  } catch (error) {
    console.error('拖动窗口失败:', error);
  }
};

// 获取当前图片
const currentImage = () => {
  if (images.value.length === 0) return null;
  return images.value[currentIndex.value];
};

// 根据图片尺寸计算窗口大小
const calculateWindowSize = (imageWidth: number, imageHeight: number): { width: number; height: number } => {
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
};

// 获取图片尺寸
const getImageDimensions = async (src: string): Promise<{ width: number; height: number }> => {
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
};

// 计算缩放比例显示
const scalePercent = computed(() => Math.round(scale.value * 100));

// 重置缩放和平移
const resetTransform = () => {
  scale.value = 1.0;
  translateX.value = 0;
  translateY.value = 0;
};

// 滚轮缩放（以鼠标位置为中心）
const handleWheel = (e: WheelEvent) => {
  e.preventDefault();
  const delta = e.deltaY > 0 ? -0.1 : 0.1;
  const newScale = Math.max(0.25, Math.min(3.0, scale.value + delta));

  if (imageContainer.value) {
    const rect = imageContainer.value.getBoundingClientRect();
    // 鼠标相对于容器的位置
    const mouseX = e.clientX - rect.left;
    const mouseY = e.clientY - rect.top;

    // 容器中心点
    const centerX = rect.width / 2;
    const centerY = rect.height / 2;

    // 鼠标相对于中心的偏移
    const offsetX = mouseX - centerX;
    const offsetY = mouseY - centerY;

    // 计算缩放比例变化
    const scaleRatio = newScale / scale.value;

    // 调整平移量，使缩放以鼠标位置为中心
    // 新的平移 = 旧的平移 + 偏移量 * (1 - 缩放比例)
    translateX.value = translateX.value + offsetX * (1 - scaleRatio);
    translateY.value = translateY.value + offsetY * (1 - scaleRatio);
  }

  scale.value = newScale;
};

// 鼠标按下开始拖动图片
const handleMouseDown = (e: MouseEvent) => {
  if (e.button !== 0) return;
  isDragging = true;
  startX = e.clientX - translateX.value;
  startY = e.clientY - translateY.value;
  if (imageContainer.value) {
    imageContainer.value.style.cursor = 'grabbing';
  }
};

// 鼠标移动拖动图片
const handleMouseMove = (e: MouseEvent) => {
  if (!isDragging) return;
  translateX.value = e.clientX - startX;
  translateY.value = e.clientY - startY;
};

// 鼠标松开停止拖动
const handleMouseUp = () => {
  isDragging = false;
  if (imageContainer.value) {
    imageContainer.value.style.cursor = 'grab';
  }
};

// 上一张图片
const prevImage = () => {
  if (currentIndex.value > 0) {
    currentIndex.value--;
    resetTransform();
  }
};

// 下一张图片
const nextImage = () => {
  if (currentIndex.value < images.value.length - 1) {
    currentIndex.value++;
    resetTransform();
  }
};

// 键盘快捷键
const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    closeWindow();
  } else if (e.key === 'ArrowLeft') {
    prevImage();
  } else if (e.key === 'ArrowRight') {
    nextImage();
  } else if (e.key === '0') {
    resetTransform();
  } else if (e.key === '+' || e.key === '=') {
    scale.value = Math.min(3.0, scale.value + 0.1);
  } else if (e.key === '-') {
    scale.value = Math.max(0.25, scale.value - 0.1);
  }
};
</script>

<template>
  <div class="image-viewer">
    <!-- 可拖拽的标题栏 -->
    <div class="title-bar" @mousedown="startDrag">
      <div class="title-bar-content">
        <span class="app-title">📷 图片查看器</span>
        <div class="title-bar-buttons">
          <button
            class="control-btn reset-btn"
            @click.stop="resetTransform"
            @mousedown.stop
            title="重置缩放 (按 0)"
          >
            🔄
          </button>
          <span class="scale-display" @mousedown.stop>{{ scalePercent }}%</span>
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

    <!-- 图片容器 -->
    <div
      class="image-container"
      ref="imageContainer"
      @wheel="handleWheel"
      @mousedown="handleMouseDown"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
      @mouseleave="handleMouseUp"
    >
      <div v-if="currentImage()" class="image-wrapper">
        <img
          :src="currentImage()!.src"
          alt="查看图片"
          draggable="false"
          @dragstart.prevent
          :style="{
            transform: `translate(${translateX}px, ${translateY}px) scale(${scale})`,
            cursor: isDragging ? 'grabbing' : 'grab'
          }"
        />
      </div>
      <div v-else class="no-image">
        <p>暂无图片</p>
      </div>

      <!-- 多图片切换按钮 -->
      <button
        v-if="images.length > 1 && currentIndex > 0"
        class="nav-btn prev-btn"
        @click="prevImage"
        @mousedown.stop
      >‹</button>
      <button
        v-if="images.length > 1 && currentIndex < images.length - 1"
        class="nav-btn next-btn"
        @click="nextImage"
        @mousedown.stop
      >›</button>

      <!-- 图片计数 -->
      <div v-if="images.length > 1" class="image-counter">
        {{ currentIndex + 1 }} / {{ images.length }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.image-viewer {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #000000;  /* 改为纯黑色，确保可见 */
}

/* 标题栏样式 */
.title-bar {
  width: 100%;
  height: 36px;
  background: rgba(26, 26, 46, 0.95);  /* 增加不透明度 */
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

.title-bar-buttons {
  display: flex;
  gap: 6px;
  align-items: center;
}

.control-btn {
  width: 28px;
  height: 28px;
  border: none;
  background-color: rgba(255, 255, 255, 0.2);
  color: white;
  font-size: 16px;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
}

.control-btn:hover {
  background-color: rgba(255, 255, 255, 0.3);
  transform: scale(1.05);
}

.scale-display {
  padding: 4px 8px;
  background-color: rgba(255, 255, 255, 0.1);
  color: white;
  font-size: 12px;
  border-radius: 4px;
  min-width: 45px;
  text-align: center;
  user-select: none;
}

.pin-btn {
  width: 28px;
  height: 28px;
  border: none;
  background-color: rgba(255, 255, 255, 0.2);
  color: white;
  font-size: 16px;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
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

.close-btn {
  width: 28px;
  height: 28px;
  border: none;
  background-color: rgba(255, 255, 255, 0.2);
  color: white;
  font-size: 20px;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background-color: #ff4444;
  transform: scale(1.05);
}

/* 图片容器 */
.image-container {
  flex: 1;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
}

.image-wrapper {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
}

.image-wrapper img {
  max-width: none;
  max-height: none;
  object-fit: contain;
  user-select: none;
  transition: transform 0.1s ease-out;
}

.no-image {
  color: rgba(255, 255, 255, 0.5);
  font-size: 16px;
}

/* 导航按钮 */
.nav-btn {
  position: absolute;
  top: 50%;
  transform: translateY(-50%);
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  border: none;
  font-size: 32px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-btn:hover {
  background: rgba(0, 0, 0, 0.8);
  transform: translateY(-50%) scale(1.1);
}

.prev-btn {
  left: 12px;
}

.next-btn {
  right: 12px;
}

/* 图片计数器 */
.image-counter {
  position: absolute;
  bottom: 12px;
  left: 50%;
  transform: translateX(-50%);
  padding: 6px 12px;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  border-radius: 12px;
  font-size: 14px;
  user-select: none;
}
</style>
