import { defineStore } from 'pinia';
import { computed, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig } from '@/types';

// 主题模式:system 跟随系统,light 浅色,dark 暗色
export type Theme = 'system' | 'light' | 'dark';

// 应用全局状态 store,记录扫描状态、语言、主题与系统主题偏好。
// Task 9.3:locale / theme / deepClean / largeFileThresholdMb 通过
// `~/.macmate/config.json` 持久化(setter 自动落盘,启动时由 hydrateFromConfig 加载)。
export const useAppStore = defineStore('app', () => {
  const isScanning = ref(false);
  const locale = ref<'zh-CN' | 'en-US'>('zh-CN');
  const theme = ref<Theme>('system');
  // 系统主题偏好(由 useTheme 监听 matchMedia 同步更新),用于 isDark 计算
  const systemDark = ref(false);
  // 深度清理开关:开启后 SystemJunk 扫描会标注系统级目录为 danger,
  // 清理时走 deep_clean_system(sudo)。默认关闭,需用户在 Settings 中显式开启。
  const deepClean = ref(false);
  // 大文件扫描阈值(MB),低于此大小的文件不进入大文件列表。默认 50。
  const largeFileThresholdMb = ref(50);

  // 启动时 hydrate 期间置为 true,抑制 setter 触发不必要的回写。
  let hydrating = false;

  // 将当前 store 中的可持久化字段写入 `~/.macmate/config.json`。
  // 失败时静默(例如运行在非 Tauri 环境的浏览器 dev 模式),避免阻塞 UI。
  function persistConfig(): void {
    if (hydrating) return;
    const config: AppConfig = {
      locale: locale.value,
      theme: theme.value,
      deepClean: deepClean.value,
      largeFileThresholdMb: largeFileThresholdMb.value,
    };
    invoke('save_config', { config }).catch(() => {
      // 非 Tauri 环境(浏览器 dev / 测试)invoke 会 reject,忽略即可。
    });
  }

  function setLocale(value: 'zh-CN' | 'en-US') {
    locale.value = value;
    persistConfig();
  }

  function setTheme(value: Theme) {
    theme.value = value;
    persistConfig();
  }

  function setSystemDark(value: boolean) {
    systemDark.value = value;
  }

  function setDeepClean(value: boolean) {
    deepClean.value = value;
    persistConfig();
  }

  function setLargeFileThresholdMb(value: number) {
    largeFileThresholdMb.value = value;
    persistConfig();
  }

  // 从后端加载配置(app.vue onMounted 调用),直接写 ref 不触发回写。
  // 调用方负责同步 i18n.global.locale.value 与 useTheme().apply()。
  async function hydrateFromConfig(): Promise<void> {
    hydrating = true;
    try {
      const cfg = await invoke<AppConfig>('get_config');
      if (cfg.locale === 'zh-CN' || cfg.locale === 'en-US') {
        locale.value = cfg.locale;
      }
      if (cfg.theme === 'system' || cfg.theme === 'light' || cfg.theme === 'dark') {
        theme.value = cfg.theme;
      }
      deepClean.value = !!cfg.deepClean;
      if (typeof cfg.largeFileThresholdMb === 'number' && cfg.largeFileThresholdMb > 0) {
        largeFileThresholdMb.value = cfg.largeFileThresholdMb;
      }
    } catch {
      // 非 Tauri 环境或命令未注册,保持默认值。
    } finally {
      hydrating = false;
    }
  }

  // 当前是否为暗色:light → false,dark → true,system → 系统偏好
  const isDark = computed(() => {
    if (theme.value === 'light') return false;
    if (theme.value === 'dark') return true;
    return systemDark.value;
  });

  return {
    isScanning,
    locale,
    theme,
    systemDark,
    deepClean,
    largeFileThresholdMb,
    isDark,
    setLocale,
    setTheme,
    setSystemDark,
    setDeepClean,
    setLargeFileThresholdMb,
    hydrateFromConfig,
  };
});
