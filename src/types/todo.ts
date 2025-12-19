// 图片数据结构
export interface TodoImage {
  id?: number;
  todoId: number;
  imagePath: string;
  orderIndex: number;
  createdAt: string;
}

// Todo 数据结构
export interface Todo {
  id?: number;
  content: string;
  source?: string;
  imagePath?: string;  // 保留用于向后兼容
  ocrText?: string;
  createdAt: string;
  expectedFinishAt?: string;
  finishedAt?: string;
  isCompleted: boolean;
  orderIndex?: number;
  images?: TodoImage[];  // 新增：图片数组
}

// 添加待办的输入参数
export interface AddTodoInput {
  content: string;
  source?: string;
  imagePaths?: string[];  // 改为数组
  ocrText?: string;
  expectedFinishAt?: string;
}

// 更新待办的输入参数
export interface UpdateTodoInput {
  id: number;
  content?: string;
  expectedFinishAt?: string;
}
