<script setup lang="ts">
import { onMounted } from 'vue';
import { useTodoStore } from '../stores/todoStore';
import TodoItem from './TodoItem.vue';
import AddTodoForm from './AddTodoForm.vue';

const todoStore = useTodoStore();

// 组件挂载时加载待办列表
onMounted(() => {
  todoStore.fetchTodos();
});
</script>

<template>
  <div class="todo-list">
    <!-- 添加待办表单 -->
    <AddTodoForm />

    <!-- 加载状态 -->
    <div v-if="todoStore.loading" class="loading">加载中...</div>

    <!-- 错误提示 -->
    <div v-if="todoStore.error" class="error">{{ todoStore.error }}</div>

    <!-- 待办列表（只显示未完成且未删除的待办） -->
    <div class="todo-items">
      <TodoItem
        v-for="todo in todoStore.todos.filter(t => !t.isCompleted && !t.isDeleted)"
        :key="todo.id"
        :todo="todo"
      />
    </div>

    <!-- 空状态 -->
    <div v-if="!todoStore.loading && todoStore.todos.filter(t => !t.isCompleted && !t.isDeleted).length === 0" class="empty">
      暂无待办事项，点击上方添加吧！
    </div>
  </div>
</template>

<style scoped>
.todo-list {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.loading,
.error,
.empty {
  padding: 20px;
  text-align: center;
  color: #888;
  font-size: 14px;
}

.error {
  color: #ff6b6b;
}

.todo-items {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

/* 滚动条样式 - 深色 */
.todo-items::-webkit-scrollbar {
  width: 6px;
}

.todo-items::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
}

.todo-items::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2);
  border-radius: 3px;
}

.todo-items::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.3);
}
</style>
