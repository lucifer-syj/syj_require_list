import { createApp } from 'vue';
import { createPinia } from 'pinia';
import AllTodos from './components/AllTodos.vue';

const pinia = createPinia();
const app = createApp(AllTodos);

app.use(pinia);
app.mount('#app');
