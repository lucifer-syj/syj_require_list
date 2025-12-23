<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';
import { useTodoStore } from '../stores/todoStore';
import { confirm } from '@tauri-apps/plugin-dialog';
import { openImageViewer } from '../composables/useImageViewer';
import type { Todo } from '../types/todo';
import type { AppConfig } from '../types/config';

const appWindow = getCurrentWindow();
const todoStore = useTodoStore();

// 选项卡状态
const activeTab = ref<'list' | 'trash'>('list');

// 图片缓存：存储每个 todo 的图片 Blob URLs
const imageCache = ref<Map<number, Array<{ path: string; src: string }>>>(new Map());

// 待办列表（未删除）
const todoList = computed(() => {
  return todoStore.todos.filter(todo => !todo.isDeleted);
});

// 回收站列表（已删除）
const trashList = computed(() => {
  return todoStore.todos.filter(todo => todo.isDeleted);
});

// 按日期分组的待办列表
const groupedTodoList = computed(() => {
  const groups = new Map<string, Todo[]>();

  todoList.value.forEach(todo => {
    // 提取日期部分（YYYY-MM-DD）
    const date = todo.createdAt.split(' ')[0];
    if (!groups.has(date)) {
      groups.set(date, []);
    }
    groups.get(date)!.push(todo);
  });

  // 转换为数组并排序（最新的日期在前）
  return Array.from(groups.entries())
    .map(([date, todos]) => ({ date, todos }))
    .sort((a, b) => b.date.localeCompare(a.date));
});

// 格式化日期显示
const formatDate = (dateString: string) => {
  const today = new Date();
  const date = new Date(dateString);

  // 重置时间为 00:00:00 以便比较日期
  today.setHours(0, 0, 0, 0);
  date.setHours(0, 0, 0, 0);

  const diffTime = today.getTime() - date.getTime();
  const diffDays = Math.floor(diffTime / (1000 * 60 * 60 * 24));

  const year = date.getFullYear();
  const month = date.getMonth() + 1;
  const day = date.getDate();

  // 格式化具体日期
  const todayYear = today.getFullYear();
  const absoluteDate = year === todayYear
    ? `${month}月${day}日`
    : `${year}年${month}月${day}日`;

  if (diffDays === 0) {
    return `今天 · ${absoluteDate}`;
  } else if (diffDays === 1) {
    return `昨天 · ${absoluteDate}`;
  } else if (diffDays < 7) {
    return `${diffDays}天前 · ${absoluteDate}`;
  } else {
    // 7天以上只显示具体日期
    return absoluteDate;
  }
};

// 加载数据
const loadData = async () => {
  // 同时加载未删除和已删除的待办，这样两个选项卡的数量都能正确显示
  try {
    const [normalTodos, deletedTodos] = await Promise.all([
      invoke<Todo[]>('get_todos', { includeDeleted: false }),
      invoke<Todo[]>('get_todos', { includeDeleted: true })
    ]);
    // 合并两种数据到 store
    todoStore.todos = [...normalTodos, ...deletedTodos];
    // 加载所有待办的图片
    await loadAllImages();
  } catch (e) {
    console.error('Failed to load todos:', e);
  }
};

// 加载单个待办的图片
const loadTodoImages = async (todo: Todo) => {
  if (!todo.id || !todo.images || todo.images.length === 0) {
    return;
  }

  try {
    const imageSources: Array<{ path: string; src: string }> = [];
    for (const image of todo.images) {
      const fileData = await invoke<number[]>('read_image', {
        filePath: image.imagePath
      });
      const uint8Array = new Uint8Array(fileData);
      const blob = new Blob([uint8Array], { type: 'image/jpeg' });
      const blobUrl = URL.createObjectURL(blob);
      imageSources.push({
        path: image.imagePath,
        src: blobUrl
      });
    }
    imageCache.value.set(todo.id, imageSources);
  } catch (error) {
    console.error(`加载待办 ${todo.id} 的图片失败:`, error);
  }
};

