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

    <!-- 待办列表 -->
    <div class="todo-items">
      <TodoItem
        v-for="todo in todoStore.todos"
        :key="todo.id"
        :todo="todo"
      />
    </div>

    <!-- 空状态 -->
    <div v-if="!todoStore.loading && todoStore.todos.length === 0" class="empty">
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
  color: #999;
  font-size: 14px;
}

.error {
  color: #ff4444;
}

.todo-items {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

/* 滚动条样式 */
.todo-items::-webkit-scrollbar {
  width: 6px;
}

.todo-items::-webkit-scrollbar-track {
  background: #f1f1f1;
}

.todo-items::-webkit-scrollbar-thumb {
  background: #888;
  border-radius: 3px;
}

.todo-items::-webkit-scrollbar-thumb:hover {
  background: #555;
}
</style>
