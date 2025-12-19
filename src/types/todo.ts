// Todo 数据结构
export interface Todo {
  id?: number;
  content: string;
  source?: string;
  imagePath?: string;
  ocrText?: string;
  createdAt: string;
  expectedFinishAt?: string;
  finishedAt?: string;
  isCompleted: boolean;
  orderIndex?: number;
}

// 添加待办的输入参数
export interface AddTodoInput {
  content: string;
  source?: string;
  imagePath?: string;
  ocrText?: string;
  expectedFinishAt?: string;
}

// 更新待办的输入参数
export interface UpdateTodoInput {
  id: number;
  content?: string;
  expectedFinishAt?: string;
}
