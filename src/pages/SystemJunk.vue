<script setup lang="ts">
import { computed, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import type { CleanOutcome, ScanItem, ScanResult, SafetyLevel } from '@/types';
import { formatSize } from '@/utils/format';
import { useAppStore } from '@/stores/app';
import ConfirmDialog from '@/components/ConfirmDialog.vue';

const { t } = useI18n();
const appStore = useAppStore();

const loading = ref(false);
const cleaning = ref(false);
const result = ref<ScanResult | null>(null);
const cleanOutcome = ref<CleanOutcome | null>(null);
const errorMsg = ref<string | null>(null);
const selected = ref<Set<string>>(new Set());
// ConfirmDialog 显隐:清理前二次确认
const confirmVisible = ref(false);

const items = computed<ScanItem[]>(() => result.value?.items ?? []);
const hasItems = computed(() => items.value.length > 0);
const selectedItems = computed(() => items.value.filter((i) => selected.value.has(i.id)));
const selectedCount = computed(() => selectedItems.value.length);
const selectedSize = computed(() => selectedItems.value.reduce((sum, i) => sum + i.sizeBytes, 0));
const freedSize = computed(() =>
  cleanOutcome.value ? cleanOutcome.value.success.reduce((sum, i) => sum + i.sizeBytes, 0) : 0,
);
const cleanDisabled = computed(
  () => !hasItems.value || cleaning.value || selectedCount.value === 0,
);

// ConfirmDialog 内容(在 selectedCount / selectedSize / selectedItems 之后定义以避免 use-before-define)
const confirmTitle = computed(() => t('systemJunk.confirmTitle'));
const confirmMessage = computed(() =>
  t('systemJunk.confirmClean', {
    count: selectedCount.value,
    size: formatSize(selectedSize.value),
  }),
);
// 详情项:全部选中项(由 ConfirmDialog 截到 10 并显示"...等 N 项"溢出提示)
const confirmDetails = computed(() =>
  selectedItems.value.map((i) => ({ path: i.path, size: i.sizeBytes })),
);

// Smart Selection:扫描后默认勾选所有 safety === 'safe' 的项,caution / danger 不勾选。
function smartSelect(scanItems: ScanItem[]): Set<string> {
  return new Set(scanItems.filter((i) => i.safety === 'safe').map((i) => i.id));
}

async function onScan() {
  loading.value = true;
  cleaning.value = false;
  errorMsg.value = null;
  cleanOutcome.value = null;
  try {
    const r = await invoke<ScanResult>('scan_system_junk');
    result.value = r;
    selected.value = smartSelect(r.items);
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    loading.value = false;
  }
}

// 点击清理按钮:打开 ConfirmDialog 二次确认(替换原 window.confirm)
function onClean() {
  if (selectedCount.value === 0) return;
  confirmVisible.value = true;
}

// 确认回调:执行实际清理 + 写入清理历史
async function onCleanConfirm() {
  confirmVisible.value = false;
  cleaning.value = true;
  errorMsg.value = null;
  cleanOutcome.value = null;
  try {
    const ids = selectedItems.value.map((i) => i.id);
    const outcome = await invoke<CleanOutcome>('clean_system_junk', { items: ids });
    cleanOutcome.value = outcome;
    // 写入清理历史(简化:仅 SystemJunk 接入,其他模块可后续补)
    const freed = outcome.success.reduce((sum, i) => sum + i.sizeBytes, 0);
    try {
      await invoke('add_clean_history', {
        entry: {
          timestamp: Math.floor(Date.now() / 1000),
          module: 'system_junk',
          freedBytes: freed,
          successCount: outcome.success.length,
          failedCount: outcome.failed.length,
        },
      });
    } catch (historyErr) {
      // 历史写入失败不阻塞主流程,记录到控制台
      console.warn('add_clean_history failed:', historyErr);
    }
    // 清理完成后重新扫描以刷新列表
    await onScan();
  } catch (e) {
    errorMsg.value = String(e);
  } finally {
    cleaning.value = false;
  }
}

function toggle(id: string) {
  const next = new Set(selected.value);
  if (next.has(id)) {
    next.delete(id);
  } else {
    next.add(id);
  }
  selected.value = next;
}

function formatDate(unix: number): string {
  if (!unix) return '-';
  const d = new Date(unix * 1000);
  const yyyy = d.getFullYear();
  const mm = String(d.getMonth() + 1).padStart(2, '0');
  const dd = String(d.getDate()).padStart(2, '0');
  return `${yyyy}-${mm}-${dd}`;
}

function safetyClass(s: SafetyLevel): string {
  if (s === 'safe') {
    return 'bg-green-100 text-green-800 dark:bg-green-900/40 dark:text-green-300';
  }
  if (s === 'caution') {
    return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/40 dark:text-yellow-300';
  }
  return 'bg-red-100 text-red-800 dark:bg-red-900/40 dark:text-red-300';
}

function safetyLabel(s: SafetyLevel): string {
  return t(`systemJunk.safety.${s}`);
}

// 路径截断显示:超过 max 字符则保留尾部并加省略号前缀,完整路径通过 title 暴露。
function truncatePath(p: string, max = 60): string {
  if (p.length <= max) return p;
  return '…' + p.slice(p.length - max + 1);
}
</script>

<template>
  <div class="p-6">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
      {{ t('page.systemJunk') }}
    </h1>

    <!-- 操作栏:扫描 + 清理 + 选中统计 -->
    <div class="mt-4 flex flex-wrap items-center gap-3">
      <button
        type="button"
        class="rounded-lg bg-blue-500 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="loading || cleaning"
        @click="onScan"
      >
        {{ t('common.scan') }}
      </button>
      <button
        type="button"
        class="rounded-lg bg-red-500 px-4 py-2 text-sm font-semibold text-white transition-colors hover:bg-red-600 disabled:cursor-not-allowed disabled:opacity-50"
        :disabled="cleanDisabled"
        @click="onClean"
      >
        {{ t('common.clean') }}
      </button>
      <span v-if="hasItems" class="text-sm text-gray-600 dark:text-gray-300">
        {{
          t('systemJunk.selected', {
            count: selectedCount,
            size: formatSize(selectedSize),
          })
        }}
      </span>
    </div>

    <!-- 错误提示 -->
    <div
      v-if="errorMsg"
      class="mt-4 rounded-lg border border-red-300 bg-red-50 p-3 text-sm text-red-700 dark:border-red-800 dark:bg-red-950/40 dark:text-red-300"
    >
      {{ errorMsg }}
    </div>

    <!-- 扫描中 -->
    <div
      v-if="loading"
      class="mt-4 flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400"
    >
      <svg
        class="h-4 w-4 animate-spin text-blue-500"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M21 12a9 9 0 1 1-6.219-8.56" />
      </svg>
      <span>{{ t('systemJunk.scanning') }}</span>
    </div>

    <!-- 清理中 -->
    <div
      v-if="cleaning"
      class="mt-4 flex items-center gap-2 text-sm text-blue-600 dark:text-blue-400"
    >
      <svg
        class="h-4 w-4 animate-spin"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M21 12a9 9 0 1 1-6.219-8.56" />
      </svg>
      <span>{{ t('systemJunk.cleaning') }}</span>
    </div>

    <!-- 完成报告:成功数 + 释放空间 + 失败列表(如有) -->
    <div
      v-if="cleanOutcome"
      class="mt-4 rounded-lg border border-green-300 bg-green-50 p-3 text-sm text-green-800 dark:border-green-800 dark:bg-green-950/40 dark:text-green-300"
    >
      {{
        t('systemJunk.cleanComplete', {
          success: cleanOutcome.success.length,
          failed: cleanOutcome.failed.length,
          size: formatSize(freedSize),
        })
      }}
      <ul v-if="cleanOutcome.failed.length" class="mt-2 list-disc pl-5">
        <li v-for="f in cleanOutcome.failed" :key="f.id">{{ f.path }} — {{ f.reason }}</li>
      </ul>
    </div>

    <!-- 空状态 -->
    <div
      v-if="!loading && result && !hasItems"
      class="mt-6 rounded-2xl border border-dashed border-gray-300 bg-gray-50/60 p-12 text-center text-sm text-gray-500 dark:border-zinc-700 dark:bg-zinc-900/40"
    >
      {{ t('systemJunk.empty') }}
    </div>

    <!-- 结果列表 -->
    <div
      v-if="hasItems"
      class="mt-4 overflow-hidden rounded-2xl border border-gray-200/60 bg-white/70 dark:border-zinc-800/60 dark:bg-zinc-900/70"
    >
      <table class="w-full text-left text-sm">
        <thead
          class="bg-gray-50/80 text-xs uppercase text-gray-500 dark:bg-zinc-800/80 dark:text-zinc-400"
        >
          <tr>
            <th class="w-10 px-3 py-2"></th>
            <th class="px-3 py-2">{{ t('common.path') }}</th>
            <th class="px-3 py-2">{{ t('common.size') }}</th>
            <th class="px-3 py-2">{{ t('systemJunk.modifiedAt') }}</th>
            <th class="px-3 py-2">{{ t('systemJunk.safetyLevel') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr
            v-for="item in items"
            :key="item.id"
            class="border-t border-gray-100 dark:border-zinc-800"
          >
            <td class="px-3 py-2">
              <input type="checkbox" :checked="selected.has(item.id)" @change="toggle(item.id)" />
            </td>
            <td
              class="px-3 py-2 font-mono text-xs text-gray-700 dark:text-gray-300"
              :title="item.path"
            >
              {{ truncatePath(item.path) }}
            </td>
            <td class="px-3 py-2 text-gray-700 dark:text-gray-300">
              {{ formatSize(item.sizeBytes) }}
            </td>
            <td class="px-3 py-2 text-gray-500 dark:text-gray-400">
              {{ formatDate(item.modifiedAt) }}
            </td>
            <td class="px-3 py-2">
              <span
                class="rounded-full px-2 py-0.5 text-xs font-medium"
                :class="safetyClass(item.safety)"
              >
                {{ safetyLabel(item.safety) }}
              </span>
            </td>
          </tr>
        </tbody>
      </table>
    </div>

    <!-- 深度清理开启时的占位提示(系统级扫描作为后续增强) -->
    <div
      v-if="appStore.deepClean"
      class="mt-4 rounded-lg border border-amber-300 bg-amber-50 p-3 text-sm text-amber-800 dark:border-amber-800 dark:bg-amber-950/40 dark:text-amber-300"
    >
      {{ t('settings.deepCleanWarning') }}
    </div>

    <!-- 二次确认弹窗:替换原 window.confirm -->
    <ConfirmDialog
      :visible="confirmVisible"
      :title="confirmTitle"
      :message="confirmMessage"
      :detail-items="confirmDetails"
      @confirm="onCleanConfirm"
      @cancel="confirmVisible = false"
    />
  </div>
</template>
