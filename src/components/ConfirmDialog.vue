<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { formatSize } from '@/utils/format';

interface DetailItem {
  path: string;
  size: number;
}

const props = withDefaults(
  defineProps<{
    visible: boolean;
    title: string;
    message: string;
    detailItems?: DetailItem[];
    confirmText?: string;
    cancelText?: string;
  }>(),
  {
    detailItems: () => [],
    confirmText: '',
    cancelText: '',
  },
);

const emit = defineEmits<{
  (e: 'confirm'): void;
  (e: 'cancel'): void;
}>();

const { t } = useI18n();

// 最多展示 10 条;超出部分以"...等 N 项"汇总。
const MAX_DETAIL = 10;
const visibleDetails = computed<DetailItem[]>(() => (props.detailItems ?? []).slice(0, MAX_DETAIL));
const hiddenCount = computed(() => {
  const total = (props.detailItems ?? []).length;
  return Math.max(0, total - MAX_DETAIL);
});

const confirmLabel = computed(() => props.confirmText || t('common.confirm'));
const cancelLabel = computed(() => props.cancelText || t('common.cancel'));

function onConfirm() {
  emit('confirm');
}

function onCancel() {
  emit('cancel');
}
</script>

<template>
  <div
    v-if="visible"
    class="fixed inset-0 z-50 flex items-center justify-center bg-black/40 backdrop-blur-sm"
    @click.self="onCancel"
  >
    <div
      class="w-[28rem] max-w-[90vw] overflow-hidden rounded-2xl border border-gray-200 bg-white shadow-2xl dark:border-zinc-700 dark:bg-zinc-900"
    >
      <h3 class="px-5 pt-5 text-base font-semibold text-gray-900 dark:text-gray-100">
        {{ title }}
      </h3>
      <p class="px-5 pt-2 text-sm text-gray-600 dark:text-gray-300">{{ message }}</p>

      <!-- 详情列表:仅当 detailItems 非空时显示,最多 10 条 + 溢出提示 -->
      <div
        v-if="visibleDetails.length > 0"
        class="mx-5 mt-3 max-h-48 overflow-auto rounded-lg border border-gray-200 bg-gray-50/80 dark:border-zinc-700 dark:bg-zinc-800/60"
      >
        <ul class="divide-y divide-gray-100 dark:divide-zinc-800">
          <li
            v-for="(item, idx) in visibleDetails"
            :key="idx"
            class="flex items-start gap-2 px-3 py-2 text-xs"
          >
            <span
              class="min-w-0 flex-1 break-all font-mono text-gray-700 dark:text-gray-300"
              :title="item.path"
            >
              {{ item.path }}
            </span>
            <span class="shrink-0 text-gray-500 dark:text-zinc-400">
              {{ formatSize(item.size) }}
            </span>
          </li>
        </ul>
        <p
          v-if="hiddenCount > 0"
          class="px-3 py-2 text-center text-xs text-gray-500 dark:text-zinc-400"
        >
          {{ t('confirm.detailCount', { count: hiddenCount }) }}
        </p>
      </div>

      <div class="flex justify-end gap-2 px-5 pb-5 pt-4">
        <button
          type="button"
          class="rounded-md border border-gray-300 px-4 py-1.5 text-sm font-medium text-gray-700 transition-colors hover:bg-gray-50 dark:border-zinc-600 dark:text-gray-300 dark:hover:bg-zinc-800"
          @click="onCancel"
        >
          {{ cancelLabel }}
        </button>
        <button
          type="button"
          class="rounded-md bg-red-600 px-4 py-1.5 text-sm font-semibold text-white transition-colors hover:bg-red-500"
          @click="onConfirm"
        >
          {{ confirmLabel }}
        </button>
      </div>
    </div>
  </div>
</template>