// 加载所有待办的图片
const loadAllImages = async () => {
  const todos = [...todoList.value, ...trashList.value];
  await Promise.all(todos.map(todo => loadTodoImages(todo)));
};

// 获取待办的图片
const getTodoImages = (todoId: number | undefined) => {
  if (!todoId) return [];
  return imageCache.value.get(todoId) || [];
};

// 查看图片
const viewImage = async (todoId: number | undefined, index: number) => {
  if (!todoId) return;
  const images = getTodoImages(todoId);
  if (images.length === 0) return;
  try {
    await openImageViewer(images, index);
  } catch (error) {
    console.error('打开图片查看器失败:', error);
  }
};

// 切换选项卡
const switchTab = (tab: 'list' | 'trash') => {
  activeTab.value = tab;
  // 不需要重新加载数据，因为所有数据已经加载了
};

// 恢复待办
const handleRestore = async (id: number) => {
  try {
    await todoStore.restoreTodo(id);
    await loadData();
  } catch (error) {
    console.error('恢复失败:', error);
  }
};

// 永久删除
const handlePermanentDelete = async (id: number) => {
  const confirmed = await confirm('确定要永久删除这个待办吗？此操作无法撤销。', {
    title: '确认删除',
    kind: 'warning'
  });

  if (confirmed) {
    try {
      await todoStore.permanentDeleteTodo(id);
      await loadData();
    } catch (error) {
      console.error('删除失败:', error);
    }
  }
};

// 关闭窗口
const closeWindow = async () => {
  // 保存窗口尺寸
  try {
    // 如果 currentSize 为 null，主动获取当前尺寸
    if (!currentSize) {
      const size = await appWindow.outerSize();
      currentSize = { width: size.width, height: size.height };
    }

    const config = await invoke<AppConfig>('get_config');
    config.allTodosWindow = currentSize;
    await invoke('save_config_only', { config });
    console.log('保存全部待办窗口尺寸:', currentSize);
  } catch (error) {
    console.error('保存全部待办窗口尺寸失败:', error);
  }

  await appWindow.hide();
};

// 监听事件
let unlistenDeleted: (() => void) | null = null;
let unlistenRestored: (() => void) | null = null;
let unlistenPermanentDeleted: (() => void) | null = null;
let unlistenResize: (() => void) | null = null;

// 窗口尺寸记忆
let currentSize: { width: number; height: number } | null = null;
const MIN_WIDTH = 600;
const MIN_HEIGHT = 400;

onMounted(async () => {
  await loadData();

  // 监听窗口尺寸变化
  unlistenResize = await appWindow.onResized((event) => {
    currentSize = { width: event.payload.width, height: event.payload.height };
    console.log('全部待办窗口尺寸变化:', currentSize);
  });

  // 监听数据变更事件
  unlistenDeleted = await listen('todo-deleted', async () => {
    await loadData();
  });

  unlistenRestored = await listen('todo-restored', async () => {
    await loadData();
  });

  unlistenPermanentDeleted = await listen('todo-permanent-deleted', async () => {
    await loadData();
  });
});

onBeforeUnmount(() => {
  if (unlistenDeleted) unlistenDeleted();
  if (unlistenRestored) unlistenRestored();
  if (unlistenPermanentDeleted) unlistenPermanentDeleted();
  if (unlistenResize) unlistenResize();

  // 清理所有 Blob URLs
  imageCache.value.forEach(images => {
    images.forEach(img => {
      if (img.src) URL.revokeObjectURL(img.src);
    });
  });
  imageCache.value.clear();
});
</script>

