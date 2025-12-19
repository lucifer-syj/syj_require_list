import { defineStore } from 'pinia';
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Todo, AddTodoInput, UpdateTodoInput } from '../types/todo';

export const useTodoStore = defineStore('todo', () => {
  // 状态
  const todos = ref<Todo[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  // 获取所有待办
  async function fetchTodos() {
    loading.value = true;
    error.value = null;
    try {
      const result = await invoke<Todo[]>('get_todos');
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

  // 删除待办
  async function deleteTodo(id: number) {
    loading.value = true;
    error.value = null;
    try {
      await invoke('delete_todo', { id });
      todos.value = todos.value.filter(todo => todo.id !== id);
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

  return {
    todos,
    loading,
    error,
    fetchTodos,
    addTodo,
    updateTodo,
    deleteTodo,
    toggleTodo,
  };
});
