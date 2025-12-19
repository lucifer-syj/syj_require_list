<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import { useTodoStore } from '../stores/todoStore';
import type { Todo } from '../types/todo';
import { invoke } from '@tauri-apps/api/core';

const props = defineProps<{
  todo: Todo;
}>();

const todoStore = useTodoStore();
const showImageModal = ref(false);
const imageSources = ref<Array<{ path: string; src: string }>>([]);
const imageLoading = ref(false);
const currentImageIndex = ref(0);
const isDeleteConfirm = ref(false); // 删除确认状态

// 加载多张图片
const loadImages = async () => {
  if (!props.todo.images || props.todo.images.length === 0) {
    console.log('没有图片');
    return;
  }

  imageLoading.value = true;
  imageSources.value = [];

  try {
    for (const image of props.todo.images) {
      const fileData = await invoke<number[]>('read_image', {
        filePath: image.imagePath
      });

      const uint8Array = new Uint8Array(fileData);
      const blob = new Blob([uint8Array], { type: 'image/jpeg' });
      const blobUrl = URL.createObjectURL(blob);

      imageSources.value.push({
        path: image.imagePath,
        src: blobUrl
      });
    }
    console.log(`✅ 加载了 ${imageSources.value.length} 张图片`);
  } catch (error) {
    console.error('图片加载失败:', error);
  } finally {
    imageLoading.value = false;
  }
};

// 组件挂载时加载图片
onMounted(() => {
  loadImages();
  window.addEventListener('keydown', handleKeydown);
});

// 组件卸载时清理
onUnmounted(() => {
  window.removeEventListener('keydown', handleKeydown);
  imageSources.value.forEach(img => {
    if (img.src) URL.revokeObjectURL(img.src);
  });
});

// 监听 images 变化
watch(() => props.todo.images, () => {
  loadImages();
}, { deep: true });

// 切换完成状态
const handleToggle = async () => {
  if (props.todo.id) {
    await todoStore.toggleTodo(props.todo.id);
  }
};

// 删除待办
const handleDelete = async () => {
  if (!props.todo.id) return;

  // 第一次点击：进入确认状态
  if (!isDeleteConfirm.value) {
    isDeleteConfirm.value = true;
    // 3秒后自动取消确认状态
    setTimeout(() => {
      isDeleteConfirm.value = false;
    }, 3000);
    return;
  }

  // 第二次点击：真正删除
  await todoStore.deleteTodo(props.todo.id);
  isDeleteConfirm.value = false;
};

// 查看图片大图
const viewImage = (index: number) => {
  currentImageIndex.value = index;
  showImageModal.value = true;
};

// 上一张图片
const prevImage = () => {
  if (currentImageIndex.value > 0) {
    currentImageIndex.value--;
  }
};

// 下一张图片
const nextImage = () => {
  if (currentImageIndex.value < imageSources.value.length - 1) {
    currentImageIndex.value++;
  }
};

// 关闭图片预览
const closeModal = () => {
  showImageModal.value = false;
  currentImageIndex.value = 0;
};

// 键盘快捷键
const handleKeydown = (e: KeyboardEvent) => {
  if (!showImageModal.value) return;

  if (e.key === 'ArrowLeft') {
    prevImage();
  } else if (e.key === 'ArrowRight') {
    nextImage();
  } else if (e.key === 'Escape') {
    closeModal();
  }
};
</script>