<template>
  <div class="all-todos">
    <!-- 标题栏 -->
    <div class="title-bar" @mousedown="appWindow.startDragging()">
      <span class="title">所有待办</span>
      <button class="close-btn" @click="closeWindow" @mousedown.stop>×</button>
    </div>

    <!-- 选项卡 -->
    <div class="tabs">
      <button
        :class="['tab', { active: activeTab === 'list' }]"
        @click="switchTab('list')"
      >
        待办列表 ({{ todoList.length }})
      </button>
      <button
        :class="['tab', { active: activeTab === 'trash' }]"
        @click="switchTab('trash')"
      >
        回收站 ({{ trashList.length }})
      </button>
    </div>

    <!-- 内容区域 -->
    <div class="content">
      <!-- 加载状态 -->
      <div v-if="todoStore.loading" class="loading">加载中...</div>

      <!-- 待办列表选项卡 -->
      <div v-else-if="activeTab === 'list'" class="todo-list">
        <div v-if="todoList.length === 0" class="empty">
          暂无待办事项
        </div>
        <div v-else class="todo-items">
          <!-- 按日期分组显示 -->
          <div v-for="group in groupedTodoList" :key="group.date" class="date-group">
            <!-- 日期分隔符 -->
            <div class="date-separator">
              <span class="date-text">{{ formatDate(group.date) }}</span>
              <span class="date-count">{{ group.todos.length }} 项</span>
            </div>

            <!-- 该日期下的待办列表 -->
            <div v-for="todo in group.todos" :key="todo.id" class="todo-item">
              <div class="todo-header">
                <input
                  type="checkbox"
                  :checked="todo.isCompleted"
                  @change="todoStore.toggleTodo(todo.id!)"
                  class="checkbox"
                />
                <span :class="['todo-content', { completed: todo.isCompleted }]">
                  {{ todo.content }}
                </span>
              </div>
              <!-- 图片缩略图 -->
              <div v-if="getTodoImages(todo.id).length > 0" class="images-thumbnails">
                <div
                  v-for="(image, index) in getTodoImages(todo.id)"
                  :key="index"
                  class="image-thumbnail"
                  @click="viewImage(todo.id, index)"
                  :title="`点击查看大图 (${index + 1}/${getTodoImages(todo.id).length})`"
                >
                  <img :src="image.src" alt="缩略图" />
                </div>
              </div>
              <div class="todo-meta">
                <span class="time">{{ todo.createdAt.split(' ')[1] }}</span>
                <span v-if="todo.finishedAt" class="time">完成于: {{ todo.finishedAt }}</span>
                <span v-if="getTodoImages(todo.id).length > 0" class="image-count">
                  📷 {{ getTodoImages(todo.id).length }}
                </span>
              </div>
              <div class="todo-actions">
                <button @click="todoStore.deleteTodo(todo.id!)" class="btn-delete">
                  删除
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 回收站选项卡 -->
      <div v-else class="trash-list">
        <div v-if="trashList.length === 0" class="empty">
          回收站为空
        </div>
        <div v-else class="trash-items">
          <div v-for="todo in trashList" :key="todo.id" class="trash-item">
            <div class="todo-header">
              <span class="todo-content deleted">{{ todo.content }}</span>
            </div>
            <!-- 图片缩略图 -->
            <div v-if="getTodoImages(todo.id).length > 0" class="images-thumbnails">
              <div
                v-for="(image, index) in getTodoImages(todo.id)"
                :key="index"
                class="image-thumbnail"
                @click="viewImage(todo.id, index)"
                :title="`点击查看大图 (${index + 1}/${getTodoImages(todo.id).length})`"
              >
                <img :src="image.src" alt="缩略图" />
              </div>
            </div>
            <div class="todo-meta">
              <span class="time">创建于: {{ todo.createdAt }}</span>
              <span v-if="todo.deletedAt" class="time">删除于: {{ todo.deletedAt }}</span>
              <span v-if="getTodoImages(todo.id).length > 0" class="image-count">
                📷 {{ getTodoImages(todo.id).length }}
              </span>
            </div>
            <div class="todo-actions">
              <button @click="handleRestore(todo.id!)" class="btn-restore">
                恢复
              </button>
              <button @click="handlePermanentDelete(todo.id!)" class="btn-permanent-delete">
                永久删除
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.all-todos {
  width: 100%;
  height: 100vh;
  display: flex;
  flex-direction: column;
  background-color: #1a1a2e;
}

/* 标题栏 */
.title-bar {
  width: 100%;
  height: 40px;
  background: #16213e;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 16px;
  cursor: move;
  user-select: none;
}

.title {
  color: white;
  font-size: 16px;
  font-weight: 600;
}

