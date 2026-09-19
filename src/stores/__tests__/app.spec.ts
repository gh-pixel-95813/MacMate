import { beforeEach, describe, expect, it } from 'vitest';
import { createPinia, setActivePinia } from 'pinia';
import { useAppStore } from '@/stores/app';

describe('useAppStore', () => {
  beforeEach(() => {
    // 每个用例使用独立的 Pinia 实例,避免 store 状态在用例间泄漏。
    setActivePinia(createPinia());
  });

  it('initial theme is "system"', () => {
    const store = useAppStore();
    expect(store.theme).toBe('system');
  });

  it('setTheme("dark") sets theme to "dark"', () => {
    const store = useAppStore();
    store.setTheme('dark');
    expect(store.theme).toBe('dark');
  });

  it('setTheme("light") makes isDark false', () => {
    const store = useAppStore();
    store.setTheme('light');
    expect(store.isDark).toBe(false);
  });

  it('setDeepClean(true) enables deepClean', () => {
    const store = useAppStore();
    store.setDeepClean(true);
    expect(store.deepClean).toBe(true);
  });

  it('setLocale("en-US") sets locale to "en-US"', () => {
    const store = useAppStore();
    store.setLocale('en-US');
    expect(store.locale).toBe('en-US');
  });
});