<template>
  <div class="todo-item" :class="{ completed: todo.isCompleted }">
    <!-- 复选框 -->
    <input
      type="checkbox"
      :checked="todo.isCompleted"
      @change="handleToggle"
      class="checkbox"
    />

    <!-- 多张图片缩略图（横向排列） -->
    <div v-if="imageSources.length > 0" class="images-thumbnails">
      <div
        v-for="(image, index) in imageSources"
        :key="index"
        class="image-thumbnail"
        @click="viewImage(index)"
        :title="`点击查看大图 (${index + 1}/${imageSources.length})`"
      >
        <img :src="image.src" alt="缩略图" />
      </div>
    </div>

    <!-- 内容 -->
    <div class="content">
      <div class="text">{{ todo.content }}</div>
      <div class="meta">
        <span v-if="todo.source" class="source">{{ todo.source }}</span>
        <span v-if="imageSources.length > 0" class="image-count">
          📷 {{ imageSources.length }}
        </span>
        <span v-if="todo.createdAt" class="time">{{ todo.createdAt }}</span>
      </div>
    </div>

    <!-- 删除按钮 -->
    <button
      class="delete-btn"
      :class="{ 'delete-confirm': isDeleteConfirm }"
      @click="handleDelete"
      :title="isDeleteConfirm ? '再次点击确认删除' : '删除'"
    >
      {{ isDeleteConfirm ? '确认?' : '×' }}
    </button>
  </div>

  <!-- 图片查看弹窗（支持切换） -->
  <Teleport to="body">
    <div v-if="showImageModal" class="image-modal" @click="closeModal">
      <div class="modal-content" @click.stop>
        <img :src="imageSources[currentImageIndex].src" alt="查看大图" />

        <!-- 图片切换按钮 -->
        <button
          v-if="imageSources.length > 1 && currentImageIndex > 0"
          class="nav-btn prev-btn"
          @click="prevImage"
        >‹</button>
        <button
          v-if="imageSources.length > 1 && currentImageIndex < imageSources.length - 1"
          class="nav-btn next-btn"
          @click="nextImage"
        >›</button>

        <!-- 图片计数 -->
        <div v-if="imageSources.length > 1" class="image-counter">
          {{ currentImageIndex + 1 }} / {{ imageSources.length }}
        </div>

        <button class="close-modal" @click="closeModal">×</button>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.todo-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: #1a1a1a;
  border-radius: 8px;
  margin-bottom: 8px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
  transition: all 0.2s;
  border: 1px solid rgba(255, 255, 255, 0.15);
}

.todo-item:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.6);
  background: #222222;
  transform: translateY(-1px);
  border-color: rgba(255, 255, 255, 0.2);
}

.todo-item.completed {
  opacity: 0.5;
  background: #0d0d0d;
}

.checkbox {
  width: 18px;
  height: 18px;
  cursor: pointer;
  flex-shrink: 0;
}

.images-thumbnails {
  display: flex;
  gap: 4px;
  flex-shrink: 0;
}

.image-thumbnail {
  width: 50px;
  height: 50px;
  border-radius: 6px;
  overflow: hidden;
  background: rgba(50, 50, 60, 0.8);
  flex-shrink: 0;
  cursor: pointer;
  transition: all 0.2s;
  border: 1px solid rgba(255, 255, 255, 0.1);
}

.image-thumbnail:hover {
  transform: scale(1.05);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.image-thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.content {
  flex: 1;
  min-width: 0;
}

.text {
  font-size: 14px;
  color: #e0e0e0;
  word-break: break-word;
}

.todo-item.completed .text {
  text-decoration: line-through;
  color: #666;
}

.meta {
  display: flex;
  gap: 8px;
  margin-top: 4px;
  font-size: 12px;
  color: #888;
}

.source {
  padding: 2px 6px;
  background: rgba(33, 150, 243, 0.2);
  color: #64b5f6;
  border-radius: 3px;
  font-size: 11px;
}

.image-count {
  padding: 2px 6px;
  background: rgba(76, 175, 80, 0.2);
  color: #81c784;
  border-radius: 3px;
  font-size: 11px;
}

.time {
  color: #666;
}

.delete-btn {
  min-width: 24px;
  height: 24px;
  border: none;
  background: #ff4444;
  color: white;
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0 4px;
}

.delete-btn:hover {
  background: #cc0000;
  transform: scale(1.1);
}

.delete-btn.delete-confirm {
  background: #ff0000;
  animation: pulse 0.6s ease-in-out infinite;
  font-size: 12px;
  min-width: 45px;
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.05);
  }
}

/* 图片查看弹窗样式 */
.image-modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  cursor: pointer;
}

.modal-content {
  position: relative;
  max-width: 90vw;
  max-height: 90vh;
  background: white;
  border-radius: 8px;
  overflow: hidden;
  cursor: default;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

.modal-content img {
  max-width: 90vw;
  max-height: 90vh;
  display: block;
  object-fit: contain;
}

.close-modal {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  border: none;
  font-size: 24px;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.close-modal:hover {
  background: rgba(0, 0, 0, 0.8);
  transform: scale(1.1);
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
}
</style>
