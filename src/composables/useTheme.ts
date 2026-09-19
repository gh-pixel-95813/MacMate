import { watch } from 'vue';
import { useAppStore } from '@/stores/app';

const SYSTEM_DARK_MQ = '(prefers-color-scheme: dark)';

// 读取系统主题偏好,SSR 或无 matchMedia 时返回 false
function readSystemDark(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') return false;
  return window.matchMedia(SYSTEM_DARK_MQ).matches;
}

// 在 document.documentElement 上 add/remove class "dark"(Tailwind class 策略)
function applyDarkClass(isDark: boolean): void {
  if (typeof document === 'undefined') return;
  document.documentElement.classList.toggle('dark', isDark);
}

// 模块级单例标记,确保系统主题监听与 store watch 只注册一次
let initialized = false;

/**
 * 主题组合式函数:负责监听系统主题(matchMedia)与 store.theme 变化,
 * 在 documentElement 上同步 add/remove `dark` class。
 *
 * main.ts 启动时调用 useTheme().apply() 初始化;
 * AppHeader 等组件可通过 useTheme() 拿到 theme / isDark / cycleTheme。
 */
export function useTheme() {
  const store = useAppStore();

  function apply(): void {
    if (!initialized) {
      initialized = true;
      // 同步当前系统主题到 store
      store.setSystemDark(readSystemDark());
      // 监听系统主题变化,主题为 system 时同步到 store(进而触发 isDark 重算)
      if (typeof window !== 'undefined' && typeof window.matchMedia === 'function') {
        const media = window.matchMedia(SYSTEM_DARK_MQ);
        media.addEventListener('change', (e: MediaQueryListEvent) => {
          store.setSystemDark(e.matches);
        });
      }
      // 监听 store 的 isDark 变化,自动应用 dark class
      watch(
        () => store.isDark,
        (val) => applyDarkClass(val),
      );
    }
    applyDarkClass(store.isDark);
  }

  function setTheme(t: 'system' | 'light' | 'dark'): void {
    store.setTheme(t);
  }

  // 主题切换循环:system → light → dark → system
  function cycleTheme(): void {
    const order: Array<'system' | 'light' | 'dark'> = ['system', 'light', 'dark'];
    const idx = order.indexOf(store.theme);
    const next = order[(idx + 1) % order.length];
    store.setTheme(next ?? 'system');
  }

  return {
    theme: store.theme,
    isDark: store.isDark,
    apply,
    setTheme,
    cycleTheme,
  };
}
