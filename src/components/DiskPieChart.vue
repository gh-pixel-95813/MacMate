<script setup lang="ts">
// 仪表盘顶部:精致磁盘使用环形图(Task 9.1)。
// - SVG 双段环形图:已用(蓝)+ 可用(灰)
// - 中央显示大号百分比与"已用 / 总量"小字
// - 右侧图例:已用 / 可用 / 总量(带色块)
// 调用后端 `get_disk_usage`,失败时回退到占位数据,保证 dev / 非 Tauri 也能展示。
import { computed, onMounted, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';

interface DiskUsage {
  total: number; // bytes
  used: number; // bytes
  available: number; // bytes
}

const { t } = useI18n();

// 占位数据:命令未实现或调用失败时兜底,保证 UI 始终可读。
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

// 已用占比(0-1),用于环形段
const usedRatio = computed(() => {
  if (disk.value.total <= 0) return 0;
  return Math.min(1, Math.max(0, disk.value.used / disk.value.total));
});
const availableRatio = computed(() => 1 - usedRatio.value);
const usedPercent = computed(() => Math.round(usedRatio.value * 100));

// 环形参数:r=60,stroke=18,viewBox=160
const RADIUS = 60;
const CIRCUMFERENCE = 2 * Math.PI * RADIUS;
// 已用段长度
const usedDash = computed(() => CIRCUMFERENCE * usedRatio.value);
// 可用段长度
const availableDash = computed(() => CIRCUMFERENCE * availableRatio.value);
// 可用段起点偏移(已用段之后)
const availableOffset = computed(() => CIRCUMFERENCE - usedDash.value);

onMounted(async () => {
  try {
    const result = await invoke<DiskUsage>('get_disk_usage');
    disk.value = result;
  } catch {
    // 命令尚未实现或运行在非 Tauri 环境,使用占位数据。
    disk.value = { ...PLACEHOLDER };
  } finally {
    loading.value = false;
  }
});
</script>

<template>
  <section
    class="flex items-center gap-8 rounded-2xl border border-gray-200/60 bg-white/70 p-6 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/70"
    aria-label="disk usage"
  >
    <!-- 左侧:大号环形图 -->
    <div class="relative h-40 w-40 shrink-0">
      <svg viewBox="0 0 160 160" class="h-40 w-40 -rotate-90">
        <!-- 底色环 -->
        <circle
          cx="80"
          cy="80"
          :r="RADIUS"
          fill="none"
          stroke-width="18"
          class="text-gray-200 dark:text-zinc-700"
          stroke="currentColor"
        />
        <!-- 已用段(蓝色) -->
        <circle
          v-if="usedDash > 0"
          cx="80"
          cy="80"
          :r="RADIUS"
          fill="none"
          stroke-width="18"
          stroke-linecap="round"
          class="text-blue-500"
          stroke="currentColor"
          :stroke-dasharray="`${usedDash} ${CIRCUMFERENCE}`"
          stroke-dashoffset="0"
        />
        <!-- 可用段(灰色) -->
        <circle
          v-if="availableDash > 0"
          cx="80"
          cy="80"
          :r="RADIUS"
          fill="none"
          stroke-width="18"
          stroke-linecap="round"
          class="text-gray-300 dark:text-zinc-600"
          stroke="currentColor"
          :stroke-dasharray="`${availableDash} ${CIRCUMFERENCE}`"
          :stroke-dashoffset="availableOffset"
        />
      </svg>
      <!-- 中央百分比(rotate 反向修正以正读) -->
      <div class="absolute inset-0 flex flex-col items-center justify-center">
        <span class="text-3xl font-bold text-gray-900 dark:text-gray-100">
          {{ usedPercent }}%
        </span>
        <span class="mt-1 text-xs text-gray-500 dark:text-gray-400">
          {{ t('disk.used') }}
        </span>
      </div>
    </div>

    <!-- 右侧:图例 -->
    <div class="flex flex-col gap-3">
      <div class="flex items-center gap-2">
        <span class="inline-block h-3 w-3 rounded-full bg-blue-500"></span>
        <span class="text-sm text-gray-600 dark:text-gray-300">
          {{ t('disk.used') }}:
          <span class="font-semibold text-gray-900 dark:text-gray-100">{{ usedGB }} GB</span>
        </span>
      </div>
      <div class="flex items-center gap-2">
        <span class="inline-block h-3 w-3 rounded-full bg-gray-300 dark:bg-zinc-600"></span>
        <span class="text-sm text-gray-600 dark:text-gray-300">
          {{ t('disk.available') }}:
          <span class="font-semibold text-gray-900 dark:text-gray-100">{{ availableGB }} GB</span>
        </span>
      </div>
      <div class="flex items-center gap-2">
        <span
          class="inline-block h-3 w-3 rounded-full border border-gray-400 dark:border-zinc-500"
        ></span>
        <span class="text-sm text-gray-600 dark:text-gray-300">
          {{ t('disk.total') }}:
          <span class="font-semibold text-gray-900 dark:text-gray-100">{{ totalGB }} GB</span>
        </span>
      </div>
      <div v-if="loading" class="text-xs text-gray-400 dark:text-gray-500">…</div>
    </div>
  </section>
</template>
