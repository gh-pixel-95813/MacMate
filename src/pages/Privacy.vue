<script setup lang="ts">
import { computed, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import type { BrowserStatus, CleanOutcome, PrivacyTarget } from '@/types';
import ConfirmDialog from '@/components/ConfirmDialog.vue';

const { t } = useI18n();

// State
const targets = ref<PrivacyTarget[]>([]);
const browserStatuses = ref<BrowserStatus[]>([]);
const selectedPaths = ref<Set<string>>(new Set());
const scanning = ref(false);
const cleaning = ref(false);
// ConfirmDialog 显隐:清理前二次确认(替换原 inline 两步确认)
const confirmVisible = ref(false);
const hasScanned = ref(false);
const outcome = ref<CleanOutcome | null>(null);
const errorMsg = ref('');

// Browser display order
const browserOrder = ['Safari', 'Chrome', 'Firefox', 'System'] as const;

// Backend label → i18n key
const labelKeyMap: Record<string, string> = {
  History: 'privacy.history',
  Cookies: 'privacy.cookies',
  Cache: 'privacy.cache',
  'Downloads History': 'privacy.downloads',
  'Form Data': 'privacy.formData',
  'Recent Documents': 'privacy.recentDocuments',
};

const browserKeyMap: Record<string, string> = {
  Safari: 'privacy.safari',
  Chrome: 'privacy.chrome',
  Firefox: 'privacy.firefox',
  System: 'privacy.system',
};

function browserLabel(browser: string): string {
  return t(browserKeyMap[browser] ?? browser);
}

function itemLabel(label: string): string {
  return t(labelKeyMap[label] ?? label);
}

// Group targets by browser, maintaining display order
const grouped = computed(() => {
  const groups: { browser: string; items: PrivacyTarget[] }[] = [];
  for (const browser of browserOrder) {
    const items = targets.value.filter((item) => item.browser === browser);
    if (items.length > 0) {
      groups.push({ browser, items });
    }
  }
  return groups;
});

function isBrowserRunning(browser: string): boolean {
  return browserStatuses.value.find((s) => s.browser === browser)?.isRunning ?? false;
}

function isGroupInstalled(browser: string, items: PrivacyTarget[]): boolean {
  if (browser === 'System') return true;
  return items.some((item) => item.isInstalled);
}

function isItemDisabled(item: PrivacyTarget): boolean {
  return !item.isInstalled || isBrowserRunning(item.browser);
}

function isItemSelected(item: PrivacyTarget): boolean {
  return selectedPaths.value.has(item.path);
}

function toggleItem(item: PrivacyTarget) {
  if (isItemDisabled(item)) return;
  const next = new Set(selectedPaths.value);
  if (next.has(item.path)) {
    next.delete(item.path);
  } else {
    next.add(item.path);
  }
  selectedPaths.value = next;
}

const selectedItems = computed(() =>
  targets.value.filter((item) => selectedPaths.value.has(item.path)),
);

const selectedSize = computed(() =>
  selectedItems.value.reduce((sum, item) => sum + item.sizeBytes, 0),
);

const totalSize = computed(() => targets.value.reduce((sum, item) => sum + item.sizeBytes, 0));

function formatSize(bytes: number): string {
  if (bytes === 0) return '0 B';
  const KB = 1024;
  const MB = KB * 1024;
  const GB = MB * 1024;
  const TB = GB * 1024;
  if (bytes >= TB) return `${(bytes / TB).toFixed(2)} TB`;
  if (bytes >= GB) return `${(bytes / GB).toFixed(2)} GB`;
  if (bytes >= MB) return `${(bytes / MB).toFixed(0)} MB`;
  if (bytes >= KB) return `${(bytes / KB).toFixed(0)} KB`;
  return `${bytes} B`;
}

const safetyStyles: Record<string, string> = {
  safe: 'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400',
  caution: 'bg-yellow-100 text-yellow-700 dark:bg-yellow-900/30 dark:text-yellow-400',
  danger: 'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400',
};

// ConfirmDialog 内容(在 selectedItems / selectedSize / formatSize 之后定义以避免 use-before-define)
const confirmTitle = computed(() => t('privacy.confirmTitle'));
const confirmMessage = computed(() =>
  t('privacy.confirmClean', {
    count: selectedItems.value.length,
    size: formatSize(selectedSize.value),
  }),
);
const confirmDetails = computed(() =>
  selectedItems.value.slice(0, 10).map((i) => ({ path: i.path, size: i.sizeBytes })),
);

async function scan() {
  scanning.value = true;
  errorMsg.value = '';
  outcome.value = null;
  confirmVisible.value = false;
  try {
    const [scanResult, runningResult] = await Promise.all([
      invoke<PrivacyTarget[]>('scan_privacy'),
      invoke<BrowserStatus[]>('check_browsers_running'),
    ]);
    targets.value = scanResult;
    browserStatuses.value = runningResult;
    // Auto-select installed, non-running items
    selectedPaths.value = new Set(
      scanResult
        .filter((item) => item.isInstalled && !isBrowserRunning(item.browser))
        .map((item) => item.path),
    );
  } catch (e) {
    errorMsg.value = String(e);
    targets.value = [];
  } finally {
    scanning.value = false;
    hasScanned.value = true;
  }
}

// 点击清理按钮:打开 ConfirmDialog 二次确认(替换原 inline 两步确认)
function onCleanClick() {
  if (selectedItems.value.length === 0) return;
  confirmVisible.value = true;
}

async function clean() {
  confirmVisible.value = false;
  cleaning.value = true;
  errorMsg.value = '';
  outcome.value = null;
  try {
    const items = selectedItems.value.map((item) => item.path);
    const result = await invoke<CleanOutcome>('clean_privacy', { items });
    outcome.value = result;
    await scan();
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    cleaning.value = false;
  }
}

function isRunningFailure(reason: string): boolean {
  return reason.toLowerCase().includes('running');
}
</script>

<template>
  <div class="p-6">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
      {{ t('page.privacy') }}
    </h1>

    <!-- 顶部:扫描按钮 -->
    <div class="mt-4 flex items-center gap-3">
      <button
        class="rounded-lg bg-blue-500 px-4 py-2 text-sm font-medium text-white shadow-sm transition-colors hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="scanning || cleaning"
        @click="scan"
      >
        {{ scanning ? '...' : t('common.scan') }}
      </button>
      <span
        v-if="hasScanned && targets.length > 0"
        class="text-sm text-gray-500 dark:text-gray-400"
      >
        {{ t('common.total') }}: {{ formatSize(totalSize) }}
      </span>
    </div>

    <!-- 错误提示 -->
    <p
      v-if="errorMsg"
      class="mt-4 rounded-lg bg-red-50 p-3 text-sm text-red-600 dark:bg-red-900/20 dark:text-red-400"
    >
      {{ errorMsg }}
    </p>

    <!-- 空状态:未扫描 -->
    <div
      v-if="!scanning && !hasScanned && targets.length === 0"
      class="mt-6 flex flex-col items-center justify-center gap-3 rounded-2xl border border-dashed border-gray-300 bg-gray-50/60 p-12 text-center dark:border-zinc-700 dark:bg-zinc-900/40"
    >
      <svg
        viewBox="0 0 24 24"
        class="h-12 w-12 text-gray-400 dark:text-zinc-500"
        fill="none"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
      </svg>
      <p class="text-base font-medium text-gray-700 dark:text-gray-300">
        {{ t('common.scan') }}
      </p>
    </div>

    <!-- 空状态:扫描后无结果 -->
    <div
      v-else-if="!scanning && hasScanned && targets.length === 0"
      class="mt-6 rounded-2xl border border-gray-200/60 bg-white/70 p-8 text-center dark:border-zinc-800/60 dark:bg-zinc-900/70"
    >
      <p class="text-sm text-gray-500 dark:text-gray-400">No privacy data found.</p>
    </div>

    <!-- 扫描结果:按浏览器分组 -->
    <div v-else-if="targets.length > 0" class="mt-6 space-y-4">
      <div
        v-for="group in grouped"
        :key="group.browser"
        class="rounded-2xl border border-gray-200/60 bg-white/70 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/70"
        :class="{ 'opacity-50': !isGroupInstalled(group.browser, group.items) }"
      >
        <!-- 组头:浏览器名 + 安装状态 + 运行状态 -->
        <div
          class="flex items-center gap-2 border-b border-gray-100 px-4 py-3 dark:border-zinc-800"
        >
          <span class="text-sm font-semibold text-gray-900 dark:text-gray-100">
            {{ browserLabel(group.browser) }}
          </span>
          <span
            v-if="!isGroupInstalled(group.browser, group.items)"
            class="rounded-full bg-gray-200 px-2 py-0.5 text-xs text-gray-500 dark:bg-zinc-700 dark:text-gray-400"
          >
            {{ t('privacy.notInstalled') }}
          </span>
          <span
            v-if="isBrowserRunning(group.browser)"
            class="rounded-full bg-orange-100 px-2 py-0.5 text-xs text-orange-700 dark:bg-orange-900/30 dark:text-orange-400"
          >
            {{ t('privacy.running') }}
          </span>
        </div>

        <!-- 组内列表 -->
        <ul class="divide-y divide-gray-100 dark:divide-zinc-800">
          <li
            v-for="item in group.items"
            :key="item.id"
            class="flex items-center gap-3 px-4 py-2.5"
            :class="{ 'opacity-50': isItemDisabled(item) }"
          >
            <!-- 复选框 -->
            <input
              type="checkbox"
              class="h-4 w-4 rounded border-gray-300 text-blue-500 focus:ring-blue-500 disabled:cursor-not-allowed dark:border-zinc-600 dark:bg-zinc-800"
              :checked="isItemSelected(item)"
              :disabled="isItemDisabled(item)"
              @change="toggleItem(item)"
            />

            <!-- 标签 -->
            <div class="flex min-w-0 flex-1 items-center gap-2">
              <span class="text-sm font-medium text-gray-700 dark:text-gray-300">
                {{ browserLabel(item.browser) }} {{ itemLabel(item.label) }}
              </span>
              <!-- 安全等级徽章 -->
              <span
                class="rounded-full px-2 py-0.5 text-xs font-medium"
                :class="safetyStyles[item.safety]"
              >
                {{ item.safety }}
              </span>
              <!-- 运行中提示 -->
              <span
                v-if="isBrowserRunning(item.browser) && item.browser !== 'System'"
                class="text-xs text-orange-600 dark:text-orange-400"
              >
                {{ t('privacy.closeBrowser') }}
              </span>
            </div>

            <!-- 路径(截断) -->
            <span
              class="hidden max-w-[200px] truncate text-xs text-gray-400 dark:text-gray-500 md:block"
              :title="item.path"
            >
              {{ item.path }}
            </span>

            <!-- 大小 -->
            <span class="text-xs text-gray-500 dark:text-gray-400">
              {{ formatSize(item.sizeBytes) }}
            </span>
          </li>
        </ul>
      </div>
    </div>

    <!-- 底部:清理按钮 + 释放空间统计 -->
    <div
      v-if="targets.length > 0"
      class="sticky bottom-0 mt-6 flex items-center justify-between rounded-2xl border border-gray-200/60 bg-white/90 px-4 py-3 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/90"
    >
      <div class="text-sm text-gray-600 dark:text-gray-400">
        <span>{{ t('common.total') }}: </span>
        <span class="font-semibold text-gray-900 dark:text-gray-100">
          {{ formatSize(selectedSize) }}
        </span>
        <span class="ml-1 text-gray-400">/ {{ formatSize(totalSize) }}</span>
      </div>
      <button
        class="rounded-lg bg-blue-500 px-4 py-1.5 text-sm font-medium text-white shadow-sm transition-colors hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="cleaning || selectedItems.length === 0"
        @click="onCleanClick"
      >
        <span v-if="cleaning">...</span>
        <span v-else>{{ t('common.clean') }}</span>
      </button>
    </div>

    <!-- 二次确认弹窗:替换原 inline 两步确认 -->
    <ConfirmDialog
      :visible="confirmVisible"
      :title="confirmTitle"
      :message="confirmMessage"
      :detail-items="confirmDetails"
      @confirm="clean"
      @cancel="confirmVisible = false"
    />

    <!-- 清理报告 -->
    <div
      v-if="outcome"
      class="mt-6 rounded-2xl border border-gray-200/60 bg-white/70 p-4 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/70"
    >
      <h2 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
        {{ t('common.clean') }}
      </h2>

      <!-- 成功项 -->
      <div v-if="outcome.success.length > 0" class="mt-2">
        <p class="text-xs text-green-600 dark:text-green-400">
          {{ outcome.success.length }} cleaned
        </p>
        <ul class="mt-1 space-y-0.5">
          <li
            v-for="item in outcome.success"
            :key="item.id"
            class="text-xs text-gray-500 dark:text-gray-400"
          >
            {{ item.path }} ({{ formatSize(item.sizeBytes) }})
          </li>
        </ul>
      </div>

      <!-- 失败项 -->
      <div v-if="outcome.failed.length > 0" class="mt-3">
        <p class="text-xs text-red-600 dark:text-red-400">{{ outcome.failed.length }} failed</p>
        <ul class="mt-1 space-y-0.5">
          <li
            v-for="item in outcome.failed"
            :key="item.id"
            class="text-xs"
            :class="
              isRunningFailure(item.reason)
                ? 'text-orange-600 dark:text-orange-400'
                : 'text-red-500 dark:text-red-400'
            "
          >
            {{ item.path }}
            <span class="ml-1 text-gray-400">— {{ item.reason }}</span>
          </li>
        </ul>
      </div>
    </div>
  </div>
</template>
