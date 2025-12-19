<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useTodoStore } from '../stores/todoStore';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { readFile } from '@tauri-apps/plugin-fs';

const todoStore = useTodoStore();
const inputValue = ref('');
const isSubmitting = ref(false);
const selectedImage = ref<string | null>(null);
const selectedImagePath = ref<string | null>(null);
const isDragging = ref(false);

// 选择图片
const selectImage = async () => {
  try {
    const file = await open({
      multiple: false,
      filters: [{
        name: 'Image',
        extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp']
      }]
    });

    if (file) {
      // 读取文件内容
      const fileContent = await readFile(file.path);

      // 调用 Rust 命令保存图片
      const savedPath = await invoke<string>('save_image', {
        fileData: Array.from(fileContent),
        fileName: file.name || 'image.jpg'
      });

      // 设置预览和路径
      selectedImagePath.value = savedPath;

      // 创建预览 URL (从文件路径读取)
      const blob = new Blob([fileContent], { type: 'image/jpeg' });
      selectedImage.value = URL.createObjectURL(blob);
    }
  } catch (error) {
    console.error('Failed to select image:', error);
  }
};

// 移除图片
const removeImage = () => {
  if (selectedImage.value) {
    URL.revokeObjectURL(selectedImage.value);
  }
  selectedImage.value = null;
  selectedImagePath.value = null;
};

// 添加待办
const handleSubmit = async () => {
  const content = inputValue.value.trim();
  if (!content) {
    return;
  }

  isSubmitting.value = true;
  try {
    await todoStore.addTodo({
      content,
      source: selectedImage.value ? '图片上传' : '手动输入',
      imagePath: selectedImagePath.value || undefined,
    });
    inputValue.value = '';
    removeImage();
  } catch (error) {
    console.error('Failed to add todo:', error);
  } finally {
    isSubmitting.value = false;
  }
};

// 按 Enter 键提交
const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    handleSubmit();
  }
};

// 处理粘贴事件
const handlePaste = async (e: ClipboardEvent) => {
  const items = e.clipboardData?.items;
  if (!items) return;

  // 查找图片项
  for (let i = 0; i < items.length; i++) {
    const item = items[i];
    if (item.type.indexOf('image') !== -1) {
      e.preventDefault();

      const file = item.getAsFile();
      if (!file) continue;

      try {
        await processImageFile(file);
      } catch (error) {
        console.error('Failed to paste image:', error);
      }

      break;
    }
  }
};

// 处理拖拽进入和经过
const handleDragOver = (e: DragEvent) => {
  e.preventDefault();
  e.stopPropagation();
  isDragging.value = true;
};

// 处理拖拽离开
const handleDragLeave = (e: DragEvent) => {
  e.preventDefault();
  e.stopPropagation();
  isDragging.value = false;
};

// 处理文件放下
const handleDrop = async (e: DragEvent) => {
  e.preventDefault();
  e.stopPropagation();
  isDragging.value = false;

  const files = e.dataTransfer?.files;
  if (!files || files.length === 0) return;

  const file = files[0];

  // 检查是否为图片文件
  if (!file.type.startsWith('image/')) {
    alert('请拖拽图片文件');
    return;
  }

  try {
    await processImageFile(file);
  } catch (error) {
    console.error('Failed to drop image:', error);
  }
};

// 处理图片文件的通用方法
const processImageFile = async (file: File) => {
  // 读取文件数据
  const arrayBuffer = await file.arrayBuffer();
  const uint8Array = new Uint8Array(arrayBuffer);

  // 调用 Rust 命令保存图片
  const savedPath = await invoke<string>('save_image', {
    fileData: Array.from(uint8Array),
    fileName: file.name || 'image.png'
  });

  // 设置预览和路径
  selectedImagePath.value = savedPath;

  // 创建预览 URL
  const blob = new Blob([uint8Array], { type: file.type });
  if (selectedImage.value) {
    URL.revokeObjectURL(selectedImage.value);
  }
  selectedImage.value = URL.createObjectURL(blob);
};

// 组件挂载时添加粘贴事件监听
onMounted(() => {
  window.addEventListener('paste', handlePaste);
});

// 组件卸载时移除粘贴事件监听
onUnmounted(() => {
  window.removeEventListener('paste', handlePaste);
  // 清理预览 URL
  if (selectedImage.value) {
    URL.revokeObjectURL(selectedImage.value);
  }
});
</script>

<template>
  <div
    class="add-todo-form"
    :class="{ 'dragging': isDragging }"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
  >
    <!-- 拖拽提示遮罩 -->
    <div v-if="isDragging" class="drag-overlay">
      <div class="drag-hint">
        📷 拖拽图片到这里上传
      </div>
    </div>
    <!-- 图片预览区域 -->
    <div v-if="selectedImage" class="image-preview">
      <img :src="selectedImage" alt="预览" />
      <button class="remove-image" @click="removeImage" title="移除图片">×</button>
    </div>

    <div class="input-row">
      <input
        v-model="inputValue"
        type="text"
        placeholder="输入待办... (可 Ctrl+V 粘贴或拖拽图片)"
        class="input"
        :disabled="isSubmitting"
        @keydown="handleKeydown"
      />
      <button
        class="image-btn"
        :disabled="isSubmitting"
        @click="selectImage"
        title="上传图片"
      >
        📷
      </button>
      <button
        class="add-btn"
        :disabled="!inputValue.trim() || isSubmitting"
        @click="handleSubmit"
      >
        {{ isSubmitting ? '添加中...' : '添加' }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.add-todo-form {
  position: relative;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 12px;
  background: white;
  border-bottom: 1px solid #e0e0e0;
  transition: all 0.2s;
}

.add-todo-form.dragging {
  border-color: #667eea;
  background: #f0f4ff;
}

.drag-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(102, 126, 234, 0.1);
  border: 2px dashed #667eea;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10;
  pointer-events: none;
}

.drag-hint {
  padding: 20px 40px;
  background: white;
  border-radius: 8px;
  font-size: 16px;
  color: #667eea;
  font-weight: 500;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.2);
}

.image-preview {
  position: relative;
  width: 100%;
  max-height: 150px;
  border-radius: 6px;
  overflow: hidden;
  background: #f5f5f5;
}

.image-preview img {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.remove-image {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  border: none;
  font-size: 20px;
  line-height: 1;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.remove-image:hover {
  background: rgba(0, 0, 0, 0.8);
  transform: scale(1.1);
}

.input-row {
  display: flex;
  gap: 8px;
}

.input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.input:focus {
  border-color: #667eea;
}

.input:disabled {
  background: #f5f5f5;
  cursor: not-allowed;
}

.image-btn {
  padding: 8px 12px;
  background: #f0f0f0;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 20px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.image-btn:hover:not(:disabled) {
  background: #e0e0e0;
  transform: translateY(-1px);
}

.image-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.add-btn {
  padding: 8px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.add-btn:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.4);
}

.add-btn:active:not(:disabled) {
  transform: translateY(0);
}

.add-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
