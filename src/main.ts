import { createApp } from "vue";
import { createPinia } from "pinia";
import { getCurrentWindow } from "@tauri-apps/api/window";
import App from "./App.vue";
import SnapPreview from "./components/SnapPreview.vue";

const currentWindow = getCurrentWindow();
const windowLabel = currentWindow.label;

// 根据窗口 label 决定渲染哪个组件
if (windowLabel === "preview") {
  // 预览窗口：只渲染预览组件
  const app = createApp(SnapPreview);
  app.mount("#app");
} else {
  // 主窗口：渲染主应用
  const app = createApp(App);
  const pinia = createPinia();
  app.use(pinia);
  app.mount("#app");
}