.close-btn {
  width: 32px;
  height: 32px;
  border: none;
  background-color: rgba(255, 255, 255, 0.2);
  color: white;
  font-size: 24px;
  cursor: pointer;
  border-radius: 4px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.close-btn:hover {
  background-color: #ff4444;
}

/* 选项卡 */
.tabs {
  display: flex;
  background: rgba(26, 26, 46, 0.3);
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.tab {
  flex: 1;
  height: 48px;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.6);
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
  border-bottom: 2px solid transparent;
}

.tab:hover {
  color: rgba(255, 255, 255, 0.8);
  background: rgba(255, 255, 255, 0.05);
}

.tab.active {
  color: white;
  border-bottom-color: #4a9eff;
  background: rgba(74, 158, 255, 0.1);
}

/* 内容区域 */
.content {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
}

.loading,
.empty {
  text-align: center;
  color: rgba(255, 255, 255, 0.5);
  padding: 40px 20px;
  font-size: 14px;
}

/* 日期分组 */
.date-group {
  margin-bottom: 24px;
}

.date-separator {
  position: sticky;
  top: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: rgba(26, 26, 46, 0.95);
  padding: 10px 16px;
  margin-bottom: 12px;
  border-radius: 6px;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  z-index: 10;
}

.date-text {
  color: rgba(255, 255, 255, 0.9);
  font-size: 14px;
  font-weight: 600;
  letter-spacing: 0.5px;
}

.date-count {
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
}

/* 待办项 */
.todo-item,
.trash-item {
  background: rgba(255, 255, 255, 0.05);
  border-radius: 8px;
  padding: 12px 16px;
  margin-bottom: 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  transition: all 0.2s;
}

.todo-item:hover,
.trash-item:hover {
  background: rgba(255, 255, 255, 0.08);
  border-color: rgba(255, 255, 255, 0.2);
}

.todo-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 6px;
}

.checkbox {
  width: 20px;
  height: 20px;
  cursor: pointer;
}

.todo-content {
  flex: 1;
  color: white;
  font-size: 15px;
  line-height: 1.5;
}

.todo-content.completed {
  text-decoration: line-through;
  opacity: 0.6;
}

.todo-content.deleted {
  opacity: 0.5;
}

/* 图片缩略图 */
.images-thumbnails {
  display: flex;
  gap: 8px;
  margin-bottom: 6px;
  flex-wrap: wrap;
}

.image-thumbnail {
  width: 60px;
  height: 60px;
  border-radius: 4px;
  overflow: hidden;
  cursor: pointer;
  border: 2px solid rgba(255, 255, 255, 0.1);
  transition: all 0.2s;
}

.image-thumbnail:hover {
  border-color: rgba(74, 158, 255, 0.5);
  transform: scale(1.05);
}

.image-thumbnail img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.todo-meta {
  display: flex;
  gap: 16px;
  margin-bottom: 8px;
}

.time {
  color: rgba(255, 255, 255, 0.5);
  font-size: 12px;
}

.image-count {
  color: rgba(255, 255, 255, 0.6);
  font-size: 12px;
}

.todo-actions {
  display: flex;
  gap: 8px;
  justify-content: flex-end;
}

.btn-delete,
.btn-restore,
.btn-permanent-delete {
  padding: 6px 16px;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-delete {
  background-color: rgba(255, 68, 68, 0.2);
  color: #ff6b6b;
}

.btn-delete:hover {
  background-color: rgba(255, 68, 68, 0.3);
}

.btn-restore {
  background-color: rgba(74, 158, 255, 0.2);
  color: #4a9eff;
}

.btn-restore:hover {
  background-color: rgba(74, 158, 255, 0.3);
}

.btn-permanent-delete {
  background-color: rgba(255, 68, 68, 0.3);
  color: #ff4444;
}

.btn-permanent-delete:hover {
  background-color: rgba(255, 68, 68, 0.5);
}

/* 滚动条样式 */
.content::-webkit-scrollbar {
  width: 8px;
}

.content::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
}

.content::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 4px;
}

.content::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}
</style>
