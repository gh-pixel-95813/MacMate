<script setup lang="ts">
import { h, type FunctionalComponent } from 'vue';
import { useRoute } from 'vue-router';
import { useI18n } from 'vue-i18n';
import { useAppStore } from '@/stores/app';

interface NavItem {
  name: string;
  labelKey: string;
  path: string;
}

// 6 个导航项:i18n key 与路由一一对应
const navItems: NavItem[] = [
  { name: 'dashboard', labelKey: 'nav.dashboard', path: '/' },
  { name: 'systemJunk', labelKey: 'nav.systemJunk', path: '/system-junk' },
  { name: 'uninstaller', labelKey: 'nav.uninstaller', path: '/uninstaller' },
  { name: 'largeFiles', labelKey: 'nav.largeFiles', path: '/large-files' },
  { name: 'privacy', labelKey: 'nav.privacy', path: '/privacy' },
  { name: 'settings', labelKey: 'nav.settings', path: '/settings' },
];

const route = useRoute();
const { t } = useI18n();
const appStore = useAppStore();

// 选中态:'/' 精确匹配,其他项支持子路由前缀
function isActive(path: string): boolean {
  if (path === '/') return route.path === '/';
  return route.path === path || route.path.startsWith(path + '/');
}

// 内联 SVG 图标(lucide-vue-next 未安装,用 functional 组件 + h 渲染)
interface NavIconProps {
  name: string;
}

const NavIcon: FunctionalComponent<NavIconProps> = (props) => {
  const svgAttrs = {
    viewBox: '0 0 24 24',
    fill: 'none',
    stroke: 'currentColor',
    'stroke-width': 1.8,
    'stroke-linecap': 'round',
    'stroke-linejoin': 'round',
  };
  switch (props.name) {
    case 'dashboard':
      return h('svg', svgAttrs, [
        h('rect', { x: 3, y: 3, width: 7, height: 7, rx: 1 }),
        h('rect', { x: 14, y: 3, width: 7, height: 7, rx: 1 }),
        h('rect', { x: 3, y: 14, width: 7, height: 7, rx: 1 }),
        h('rect', { x: 14, y: 14, width: 7, height: 7, rx: 1 }),
      ]);
    case 'systemJunk':
      return h('svg', svgAttrs, [
        h('path', {
          d: 'M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6',
        }),
        h('path', { d: 'M10 11v6M14 11v6' }),
      ]);
    case 'uninstaller':
      return h('svg', svgAttrs, [
        h('path', {
          d: 'M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z',
        }),
        h('path', { d: 'M3.27 6.96 12 12l8.73-5.04M12 22V12' }),
      ]);
    case 'largeFiles':
      return h('svg', svgAttrs, [
        h('path', { d: 'M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z' }),
        h('path', { d: 'M14 2v6h6' }),
      ]);
    case 'privacy':
      return h('svg', svgAttrs, [h('path', { d: 'M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z' })]);
    case 'settings':
      return h('svg', svgAttrs, [
        h('circle', { cx: 12, cy: 12, r: 3 }),
        h('path', {
          d: 'M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-4 0v-.09a1.65 1.65 0 0 0-1-1.51 1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1 0-4h.09a1.65 1.65 0 0 0 1.51-1 1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06a1.65 1.65 0 0 0 1.82.33h0a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82v0a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z',
        }),
      ]);
    default:
      return h('svg', svgAttrs, []);
  }
};
// 显式声明 props,使 Vue 在运行时将 `name` 作为 prop 传入(否则会进入 fallthrough attrs)
NavIcon.props = ['name'];
</script>

<template>
  <aside
    class="flex h-full w-[220px] shrink-0 flex-col border-r border-gray-200/60 bg-white/70 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/70"
  >
    <!-- 顶部 logo + 应用名 -->
    <div class="flex items-center gap-2 px-5 py-4">
      <span
        class="flex h-8 w-8 items-center justify-center rounded-lg bg-blue-500 text-white shadow-sm"
      >
        <svg
          viewBox="0 0 24 24"
          class="h-5 w-5"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z" />
        </svg>
      </span>
      <span class="text-base font-semibold text-gray-900 dark:text-gray-100">MacMate</span>
    </div>

    <!-- 导航项 -->
    <nav class="mt-2 flex flex-1 flex-col gap-1 px-3">
      <RouterLink
        v-for="item in navItems"
        :key="item.path"
        :to="item.path"
        class="flex items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium transition-colors"
        :class="
          isActive(item.path)
            ? 'bg-blue-500/10 text-blue-600 dark:text-blue-400'
            : 'text-gray-600 hover:bg-gray-500/5 hover:text-gray-900 dark:text-gray-400 dark:hover:bg-zinc-50/5 dark:hover:text-gray-100'
        "
      >
        <NavIcon :name="item.name" class="h-5 w-5 shrink-0" />
        <span>{{ t(item.labelKey) }}</span>
      </RouterLink>
    </nav>

    <!-- 底部版本号:从 store 读动态版本,App.vue 启动时 hydrate -->
    <div class="px-5 py-3 text-xs text-gray-400 dark:text-gray-600">v{{ appStore.appVersion }}</div>
  </aside>
</template>
