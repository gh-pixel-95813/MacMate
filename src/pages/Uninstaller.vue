<script setup lang="ts">
// 应用卸载器页面(Task 5)。
// - 顶部:扫描按钮 + 按应用名搜索的过滤输入框
// - 中间:应用列表(图标首字母圆角块 + 名称 + 大小 + 关联文件数 + 卸载按钮)
// - 点击应用名展开:本体路径 + 8 个关联文件位置(exists 绿勾,不存在灰显,安全等级徽章)
// - 卸载按钮:二次确认对话框(列出将删除的应用本体 + N 个关联文件 + 共 X MB)
// - 完成报告:成功 / 失败项数,失败项可展开
import { computed, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import type { AppEntry, CleanOutcome, RelatedPath } from '@/types';
import ConfirmDialog from '@/components/ConfirmDialog.vue';

const { t } = useI18n();

const apps = ref<AppEntry[]>([]);
const scanning = ref(false);
const searchQuery = ref('');
const expandedPath = ref<string | null>(null);
const errorMsg = ref('');
// 待确认卸载的应用(点击卸载按钮后弹出确认框)
const confirmTarget = ref<AppEntry | null>(null);
// ConfirmDialog 显隐由 confirmTarget 控制:非空即显示
const confirmVisible = computed(() => confirmTarget.value !== null);
const confirmTitle = computed(() => t('uninstaller.confirmTitle'));
const confirmMessage = computed(() => {
  const target = confirmTarget.value;
  if (!target) return '';
  return t('uninstaller.confirmUninstall', {
    name: target.name,
    related: existingRelatedCount(target),
    size: formatSize(totalUninstallSize(target)),
  });
});
const uninstalling = ref(false);
// 上一次卸载结果(成功 / 失败项),供完成报告展示
const lastOutcome = ref<CleanOutcome | null>(null);
const showFailedDetail = ref(false);

// 按应用名过滤(大小写不敏感)
const filteredApps = computed(() => {
  const q = searchQuery.value.trim().toLowerCase();
  if (!q) return apps.value;
  return apps.value.filter((a) => a.name.toLowerCase().includes(q));
});

// 安全等级 -> 徽章样式类
const SAFETY_CLASS: Record<string, string> = {
  safe: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-300',
  caution: 'bg-amber-100 text-amber-700 dark:bg-amber-500/15 dark:text-amber-300',
  danger: 'bg-rose-100 text-rose-700 dark:bg-rose-500/15 dark:text-rose-300',
};

function safetyLabel(s: string): string {
  return s;
}

// 字节数 -> 人类可读
function formatSize(bytes: number): string {
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

// 应用名首字母(用于图标占位)
function initial(name: string): string {
  return name.charAt(0).toUpperCase() || '?';
}

// 图标背景色:按首字母 hash 取色,保持稳定
function iconColor(name: string): string {
  const colors = [
    'bg-blue-500',
    'bg-emerald-500',
    'bg-amber-500',
    'bg-rose-500',
    'bg-violet-500',
    'bg-cyan-500',
    'bg-pink-500',
    'bg-indigo-500',
  ];
  let hash = 0;
  for (const ch of name) hash = (hash * 31 + ch.charCodeAt(0)) | 0;
  return colors[Math.abs(hash) % colors.length];
}

// 存在的关联文件数(用于列表摘要与确认对话框)
function existingRelatedCount(app: AppEntry): number {
  return app.related.filter((r) => r.exists).length;
}

// 将释放的总空间(应用本体 + 存在的关联文件)
function totalUninstallSize(app: AppEntry): number {
  const relatedSize = app.related.filter((r) => r.exists).reduce((s, r) => s + r.sizeBytes, 0);
  return app.sizeBytes + relatedSize;
}

// 存在的关联路径列表(传给 uninstall_app)
function existingRelatedPaths(app: AppEntry): string[] {
  return app.related.filter((r) => r.exists).map((r) => r.path);
}

async function runScan() {
  scanning.value = true;
  errorMsg.value = '';
  lastOutcome.value = null;
  try {
    apps.value = await invoke<AppEntry[]>('scan_applications');
  } catch (e) {
    errorMsg.value = typeof e === 'string' ? e : String(e);
  } finally {
    scanning.value = false;
  }
}

function toggleExpand(app: AppEntry) {
  if (expandedPath.value === app.path) {
    expandedPath.value = null;
  } else {
    expandedPath.value = app.path;
  }
}

function startUninstall(app: AppEntry) {
  // 系统应用不提供卸载
  if (app.isSystem) return;
  confirmTarget.value = app;
}

function cancelUninstall() {
  confirmTarget.value = null;
}

async function confirmUninstall() {
  const target = confirmTarget.value;
  if (!target) return;
  uninstalling.value = true;
  errorMsg.value = '';
  try {
    const relatedPaths = existingRelatedPaths(target);
    const outcome = await invoke<CleanOutcome>('uninstall_app', {
      appPath: target.path,
      relatedPaths,
    });
    lastOutcome.value = outcome;
    // 从列表中移除已成功卸载的应用
    const removed = new Set(outcome.success.map((c) => c.path));
    if (removed.size > 0) {
      apps.value = apps.value.filter((a) => !removed.has(a.path));
    }
  } catch (e) {
    // 应用正在运行等错误
    errorMsg.value = typeof e === 'string' ? e : String(e);
  } finally {
    uninstalling.value = false;
    confirmTarget.value = null;
  }
}

// 刷新单个应用的关联文件(调用 find_app_related)
async function refreshRelated(app: AppEntry) {
  try {
    const related = await invoke<RelatedPath[]>('find_app_related', {
      bundleId: app.bundleId ?? '',
      appName: app.name,
    });
    const idx = apps.value.findIndex((a) => a.path === app.path);
    if (idx >= 0) {
      apps.value[idx] = { ...apps.value[idx], related };
    }
  } catch (e) {
    errorMsg.value = typeof e === 'string' ? e : String(e);
  }
}
</script>

<template>
  <div class="flex h-full flex-col p-6">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
      {{ t('page.uninstaller') }}
    </h1>

    <!-- 顶部:扫描按钮 + 搜索输入框 -->
    <div
      class="mt-4 flex flex-wrap items-center gap-3 rounded-2xl border border-gray-200/60 bg-white/70 p-4 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/70"
    >
      <button
        type="button"
        class="inline-flex items-center gap-1.5 rounded-md bg-blue-600 px-4 py-1.5 text-sm font-semibold text-white transition-colors hover:bg-blue-500 disabled:opacity-50"
        :disabled="scanning"
        @click="runScan"
      >
        {{ scanning ? t('common.scan') : t('uninstaller.scanApps') }}
      </button>
      <input
        v-model="searchQuery"
        type="text"
        :placeholder="t('uninstaller.searchPlaceholder')"
        class="ml-auto w-64 rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm text-gray-900 placeholder:text-gray-400 focus:border-blue-500 focus:outline-none dark:border-zinc-600 dark:bg-zinc-800 dark:text-gray-100 dark:placeholder:text-zinc-500"
      />
    </div>

    <!-- 错误提示 -->
    <p
      v-if="errorMsg"
      class="mt-3 whitespace-pre-line rounded-md bg-red-50 px-3 py-2 text-sm text-red-600 dark:bg-red-500/10 dark:text-red-300"
    >
      {{ errorMsg }}
    </p>

    <!-- 完成报告:成功 / 失败项数 -->
    <div
      v-if="lastOutcome"
      class="mt-3 rounded-2xl border border-gray-200/60 bg-white/70 p-4 dark:border-zinc-800/60 dark:bg-zinc-900/70"
    >
      <div class="flex items-center gap-4">
        <span class="text-sm text-emerald-600 dark:text-emerald-400">
          {{ lastOutcome.success.length }}
        </span>
        <span class="text-sm text-gray-600 dark:text-zinc-300">/</span>
        <span class="text-sm text-rose-600 dark:text-rose-400">
          {{ lastOutcome.failed.length }}
        </span>
        <button
          v-if="lastOutcome.failed.length > 0"
          type="button"
          class="ml-auto text-xs text-blue-600 hover:underline dark:text-blue-400"
          @click="showFailedDetail = !showFailedDetail"
        >
          {{ showFailedDetail ? t('common.cancel') : t('uninstaller.relatedFiles') }}
        </button>
      </div>
      <!-- 失败项展开 -->
      <ul v-if="showFailedDetail && lastOutcome.failed.length > 0" class="mt-2 space-y-1">
        <li
          v-for="f in lastOutcome.failed"
          :key="f.id"
          class="truncate text-xs text-rose-600 dark:text-rose-400"
          :title="f.path"
        >
          {{ f.path }}: {{ f.reason }}
        </li>
      </ul>
    </div>

    <!-- 应用列表 -->
    <div class="mt-4 flex-1 overflow-auto">
      <div
        v-if="apps.length === 0"
        class="rounded-2xl border border-dashed border-gray-300 bg-gray-50/60 p-12 text-center text-sm text-gray-400 dark:border-zinc-700 dark:bg-zinc-900/40 dark:text-zinc-500"
      >
        {{ scanning ? t('common.scan') : t('uninstaller.scanApps') }}
      </div>
      <div v-else class="space-y-2">
        <div
          v-for="app in filteredApps"
          :key="app.path"
          class="overflow-hidden rounded-2xl border border-gray-200/60 bg-white/70 dark:border-zinc-800/60 dark:bg-zinc-900/70"
        >
          <!-- 应用行:图标 + 名称 + 大小 + 关联数 + 卸载按钮 -->
          <div class="flex items-center gap-3 px-4 py-3">
            <!-- 图标:首字母圆角块 -->
            <div
              class="flex h-10 w-10 shrink-0 items-center justify-center rounded-lg text-base font-bold text-white"
              :class="iconColor(app.name)"
            >
              {{ initial(app.name) }}
            </div>
            <!-- 名称(可点击展开) -->
            <button
              type="button"
              class="truncate text-left text-sm font-medium text-gray-900 hover:text-blue-600 dark:text-gray-100 dark:hover:text-blue-400"
              :class="expandedPath === app.path ? 'text-blue-600 dark:text-blue-400' : ''"
              @click="toggleExpand(app)"
            >
              {{ app.name }}
              <span
                v-if="app.isSystem"
                class="ml-2 rounded bg-gray-200 px-1.5 py-0.5 text-xs text-gray-600 dark:bg-zinc-700 dark:text-zinc-300"
              >
                System
              </span>
            </button>
            <!-- 大小 -->
            <span
              class="ml-auto whitespace-nowrap font-mono text-sm text-gray-700 dark:text-gray-300"
            >
              {{ formatSize(app.sizeBytes) }}
            </span>
            <!-- 关联文件数 -->
            <span class="whitespace-nowrap text-xs text-gray-500 dark:text-zinc-400">
              {{ existingRelatedCount(app) }} / {{ app.related.length }}
            </span>
            <!-- 卸载按钮 -->
            <button
              type="button"
              class="inline-flex items-center rounded-md px-3 py-1 text-xs font-semibold transition-colors disabled:cursor-not-allowed disabled:opacity-50"
              :class="
                app.isSystem || app.isRunning
                  ? 'bg-gray-100 text-gray-400 dark:bg-zinc-800 dark:text-zinc-600'
                  : 'bg-red-600 text-white hover:bg-red-500'
              "
              :disabled="app.isSystem || app.isRunning || uninstalling"
              :title="
                app.isRunning
                  ? t('uninstaller.appRunning')
                  : app.isSystem
                    ? ''
                    : t('uninstaller.uninstall')
              "
              @click="startUninstall(app)"
            >
              {{ t('uninstaller.uninstall') }}
            </button>
          </div>

          <!-- 展开详情:本体路径 + 8 个关联文件位置 -->
          <div
            v-if="expandedPath === app.path"
            class="border-t border-gray-200/60 bg-gray-50/40 px-4 py-3 dark:border-zinc-800/60 dark:bg-zinc-800/20"
          >
            <!-- 本体路径 -->
            <div class="mb-2">
              <span class="text-xs font-medium text-gray-500 dark:text-zinc-400">
                {{ t('common.path') }}:
              </span>
              <span class="ml-2 break-all font-mono text-xs text-gray-700 dark:text-gray-300">
                {{ app.path }}
              </span>
            </div>
            <!-- 关联文件标题 + 刷新按钮 -->
            <div class="mb-2 flex items-center">
              <span class="text-xs font-medium text-gray-500 dark:text-zinc-400">
                {{ t('uninstaller.relatedFiles') }}
              </span>
              <button
                type="button"
                class="ml-auto text-xs text-blue-600 hover:underline dark:text-blue-400"
                @click="refreshRelated(app)"
              >
                ↻
              </button>
            </div>
            <!-- 8 个关联位置 -->
            <ul class="space-y-1">
              <li
                v-for="(rel, idx) in app.related"
                :key="idx"
                class="flex items-start gap-2 text-xs"
                :class="
                  rel.exists
                    ? 'text-gray-700 dark:text-gray-300'
                    : 'text-gray-400 dark:text-zinc-600'
                "
              >
                <!-- exists 绿勾 / 不存在灰显 -->
                <span v-if="rel.exists" class="mt-0.5 text-emerald-500">✓</span>
                <span v-else class="mt-0.5 text-gray-300 dark:text-zinc-700">✗</span>
                <span class="break-all font-mono" :title="rel.path">
                  {{ rel.path }}
                </span>
                <span
                  v-if="rel.exists"
                  class="ml-auto whitespace-nowrap text-gray-500 dark:text-zinc-400"
                >
                  {{ formatSize(rel.sizeBytes) }}
                </span>
                <span v-else class="ml-auto whitespace-nowrap">
                  {{ t('uninstaller.notExist') }}
                </span>
                <!-- 安全等级徽章 -->
                <span
                  class="inline-flex shrink-0 rounded-full px-2 py-0.5 text-xs font-medium"
                  :class="SAFETY_CLASS[rel.safety]"
                >
                  {{ safetyLabel(rel.safety) }}
                </span>
              </li>
            </ul>
          </div>
        </div>
      </div>
    </div>

    <!-- 确认对话框:替换原内联模态为统一 ConfirmDialog -->
    <ConfirmDialog
      :visible="confirmVisible"
      :title="confirmTitle"
      :message="confirmMessage"
      @confirm="confirmUninstall"
      @cancel="cancelUninstall"
    />
  </div>
</template>
