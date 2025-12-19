<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { useTodoStore } from '../stores/todoStore';
import type { Todo } from '../types/todo';
import { invoke } from '@tauri-apps/api/core';
import { confirm } from '@tauri-apps/plugin-dialog';

const props = defineProps<{
  todo: Todo;
}>();

const todoStore = useTodoStore();
const showImageModal = ref(false);
const imageSrc = ref<string | null>(null);
const imageLoading = ref(false);

// 加载图片
const loadImage = async () => {
  if (!props.todo.imagePath) {
    console.log('❌ 没有图片路径');
    return;
  }

  imageLoading.value = true;
  console.log('=== 加载图片 ===');
  console.log('图片路径:', props.todo.imagePath);

  try {
    // 调用 Rust 命令读取文件
    const fileData = await invoke<number[]>('read_image', {
      filePath: props.todo.imagePath
    });

    // 转换为 Uint8Array
    const uint8Array = new Uint8Array(fileData);

    // 创建 Blob URL
    const blob = new Blob([uint8Array], { type: 'image/jpeg' });
    const blobUrl = URL.createObjectURL(blob);

    imageSrc.value = blobUrl;
    console.log('✅ 图片加载成功');
  } catch (error) {
    console.error('❌ 图片加载失败:', error);
    imageSrc.value = null;
  } finally {
    imageLoading.value = false;
  }
};

// 组件挂载时加载图片
onMounted(() => {
  console.log('Todo data:', props.todo);
  loadImage();
});

// 监听 imagePath 变化
watch(() => props.todo.imagePath, () => {
  loadImage();
});

// 切换完成状态
const handleToggle = async () => {
  if (props.todo.id) {
    await todoStore.toggleTodo(props.todo.id);
  }
};

// 删除待办
const handleDelete = async () => {
  if (!props.todo.id) return;

  const confirmed = await confirm('确定要删除这条待办吗？', {
    title: '确认删除',
    kind: 'warning'
  });

  if (confirmed) {
    await todoStore.deleteTodo(props.todo.id);
  }
};

// 查看图片大图
const viewImage = () => {
  if (imageSrc.value) {
    showImageModal.value = true;
  }
};

// 关闭图片预览
const closeModal = () => {
  showImageModal.value = false;
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

    <!-- 图片缩略图 -->
    <div v-if="imageSrc" class="image-thumbnail" @click="viewImage" title="点击查看大图">
      <img :src="imageSrc" alt="缩略图" />
    </div>

    <!-- 内容 -->
    <div class="content">
      <div class="text">{{ todo.content }}</div>
      <div class="meta">
        <span v-if="todo.source" class="source">{{ todo.source }}</span>
        <span v-if="todo.createdAt" class="time">{{ todo.createdAt }}</span>
      </div>
    </div>

    <!-- 删除按钮 -->
    <button class="delete-btn" @click="handleDelete" title="删除">
      ×
    </button>
  </div>

  <!-- 图片查看弹窗 -->
  <Teleport to="body">
    <div v-if="showImageModal" class="image-modal" @click="closeModal">
      <div class="modal-content" @click.stop>
        <img :src="imageSrc" alt="查看大图" />
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
  background: white;
  border-radius: 8px;
  margin-bottom: 8px;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  transition: all 0.2s;
}

.todo-item:hover {
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
}

.todo-item.completed {
  opacity: 0.6;
  background: #f5f5f5;
}

.checkbox {
  width: 18px;
  height: 18px;
  cursor: pointer;
  flex-shrink: 0;
}

.image-thumbnail {
  width: 50px;
  height: 50px;
  border-radius: 6px;
  overflow: hidden;
  background: #f5f5f5;
  flex-shrink: 0;
  cursor: pointer;
  transition: all 0.2s;
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
  color: #333;
  word-break: break-word;
}

.todo-item.completed .text {
  text-decoration: line-through;
  color: #999;
}

.meta {
  display: flex;
  gap: 8px;
  margin-top: 4px;
  font-size: 12px;
  color: #999;
}

.source {
  padding: 2px 6px;
  background: #e3f2fd;
  color: #1976d2;
  border-radius: 3px;
  font-size: 11px;
}

.time {
  color: #999;
}

.delete-btn {
  width: 24px;
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
  padding: 0;
}

.delete-btn:hover {
  background: #cc0000;
  transform: scale(1.1);
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
</style>
