<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useTodoStore } from '../stores/todoStore';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { readFile } from '@tauri-apps/plugin-fs';

const todoStore = useTodoStore();
const inputValue = ref('');
const isSubmitting = ref(false);
const selectedImages = ref<Array<{ path: string; preview: string }>>([]);
const isDragging = ref(false);

// 选择图片（支持多选）
const selectImage = async () => {
  try {
    const files = await open({
      multiple: true,  // 改为 true
      filters: [{
        name: 'Image',
        extensions: ['png', 'jpg', 'jpeg', 'gif', 'bmp', 'webp']
      }]
    });

    if (files) {
      const fileArray = Array.isArray(files) ? files : [files];
      for (const filePath of fileArray) {
        // 读取文件内容
        const fileContent = await readFile(filePath);

        // 从路径中提取文件名
        const fileName = filePath.split(/[/\\]/).pop() || 'image.jpg';

        // 调用 Rust 命令保存图片
        const savedPath = await invoke<string>('save_image', {
          fileData: Array.from(fileContent),
          fileName: fileName
        });

        // 创建预览 URL
        const blob = new Blob([fileContent], { type: 'image/jpeg' });
        const preview = URL.createObjectURL(blob);

        selectedImages.value.push({ path: savedPath, preview });
      }
    }
  } catch (error) {
    console.error('Failed to select image:', error);
  }
};

// 移除单张图片
const removeImage = (index: number) => {
  const image = selectedImages.value[index];
  if (image.preview) {
    URL.revokeObjectURL(image.preview);
  }
  selectedImages.value.splice(index, 1);
};

// 清空所有图片
const clearImages = () => {
  selectedImages.value.forEach(img => {
    if (img.preview) URL.revokeObjectURL(img.preview);
  });
  selectedImages.value = [];
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
      source: selectedImages.value.length > 0 ? '图片上传' : '手动输入',
      imagePaths: selectedImages.value.map(img => img.path),
    });
    inputValue.value = '';
    clearImages();
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

  // 处理多个文件
  for (let i = 0; i < files.length; i++) {
    const file = files[i];
    if (file.type.startsWith('image/')) {
      try {
        await processImageFile(file);
      } catch (error) {
        console.error('Failed to drop image:', error);
      }
    }
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

  // 创建预览 URL
  const blob = new Blob([uint8Array], { type: file.type });
  const preview = URL.createObjectURL(blob);

  selectedImages.value.push({ path: savedPath, preview });
};

// 组件挂载时添加粘贴事件监听
onMounted(() => {
  window.addEventListener('paste', handlePaste);
});

// 组件卸载时移除粘贴事件监听
onUnmounted(() => {
  window.removeEventListener('paste', handlePaste);
  // 清理所有预览 URL
  selectedImages.value.forEach(img => {
    if (img.preview) URL.revokeObjectURL(img.preview);
  });
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

    <!-- 图片预览区域（横向排列） -->
    <div v-if="selectedImages.length > 0" class="images-preview-container">
      <div class="images-preview">
        <div
          v-for="(image, index) in selectedImages"
          :key="index"
          class="image-preview-item"
        >
          <img :src="image.preview" alt="预览" />
          <button
            class="remove-image"
            @click="removeImage(index)"
            title="移除图片"
          >×</button>
        </div>
      </div>
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
  background: rgba(30, 30, 40, 0.5);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  transition: all 0.2s;
  backdrop-filter: blur(10px);
}

.add-todo-form.dragging {
  border-color: #667eea;
  background: rgba(40, 40, 60, 0.6);
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

.images-preview-container {
  width: 100%;
  overflow-x: auto;
  padding: 4px 0;
}

.images-preview {
  display: flex;
  gap: 8px;
  min-height: 80px;
}

.image-preview-item {
  position: relative;
  width: 80px;
  height: 80px;
  border-radius: 6px;
  overflow: hidden;
  background: rgba(50, 50, 60, 0.8);
  flex-shrink: 0;
}

.image-preview-item img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.remove-image {
  position: absolute;
  top: 4px;
  right: 4px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: rgba(0, 0, 0, 0.6);
  color: white;
  border: none;
  font-size: 16px;
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
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
  background: rgba(50, 50, 60, 0.6);
  color: #e0e0e0;
}

.input::placeholder {
  color: #666;
}

.input:focus {
  border-color: #667eea;
  background: rgba(50, 50, 60, 0.8);
}

.input:disabled {
  background: rgba(30, 30, 40, 0.5);
  cursor: not-allowed;
}

.image-btn {
  padding: 8px 12px;
  background: rgba(100, 100, 120, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.2);
  border-radius: 6px;
  font-size: 20px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.image-btn:hover:not(:disabled) {
  background: rgba(100, 100, 120, 0.5);
  transform: translateY(-1px);
}

.image-btn:disabled {
  opacity: 0.3;
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
