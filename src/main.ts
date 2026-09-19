import { createApp } from 'vue';
import { createPinia, setActivePinia } from 'pinia';
import App from './App.vue';
import router from './router';
import { i18n } from './i18n';
import { useTheme } from './composables/useTheme';
import './style.css';

// Vue 应用入口:注册 Pinia / Router / i18n
const app = createApp(App);
const pinia = createPinia();
app.use(pinia);
app.use(router);
app.use(i18n);

// 在挂载前应用主题(避免暗色模式闪烁),useTheme 内部使用 store,需激活 pinia
setActivePinia(pinia);
useTheme().apply();

app.mount('#app');
