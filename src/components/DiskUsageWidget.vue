<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';

// Tauri 命令 get_disk_usage 的返回结构(Task 3 实现)
interface DiskUsage {
  total: number; // bytes
  used: number; // bytes
  available: number; // bytes
}

const { t } = useI18n();

// 占位数据,用于命令未实现或调用失败时兜底显示
const PLACEHOLDER: DiskUsage = {
  total: 512 * 1024 * 1024 * 1024,
  used: 234 * 1024 * 1024 * 1024,
  available: (512 - 234) * 1024 * 1024 * 1024,
};

const loading = ref(true);
const disk = ref<DiskUsage>({ ...PLACEHOLDER });

// 字节转 GB(保留 1 位小数)
function formatGB(bytes: number): number {
  return Math.round((bytes / 1024 / 1024 / 1024) * 10) / 10;
}

const usedGB = computed(() => formatGB(disk.value.used));
const totalGB = computed(() => formatGB(disk.value.total));
const availableGB = computed(() => formatGB(disk.value.available));

// 已用占比(0-1),用于环形进度
const ratio = computed(() => {
  if (disk.value.total <= 0) return 0;
  return Math.min(1, Math.max(0, disk.value.used / disk.value.total));
});

// 环形周长(r=40),用于 stroke-dasharray / stroke-dashoffset
const RADIUS = 40;
const circumference = 2 * Math.PI * RADIUS;
const strokeOffset = computed(() => circumference * (1 - ratio.value));

onMounted(async () => {
  try {
    const result = await invoke<DiskUsage>('get_disk_usage');
    disk.value = result;
  } catch {
    // 命令尚未实现(Task 3 实现)或运行在非 Tauri 环境,使用占位数据
    disk.value = { ...PLACEHOLDER };
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <section
    class="flex items-center gap-6 rounded-2xl border border-gray-200/60 bg-white/70 p-5 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/70"
    aria-label="disk usage"
  >
    <!-- 左侧:环形进度图 -->
    <div class="relative h-24 w-24 shrink-0">
      <svg viewBox="0 0 100 100" class="h-24 w-24 -rotate-90">
        <circle
          cx="50"
          cy="50"
          :r="RADIUS"
          fill="none"
          stroke-width="10"
          class="text-gray-200 dark:text-zinc-700"
          stroke="currentColor"
        />
        <circle
          cx="50"
          cy="50"
          :r="RADIUS"
          fill="none"
          stroke-width="10"
          stroke-linecap="round"
          class="text-blue-500"
          stroke="currentColor"
          :stroke-dasharray="circumference"
          :stroke-dashoffset="strokeOffset"
        />
      </svg>
      <div
        class="absolute inset-0 flex items-center justify-center text-sm font-semibold text-gray-700 dark:text-gray-200"
      >
        {{ Math.round(ratio * 100) }}%
      </div>
    </div>

    <!-- 右侧:数字 -->
    <div class="flex flex-col gap-1">
      <div class="text-sm text-gray-500 dark:text-gray-400">
        {{ t('disk.used') }}
      </div>
      <div class="text-lg font-semibold text-gray-900 dark:text-gray-100">
        {{ usedGB }} GB <span class="text-gray-400">/ {{ totalGB }} GB</span>
      </div>
      <div class="text-sm text-gray-600 dark:text-gray-400">
        {{ t('disk.available') }}: {{ availableGB }} GB
      </div>
      <div v-if="loading" class="text-xs text-gray-400 dark:text-gray-500">…</div>
    </div>
  </section>
</template>
