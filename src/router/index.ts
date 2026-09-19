import { createRouter, createWebHistory } from 'vue-router';
import type { RouteRecordRaw } from 'vue-router';

// 路由表:6 个导航项,懒加载占位页面,meta.title 用于 AppHeader 显示
const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'dashboard',
    component: () => import('@/pages/Dashboard.vue'),
    meta: { title: 'nav.dashboard' },
  },
  {
    path: '/system-junk',
    name: 'systemJunk',
    component: () => import('@/pages/SystemJunk.vue'),
    meta: { title: 'nav.systemJunk' },
  },
  {
    path: '/uninstaller',
    name: 'uninstaller',
    component: () => import('@/pages/Uninstaller.vue'),
    meta: { title: 'nav.uninstaller' },
  },
  {
    path: '/large-files',
    name: 'largeFiles',
    component: () => import('@/pages/LargeFiles.vue'),
    meta: { title: 'nav.largeFiles' },
  },
  {
    path: '/privacy',
    name: 'privacy',
    component: () => import('@/pages/Privacy.vue'),
    meta: { title: 'nav.privacy' },
  },
  {
    path: '/settings',
    name: 'settings',
    component: () => import('@/pages/Settings.vue'),
    meta: { title: 'nav.settings' },
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
