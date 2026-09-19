<script setup lang="ts">
// 大文件扫描与重复文件检测页面(Task 6)。
// - 双 Tab:大文件(阈值过滤)/ 重复文件(SHA-256 分组)
// - 扫描路径多选(默认勾选 5 个用户目录)、阈值输入、扫描按钮
// - 大文件复选框默认不勾(谨慎清理);重复文件组默认保留最早修改,其余勾选
// - 二次确认后调用 clean_large_files 移至废纸篓
import { computed, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import type { CleanOutcome, DuplicateGroup, ScanItem } from '@/types';
import ConfirmDialog from '@/components/ConfirmDialog.vue';

const { t } = useI18n();

// 默认扫描的 5 个用户子目录(与后端 DEFAULT_SCAN_DIRS 对齐),前端只传目录名,
// 后端 resolve_scan_paths 会相对 home 解析。
const SCAN_PATH_OPTIONS = ['Downloads', 'Documents', 'Movies', 'Pictures', 'Desktop'] as const;

type Tab = 'large' | 'duplicates';
type FileCategory = 'video' | 'image' | 'archive' | 'other';

const activeTab = ref<Tab>('large');
const thresholdMb = ref(50);
// 默认勾选全部 5 个目录。
const selectedPaths = ref<string[]>([...SCAN_PATH_OPTIONS]);
const scanning = ref(false);
const errorMsg = ref('');

const largeFiles = ref<ScanItem[]>([]);
const duplicateGroups = ref<DuplicateGroup[]>([]);
// 待清理的文件路径集合(大文件默认空;重复文件默认除每组第一个外全选)。
const selectedIds = ref<Set<string>>(new Set());
// ConfirmDialog 显隐:删除前二次确认
const confirmVisible = ref(false);

// 选中待清理的文件数。
const selectedCount = computed(() => selectedIds.value.size);

// 路径 -> 字节数 的查找表(合并两份结果),用于计算将释放的空间。
const sizeByPath = computed<Map<string, number>>(() => {
  const m = new Map<string, number>();
  for (const it of largeFiles.value) m.set(it.path, it.sizeBytes);
  for (const g of duplicateGroups.value) for (const f of g.files) m.set(f.path, f.sizeBytes);
  return m;
});

// 选中项将释放的空间(字节)。
const freedBytes = computed(() => {
  let total = 0;
  for (const id of selectedIds.value) total += sizeByPath.value.get(id) ?? 0;
  return total;
});

// ConfirmDialog 内容(在 selectedCount / freedBytes / sizeByPath 之后定义以避免 use-before-define)
const confirmTitle = computed(() => t('largeFiles.confirmTitle'));
const confirmMessage = computed(() =>
  t('largeFiles.confirmDelete', {
    count: selectedCount.value,
    size: formatSize(freedBytes.value),
  }),
);
// 详情项:全部选中文件(由 ConfirmDialog 截到 10 并显示"...等 N 项"溢出提示)
const confirmDetails = computed(() => {
  const items: { path: string; size: number }[] = [];
  for (const id of selectedIds.value) {
    const size = sizeByPath.value.get(id);
    if (size !== undefined) items.push({ path: id, size });
  }
  return items;
});

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

function formatTime(unix: number): string {
  if (!unix) return '—';
  return new Date(unix * 1000).toLocaleString();
}

function baseName(path: string): string {
  const idx = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
  return idx >= 0 ? path.slice(idx + 1) : path;
}

// 按扩展名分类(后端 ScanItem.label 即小写扩展名)。
function fileCategory(label: string | null): FileCategory {
  if (!label) return 'other';
  const e = label.toLowerCase();
  if (e === 'mp4' || e === 'mov') return 'video';
  if (e === 'jpg' || e === 'png') return 'image';
  if (e === 'zip' || e === 'rar') return 'archive';
  return 'other';
}

const CATEGORY_CLASS: Record<FileCategory, string> = {
  video: 'bg-rose-100 text-rose-700 dark:bg-rose-500/15 dark:text-rose-300',
  image: 'bg-emerald-100 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-300',
  archive: 'bg-amber-100 text-amber-700 dark:bg-amber-500/15 dark:text-amber-300',
  other: 'bg-gray-100 text-gray-600 dark:bg-zinc-700/40 dark:text-zinc-300',
};

function categoryLabel(c: FileCategory): string {
  return t(`largeFiles.${c}`);
}

function togglePath(path: string) {
  const next = new Set(selectedIds.value);
  if (next.has(path)) next.delete(path);
  else next.add(path);
  selectedIds.value = next;
}

function toggleAllLarge(checked: boolean) {
  const next = new Set(selectedIds.value);
  if (checked) for (const it of largeFiles.value) next.add(it.path);
  else for (const it of largeFiles.value) next.delete(it.path);
  selectedIds.value = next;
}

const allLargeChecked = computed(
  () =>
    largeFiles.value.length > 0 && largeFiles.value.every((it) => selectedIds.value.has(it.path)),
);

async function runScan() {
  scanning.value = true;
  errorMsg.value = '';
  try {
    const paths = [...selectedPaths.value];
    if (activeTab.value === 'large') {
      const result = await invoke<ScanItem[]>('scan_large_files', {
        thresholdMb: thresholdMb.value,
        scanPaths: paths,
      });
      largeFiles.value = result;
      // 大文件默认不勾(谨慎清理)。
      selectedIds.value = new Set();
    } else {
      const result = await invoke<DuplicateGroup[]>('scan_duplicates', { scanPaths: paths });
      duplicateGroups.value = result;
      // 重复文件:每组默认保留第一个(最早修改)不勾,其余勾选。
      const next = new Set<string>();
      for (const g of result) {
        g.files.forEach((f, i) => {
          if (i > 0) next.add(f.path);
        });
      }
      selectedIds.value = next;
    }
  } catch (e) {
    errorMsg.value = typeof e === 'string' ? e : String(e);
  } finally {
    scanning.value = false;
  }
}

// 点击删除按钮:打开 ConfirmDialog 二次确认(替换原 window.confirm)
function deleteSelected() {
  if (selectedIds.value.size === 0) return;
  confirmVisible.value = true;
}

// 确认回调:执行实际删除
async function onConfirmDelete() {
  confirmVisible.value = false;
  const items = [...selectedIds.value];
  try {
    const outcome: CleanOutcome = await invoke<CleanOutcome>('clean_large_files', { items });
    // 移除已成功清理的项。
    const removed = new Set(outcome.success.map((c) => c.path));
    if (removed.size > 0) {
      largeFiles.value = largeFiles.value.filter((it) => !removed.has(it.path));
      duplicateGroups.value = duplicateGroups.value
        .map((g) => ({ ...g, files: g.files.filter((f) => !removed.has(f.path)) }))
        .filter((g) => g.files.length >= 2);
      const next = new Set<string>();
      for (const id of selectedIds.value) if (!removed.has(id)) next.add(id);
      selectedIds.value = next;
    }
    if (outcome.failed.length > 0) {
      errorMsg.value = outcome.failed.map((f) => `${f.path}: ${f.reason}`).join('\n');
    }
  } catch (e) {
    errorMsg.value = typeof e === 'string' ? e : String(e);
  }
}
</script>

<template>
  <div class="flex h-full flex-col p-6">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
      {{ t('page.largeFiles') }}
    </h1>

    <!-- 顶部控制区:Tab 切换 + 阈值 + 扫描路径 + 扫描按钮 -->
    <div
      class="mt-4 rounded-2xl border border-gray-200/60 bg-white/70 p-4 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/70"
    >
      <div class="flex flex-wrap items-center gap-3">
        <!-- Tab 切换 -->
        <div class="flex rounded-lg bg-gray-100 p-1 dark:bg-zinc-800">
          <button
            type="button"
            class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
            :class="
              activeTab === 'large'
                ? 'bg-white text-gray-900 shadow-sm dark:bg-zinc-700 dark:text-gray-100'
                : 'text-gray-500 hover:text-gray-700 dark:text-zinc-400 dark:hover:text-zinc-200'
            "
            @click="activeTab = 'large'"
          >
            {{ t('largeFiles.tabLarge') }}
          </button>
          <button
            type="button"
            class="rounded-md px-3 py-1.5 text-sm font-medium transition-colors"
            :class="
              activeTab === 'duplicates'
                ? 'bg-white text-gray-900 shadow-sm dark:bg-zinc-700 dark:text-gray-100'
                : 'text-gray-500 hover:text-gray-700 dark:text-zinc-400 dark:hover:text-zinc-200'
            "
            @click="activeTab = 'duplicates'"
          >
            {{ t('largeFiles.tabDuplicates') }}
          </button>
        </div>

        <!-- 阈值输入:仅大文件 Tab 显示 -->
        <label
          v-if="activeTab === 'large'"
          class="flex items-center gap-2 text-sm text-gray-600 dark:text-zinc-300"
        >
          <span>{{ t('largeFiles.threshold') }}</span>
          <input
            v-model.number="thresholdMb"
            type="number"
            min="1"
            class="w-24 rounded-md border border-gray-300 bg-white px-2 py-1 text-sm text-gray-900 focus:border-blue-500 focus:outline-none dark:border-zinc-600 dark:bg-zinc-800 dark:text-gray-100"
          />
          <span class="text-gray-400">MB</span>
        </label>

        <button
          type="button"
          class="ml-auto inline-flex items-center gap-1.5 rounded-md bg-blue-600 px-4 py-1.5 text-sm font-semibold text-white transition-colors hover:bg-blue-500 disabled:opacity-50"
          :disabled="scanning || selectedPaths.length === 0"
          @click="runScan"
        >
          {{ scanning ? t('largeFiles.scanning') : t('common.scan') }}
        </button>
      </div>

      <!-- 扫描路径多选 -->
      <div class="mt-3 flex flex-wrap items-center gap-x-4 gap-y-2">
        <span class="text-xs font-medium text-gray-500 dark:text-zinc-400">
          {{ t('largeFiles.scanPaths') }}:
        </span>
        <label
          v-for="dir in SCAN_PATH_OPTIONS"
          :key="dir"
          class="flex items-center gap-1.5 text-sm text-gray-600 dark:text-zinc-300"
        >
          <input
            type="checkbox"
            class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-zinc-600 dark:bg-zinc-800"
            :checked="selectedPaths.includes(dir)"
            @change="
              selectedPaths.includes(dir)
                ? (selectedPaths = selectedPaths.filter((d) => d !== dir))
                : selectedPaths.push(dir)
            "
          />
          <span>{{ dir }}</span>
        </label>
      </div>
    </div>

    <!-- 错误提示 -->
    <p
      v-if="errorMsg"
      class="mt-3 whitespace-pre-line rounded-md bg-red-50 px-3 py-2 text-sm text-red-600 dark:bg-red-500/10 dark:text-red-300"
    >
      {{ errorMsg }}
    </p>

    <!-- 列表区 -->
    <div class="mt-4 flex-1 overflow-auto">
      <!-- 大文件 Tab -->
      <div
        v-if="activeTab === 'large'"
        class="rounded-2xl border border-gray-200/60 bg-white/70 dark:border-zinc-800/60 dark:bg-zinc-900/70"
      >
        <div
          v-if="largeFiles.length === 0"
          class="p-12 text-center text-sm text-gray-400 dark:text-zinc-500"
        >
          {{ scanning ? t('largeFiles.scanning') : t('largeFiles.noResults') }}
        </div>
        <table v-else class="w-full text-sm">
          <thead>
            <tr
              class="border-b border-gray-200/60 text-left text-xs text-gray-500 dark:border-zinc-800/60 dark:text-zinc-400"
            >
              <th class="w-10 px-3 py-2">
                <input
                  type="checkbox"
                  class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-zinc-600 dark:bg-zinc-800"
                  :checked="allLargeChecked"
                  @change="toggleAllLarge(($event.target as HTMLInputElement).checked)"
                />
              </th>
              <th class="px-3 py-2">{{ t('largeFiles.fileName') }}</th>
              <th class="px-3 py-2">{{ t('common.path') }}</th>
              <th class="px-3 py-2">{{ t('common.size') }}</th>
              <th class="px-3 py-2">{{ t('largeFiles.modifiedAt') }}</th>
              <th class="px-3 py-2">{{ t('common.total') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="item in largeFiles"
              :key="item.id"
              class="border-b border-gray-100/60 last:border-0 hover:bg-gray-50/60 dark:border-zinc-800/40 dark:hover:bg-zinc-800/30"
            >
              <td class="px-3 py-2">
                <input
                  type="checkbox"
                  class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-zinc-600 dark:bg-zinc-800"
                  :checked="selectedIds.has(item.path)"
                  @change="togglePath(item.path)"
                />
              </td>
              <td
                class="max-w-[12rem] truncate px-3 py-2 font-medium text-gray-900 dark:text-gray-100"
              >
                {{ baseName(item.path) }}
              </td>
              <td
                class="max-w-[16rem] truncate px-3 py-2 text-gray-500 dark:text-zinc-400"
                :title="item.path"
              >
                {{ item.path }}
              </td>
              <td class="whitespace-nowrap px-3 py-2 font-mono text-gray-700 dark:text-gray-300">
                {{ formatSize(item.sizeBytes) }}
              </td>
              <td class="whitespace-nowrap px-3 py-2 text-gray-500 dark:text-zinc-400">
                {{ formatTime(item.modifiedAt) }}
              </td>
              <td class="px-3 py-2">
                <span
                  class="inline-flex rounded-full px-2 py-0.5 text-xs font-medium"
                  :class="CATEGORY_CLASS[fileCategory(item.label)]"
                >
                  {{ categoryLabel(fileCategory(item.label)) }}
                </span>
              </td>
            </tr>
          </tbody>
        </table>
      </div>

      <!-- 重复文件 Tab -->
      <div v-else class="space-y-3">
        <div
          v-if="duplicateGroups.length === 0"
          class="rounded-2xl border border-gray-200/60 bg-white/70 p-12 text-center text-sm text-gray-400 dark:border-zinc-800/60 dark:bg-zinc-900/70 dark:text-zinc-500"
        >
          {{ scanning ? t('largeFiles.scanning') : t('largeFiles.noResults') }}
        </div>
        <div
          v-for="group in duplicateGroups"
          :key="group.hash"
          class="overflow-hidden rounded-2xl border border-gray-200/60 bg-white/70 dark:border-zinc-800/60 dark:bg-zinc-900/70"
        >
          <!-- 组标题:SHA-256 | 大小 | N 个重复 -->
          <div
            class="flex flex-wrap items-center gap-x-4 gap-y-1 border-b border-gray-200/60 bg-gray-50/60 px-4 py-2 dark:border-zinc-800/60 dark:bg-zinc-800/30"
          >
            <span class="font-mono text-xs text-gray-500 dark:text-zinc-400" :title="group.hash">
              SHA-256: {{ group.hash.slice(0, 16) }}…
            </span>
            <span class="font-mono text-sm font-semibold text-gray-900 dark:text-gray-100">
              {{ formatSize(group.sizeBytes) }}
            </span>
            <span class="text-xs text-gray-500 dark:text-zinc-400">
              {{ t('largeFiles.duplicateGroup', { count: group.files.length }) }}
            </span>
            <span class="ml-auto text-xs text-gray-400 dark:text-zinc-500">
              {{ t('largeFiles.keepFirst') }}
            </span>
          </div>
          <!-- 组内文件列表:第一个(mtime 最早)默认不勾,其余默认勾选 -->
          <table class="w-full text-sm">
            <tbody>
              <tr
                v-for="(file, idx) in group.files"
                :key="file.id"
                class="border-b border-gray-100/60 last:border-0 hover:bg-gray-50/60 dark:border-zinc-800/40 dark:hover:bg-zinc-800/30"
                :class="idx === 0 ? 'bg-emerald-50/40 dark:bg-emerald-500/5' : ''"
              >
                <td class="w-10 px-3 py-2">
                  <input
                    type="checkbox"
                    class="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-zinc-600 dark:bg-zinc-800"
                    :checked="selectedIds.has(file.path)"
                    @change="togglePath(file.path)"
                  />
                </td>
                <td
                  class="max-w-[24rem] truncate px-3 py-2 text-gray-700 dark:text-gray-300"
                  :title="file.path"
                >
                  {{ file.path }}
                </td>
                <td class="whitespace-nowrap px-3 py-2 text-gray-500 dark:text-zinc-400">
                  {{ formatTime(file.modifiedAt) }}
                </td>
                <td v-if="idx === 0" class="px-3 py-2 text-right">
                  <span class="text-xs font-medium text-emerald-600 dark:text-emerald-400">
                    {{ t('largeFiles.keepFirst') }}
                  </span>
                </td>
                <td v-else class="px-3 py-2"></td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>
    </div>

    <!-- 底部:删除按钮 + 释放空间统计 -->
    <div
      class="mt-4 flex shrink-0 items-center gap-4 rounded-2xl border border-gray-200/60 bg-white/70 px-4 py-3 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/70"
    >
      <span class="text-sm text-gray-600 dark:text-zinc-300">
        {{ t('largeFiles.selectedCount', { count: selectedCount }) }}
      </span>
      <span class="text-sm text-gray-400 dark:text-zinc-500">·</span>
      <span class="text-sm text-gray-600 dark:text-zinc-300">
        {{ t('largeFiles.freed') }}:
        <span class="font-mono font-semibold text-gray-900 dark:text-gray-100">
          {{ formatSize(freedBytes) }}
        </span>
      </span>
      <button
        type="button"
        class="ml-auto inline-flex items-center rounded-md bg-red-600 px-4 py-1.5 text-sm font-semibold text-white transition-colors hover:bg-red-500 disabled:opacity-50"
        :disabled="selectedCount === 0 || scanning"
        @click="deleteSelected"
      >
        {{ t('largeFiles.deleteSelected') }}
      </button>
    </div>

    <!-- 二次确认弹窗:替换原 window.confirm -->
    <ConfirmDialog
      :visible="confirmVisible"
      :title="confirmTitle"
      :message="confirmMessage"
      :detail-items="confirmDetails"
      @confirm="onConfirmDelete"
      @cancel="confirmVisible = false"
    />
  </div>
</template>
