<script setup lang="ts">
// 仪表盘(Task 9.1):磁盘使用概览 + 模块快捷入口 + 最近清理历史。
// - 顶部:大号磁盘使用环形图(DiskPieChart)
// - 中部:4 个模块快捷入口卡片(系统垃圾/应用卸载/大文件/隐私),点击跳转
// - 底部:最近 5 条清理历史 + "清空历史"按钮
import { computed, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useRouter } from 'vue-router';
import { useI18n } from 'vue-i18n';
import DiskPieChart from '@/components/DiskPieChart.vue';
import { formatSize } from '@/utils/format';
import type { CleanHistoryEntry } from '@/types';

const { t } = useI18n();
const router = useRouter();

// 模块名 → 中文名映射(后端 module 字段值 → 中文展示)。
const moduleNames: Record<string, string> = {
  system_junk: '系统垃圾',
  uninstaller: '应用卸载',
  large_files: '大文件扫描',
  privacy: '隐私清理',
};

// 4 个快捷入口卡片:图标 + 标题 + 描述 + 跳转路由。
interface QuickAction {
  key: string;
  titleKey: string;
  descKey: string;
  route: string;
  icon: 'trash' | 'box' | 'file' | 'shield';
}

const quickActions: QuickAction[] = [
  {
    key: 'systemJunk',
    titleKey: 'nav.systemJunk',
    descKey: 'dashboard.moduleSystemJunk',
    route: '/system-junk',
    icon: 'trash',
  },
  {
    key: 'uninstaller',
    titleKey: 'nav.uninstaller',
    descKey: 'dashboard.moduleUninstaller',
    route: '/uninstaller',
    icon: 'box',
  },
  {
    key: 'largeFiles',
    titleKey: 'nav.largeFiles',
    descKey: 'dashboard.moduleLargeFiles',
    route: '/large-files',
    icon: 'file',
  },
  {
    key: 'privacy',
    titleKey: 'nav.privacy',
    descKey: 'dashboard.modulePrivacy',
    route: '/privacy',
    icon: 'shield',
  },
];

function goTo(route: string) {
  router.push(route);
}

// 最近清理历史:启动时加载,显示最近 5 条,可清空。
const history = ref<CleanHistoryEntry[]>([]);
const historyLoading = ref(true);
const clearing = ref(false);

const recentHistory = computed(() => history.value.slice(-5).reverse());

async function loadHistory() {
  historyLoading.value = true;
  try {
    history.value = await invoke<CleanHistoryEntry[]>('get_clean_history');
  } catch {
    // 非 Tauri 环境或命令未注册,显示空状态。
    history.value = [];
  } finally {
    historyLoading.value = false;
  }
}

async function onClearHistory() {
  clearing.value = true;
  try {
    await invoke('clear_clean_history');
    await loadHistory();
  } catch {
    // 静默失败,避免阻塞 UI。
  } finally {
    clearing.value = false;
  }
}

// 时间戳(unix 秒)→ 本地可读字符串。
function formatTime(ts: number): string {
  return new Date(ts * 1000).toLocaleString();
}

// 模块名展示:未知模块回退原值。
function moduleLabel(module: string): string {
  return moduleNames[module] ?? module;
}

onMounted(loadHistory);
</script>

<template>
  <div class="p-6">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
      {{ t('app.name') }}
    </h1>
    <p class="mt-2 text-gray-600 dark:text-gray-400">{{ t('app.tagline') }}</p>

    <!-- 顶部:大号磁盘使用环形图 -->
    <div class="mt-6">
      <DiskPieChart />
    </div>

    <!-- 中部:4 个模块快捷入口卡片 -->
    <section class="mt-8">
      <h2 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
        {{ t('dashboard.quickActions') }}
      </h2>
      <div class="mt-3 grid grid-cols-2 gap-3 lg:grid-cols-4">
        <button
          v-for="action in quickActions"
          :key="action.key"
          type="button"
          class="flex flex-col items-start gap-2 rounded-2xl border border-gray-200/60 bg-white/70 p-4 text-left transition-colors hover:border-blue-300 hover:bg-blue-50/40 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-zinc-800/60 dark:bg-zinc-900/70 dark:hover:border-blue-700 dark:hover:bg-blue-950/30"
          @click="goTo(action.route)"
        >
          <span
            class="flex h-9 w-9 items-center justify-center rounded-lg bg-blue-500/10 text-blue-600 dark:text-blue-400"
          >
            <!-- trash:系统垃圾 -->
            <svg
              v-if="action.icon === 'trash'"
              viewBox="0 0 24 24"
              class="h-5 w-5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path
                d="M3 6h18M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6"
              />
              <path d="M10 11v6M14 11v6" />
            </svg>
            <!-- box:应用卸载 -->
            <svg
              v-else-if="action.icon === 'box'"
              viewBox="0 0 24 24"
              class="h-5 w-5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path
                d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"
              />
              <path d="M3.27 6.96 12 12l8.73-5.04M12 22V12" />
            </svg>
            <!-- file:大文件 -->
            <svg
              v-else-if="action.icon === 'file'"
              viewBox="0 0 24 24"
              class="h-5 w-5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z" />
              <path d="M14 2v6h6" />
            </svg>
            <!-- shield:隐私 -->
            <svg
              v-else
              viewBox="0 0 24 24"
              class="h-5 w-5"
              fill="none"
              stroke="currentColor"
              stroke-width="1.8"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            </svg>
          </span>
          <span class="text-sm font-semibold text-gray-900 dark:text-gray-100">
            {{ t(action.titleKey) }}
          </span>
          <span class="text-xs text-gray-500 dark:text-zinc-400">
            {{ t(action.descKey) }}
          </span>
        </button>
      </div>
    </section>

    <!-- 底部:最近清理历史 -->
    <section class="mt-8">
      <div class="flex items-center justify-between">
        <h2 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
          {{ t('dashboard.recentHistory') }}
        </h2>
        <button
          v-if="recentHistory.length > 0"
          type="button"
          :disabled="clearing"
          class="rounded-md border border-gray-300 px-3 py-1 text-xs font-medium text-gray-600 transition-colors hover:bg-gray-100 disabled:opacity-50 dark:border-zinc-700 dark:text-gray-300 dark:hover:bg-zinc-800"
          @click="onClearHistory"
        >
          {{ clearing ? '…' : t('dashboard.clearHistory') }}
        </button>
      </div>

      <div
        class="mt-3 rounded-2xl border border-gray-200/60 bg-white/70 p-4 dark:border-zinc-800/60 dark:bg-zinc-900/70"
      >
        <div v-if="historyLoading" class="py-4 text-center text-sm text-gray-400">…</div>
        <ul
          v-else-if="recentHistory.length > 0"
          class="divide-y divide-gray-100 dark:divide-zinc-800"
        >
          <li
            v-for="(entry, idx) in recentHistory"
            :key="idx"
            class="flex items-center justify-between py-2 text-sm"
          >
            <div class="flex flex-col">
              <span class="font-medium text-gray-900 dark:text-gray-100">
                {{ moduleLabel(entry.module) }}
              </span>
              <span class="text-xs text-gray-500 dark:text-gray-400">
                {{ formatTime(entry.timestamp) }}
              </span>
            </div>
            <span class="font-semibold text-blue-600 dark:text-blue-400">
              {{ t('history.freed', { size: formatSize(entry.freedBytes) }) }}
            </span>
          </li>
        </ul>
        <div v-else class="py-6 text-center text-sm text-gray-400 dark:text-gray-500">
          {{ t('dashboard.noHistory') }}
        </div>
      </div>
    </section>
  </div>
</template>
