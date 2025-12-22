import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { emit } from '@tauri-apps/api/event';
import type { Todo, AddTodoInput, UpdateTodoInput } from '../types/todo';

export const useTodoStore = defineStore('todo', () => {
  // 状态
  const todos = ref<Todo[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // 获取所有待办
  async function fetchTodos(includeDeleted: boolean = false) {
    loading.value = true;
    error.value = null;
    try {
      const result = await invoke<Todo[]>('get_todos', { includeDeleted });
      todos.value = result;
    } catch (e) {
      error.value = e as string;
      console.error('Failed to fetch todos:', e);
    } finally {
      loading.value = false;
    }
  }

  // 添加待办
  async function addTodo(input: AddTodoInput) {
    loading.value = true;
    error.value = null;
    try {
      const newTodo = await invoke<Todo>('add_todo', { input });
      todos.value.unshift(newTodo);
    } catch (e) {
      error.value = e as string;
      console.error('Failed to add todo:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  // 更新待办
  async function updateTodo(input: UpdateTodoInput) {
    loading.value = true;
    error.value = null;
    try {
      await invoke('update_todo', { input });
      await fetchTodos();
    } catch (e) {
      error.value = e as string;
      console.error('Failed to update todo:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  // 删除待办（软删除）
  async function deleteTodo(id: number) {
    loading.value = true;
    error.value = null;
    try {
      await invoke('delete_todo', { id });
      todos.value = todos.value.filter(todo => todo.id !== id);
      // 发送事件通知其他窗口
      await emit('todo-deleted', { id });
    } catch (e) {
      error.value = e as string;
      console.error('Failed to delete todo:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  // 切换完成状态
  async function toggleTodo(id: number) {
    loading.value = true;
    error.value = null;
    try {
      await invoke('toggle_todo', { id });
      await fetchTodos();
    } catch (e) {
      error.value = e as string;
      console.error('Failed to toggle todo:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  // 永久删除待办（物理删除）
  async function permanentDeleteTodo(id: number) {
    loading.value = true;
    error.value = null;
    try {
      await invoke('permanent_delete_todo', { id });
      todos.value = todos.value.filter(todo => todo.id !== id);
      // 发送事件通知其他窗口
      await emit('todo-permanent-deleted', { id });
    } catch (e) {
      error.value = e as string;
      console.error('Failed to permanently delete todo:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  // 恢复待办（从回收站恢复）
  async function restoreTodo(id: number) {
    loading.value = true;
    error.value = null;
    try {
      await invoke('restore_todo', { id });
      await fetchTodos(true); // 重新加载回收站数据
      // 发送事件通知其他窗口
      await emit('todo-restored', { id });
    } catch (e) {
      error.value = e as string;
      console.error('Failed to restore todo:', e);
      throw e;
    } finally {
      loading.value = false;
    }
  }

  return {
    todos,
    loading,
    error,
    fetchTodos,
    addTodo,
    updateTodo,
    deleteTodo,
    toggleTodo,
    permanentDeleteTodo,
    restoreTodo,
  };
});
