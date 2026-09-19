/// <reference types="vite/client" />

// 使本文件成为 ES 模块,从而让下方的 `declare module 'vue-router'` 作为
// "模块增强"(augmentation)而非"模块声明"(覆盖原有类型)
export {};

declare module '*.vue' {
  import type { DefineComponent } from 'vue';
  const component: DefineComponent<Record<string, never>, Record<string, never>, unknown>;
  export default component;
}

// 路由 meta 类型增强:声明 title 字段用于 AppHeader 显示页面标题
declare module 'vue-router' {
  interface RouteMeta {
    title?: string;
  }
}
