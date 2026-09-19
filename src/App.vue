<script setup lang="ts">
// MacMate 根组件:左侧 Sidebar + 主内容区布局
import { onMounted } from 'vue';
import AppSidebar from '@/components/AppSidebar.vue';
import AppHeader from '@/components/AppHeader.vue';
import { i18n } from '@/i18n';
import { useAppStore } from '@/stores/app';
import { useTheme } from '@/composables/useTheme';

const store = useAppStore();
const theme = useTheme();

// Task 9.3:启动时从 `~/.macmate/config.json` 加载持久化配置(locale/theme/
// deepClean/largeFileThresholdMb)到 store,同步 i18n 与主题 class。
// 非 Tauri 环境(浏览器 dev)调用失败时静默回退到默认值。
onMounted(async () => {
  await store.hydrateFromConfig();
  i18n.global.locale.value = store.locale;
  theme.apply();
});
</script>

<template>
  <div class="flex h-screen w-screen overflow-hidden bg-gray-50 dark:bg-zinc-950">
    <AppSidebar />
    <main class="flex flex-1 flex-col overflow-hidden">
      <AppHeader />
      <div class="flex-1 overflow-auto">
        <RouterView />
      </div>
    </main>
  </div>
</template>

<style scoped>
/* 全局样式见 src/style.css */
</style>
