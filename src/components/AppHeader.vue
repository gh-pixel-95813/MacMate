<script setup lang="ts">
import { computed } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { i18n } from '@/i18n';
import { useAppStore } from '@/stores/app';
import { useTheme } from '@/composables/useTheme';

const route = useRoute();
const { t } = useI18n();
const store = useAppStore();
const { cycleTheme } = useTheme();

// 当前页面标题:从路由 meta.title 取 i18n key,缺省回退到 nav.dashboard
const pageTitle = computed(() => {
  const key = route.meta?.title;
  if (typeof key === 'string' && key.length > 0) return t(key);
  return t('nav.dashboard');
});

// 当前主题对应的 i18n 文案
const themeLabel = computed(() => {
  if (store.theme === 'light') return t('theme.light');
  if (store.theme === 'dark') return t('theme.dark');
  return t('theme.system');
});

// 语言切换按钮:显示切换目标的语言缩写(当前为中文则显示 EN)
const localeButtonLabel = computed(() => (i18n.global.locale.value === 'zh-CN' ? 'EN' : '中'));

function toggleLocale() {
  const next = i18n.global.locale.value === 'zh-CN' ? 'en-US' : 'zh-CN';
  i18n.global.locale.value = next;
  store.setLocale(next);
}
</script>

<template>
  <header
    class="flex h-12 shrink-0 items-center justify-between border-b border-gray-200/60 bg-white/60 px-6 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/60"
  >
    <!-- 左侧:当前页面标题(从路由 meta.title 取 i18n key) -->
    <h1 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
      {{ pageTitle }}
    </h1>

    <!-- 右侧:主题切换 + 语言切换 -->
    <div class="flex items-center gap-2">
      <button
        type="button"
        class="flex h-8 w-8 items-center justify-center rounded-md text-gray-600 transition-colors hover:bg-gray-500/10 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-zinc-50/10 dark:hover:text-gray-100"
        :title="themeLabel"
        :aria-label="themeLabel"
        @click="cycleTheme"
      >
        <!-- system:显示器图标 -->
        <svg
          v-if="store.theme === 'system'"
          viewBox="0 0 24 24"
          class="h-4 w-4"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="2" y="3" width="20" height="14" rx="2" />
          <path d="M8 21h8M12 17v4" />
        </svg>
        <!-- light:太阳图标 -->
        <svg
          v-else-if="store.theme === 'light'"
          viewBox="0 0 24 24"
          class="h-4 w-4"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="12" r="4" />
          <path
            d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"
          />
        </svg>
        <!-- dark:月亮图标 -->
        <svg
          v-else
          viewBox="0 0 24 24"
          class="h-4 w-4"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
        </svg>
      </button>

      <button
        type="button"
        class="flex h-8 min-w-[2rem] items-center justify-center rounded-md px-2 text-xs font-semibold text-gray-600 transition-colors hover:bg-gray-500/10 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-zinc-50/10 dark:hover:text-gray-100"
        :aria-label="localeButtonLabel"
        @click="toggleLocale"
      >
        {{ localeButtonLabel }}
      </button>
    </div>
  </header>
</template>
