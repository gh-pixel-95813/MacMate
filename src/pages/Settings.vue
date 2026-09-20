<script setup lang="ts">
// 设置页(Task 9.2):4 个分区,保留 Task 8 的深度清理开关。
// - 通用:语言下拉 + 主题三按钮
// - 清理:深度清理开关(Task 8)+ 大文件阈值数字输入
// - 数据:清空清理历史 + 打开配置目录
// - 关于:版本 / GitHub / License
import { computed, ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import { i18n } from '@/i18n';
import { useAppStore, type Theme } from '@/stores/app';
import { useTheme } from '@/composables/useTheme';

const { t } = useI18n();
const appStore = useAppStore();
const theme = useTheme();

const deepCleanOn = computed(() => appStore.deepClean);

function toggleDeepClean() {
  appStore.setDeepClean(!appStore.deepClean);
}

// 通用:语言切换。change 时同步 i18n.global.locale.value(由 store.setLocale 持久化)。
function onLocaleChange(e: Event) {
  const value = (e.target as HTMLSelectElement).value as 'zh-CN' | 'en-US';
  i18n.global.locale.value = value;
  appStore.setLocale(value);
}

// 通用:主题切换。三按钮,由 store.setTheme 持久化,useTheme 的 watch 自动应用 dark class。
function onThemeChange(value: Theme) {
  appStore.setTheme(value);
  theme.apply();
}

// 清理:大文件阈值输入(单位 MB)。change 时持久化。
function onThresholdChange(e: Event) {
  const raw = Number((e.target as HTMLInputElement).value);
  if (Number.isFinite(raw) && raw > 0) {
    appStore.setLargeFileThresholdMb(Math.floor(raw));
  }
}

// 数据:清空清理历史。
const clearingHistory = ref(false);
const clearHistoryMsg = ref<string | null>(null);

async function onClearHistory() {
  clearingHistory.value = true;
  clearHistoryMsg.value = null;
  try {
    await invoke('clear_clean_history');
    clearHistoryMsg.value = t('dashboard.clearHistory') + ' ✓';
  } catch (e) {
    clearHistoryMsg.value = String(e);
  } finally {
    clearingHistory.value = false;
  }
}

// 数据:打开配置目录(macOS 用 open,非 macOS stub 成功)。
const openDirMsg = ref<string | null>(null);

async function onOpenConfigDir() {
  openDirMsg.value = null;
  try {
    await invoke('reveal_config_dir');
  } catch (e) {
    // 非 macOS 或 home 不可解析:回退显示路径提示。
    openDirMsg.value = `${String(e)} (~/.macmate/)`;
  }
}

// 日志上报(Task 11):GitHub PAT 管理与日志上报到 GitHub Issue。
// - onCheckTokenStatus:启动时调用一次,展示当前 PAT 是否已配置。
// - onSaveToken:把用户输入的 PAT 存到 macOS Keychain(服务名 com.macmate.app)。
// - onClearToken:删除 Keychain 中的 PAT。
// - onSubmitLogs:读取最近 N 天本地日志,脱敏 + 截断后调 GitHub API 创建 Issue。
interface GithubTokenStatus {
  configured: boolean;
  targetRepo: string;
}
interface SubmitResult {
  issueNumber: number;
  htmlUrl: string;
  filesRead: number;
  bodyBytes: number;
}

const tokenStatus = ref<GithubTokenStatus | null>(null);
const tokenInput = ref('');
const tokenMsg = ref<string | null>(null);
const submitting = ref(false);
const submitMsg = ref<string | null>(null);
const submitDays = ref(7);

async function refreshTokenStatus() {
  try {
    tokenStatus.value = await invoke<GithubTokenStatus>('get_github_token_status');
  } catch (e) {
    tokenStatus.value = null;
    tokenMsg.value = String(e);
  }
}

async function onSaveToken() {
  tokenMsg.value = null;
  const v = tokenInput.value.trim();
  if (!v) {
    tokenMsg.value = t('settings.tokenEmpty');
    return;
  }
  try {
    await invoke('save_github_token', { token: v });
    tokenInput.value = '';
    await refreshTokenStatus();
    tokenMsg.value = t('settings.tokenSaved');
  } catch (e) {
    tokenMsg.value = String(e);
  }
}

async function onClearToken() {
  tokenMsg.value = null;
  try {
    await invoke('clear_github_token');
    await refreshTokenStatus();
    tokenMsg.value = t('settings.tokenCleared');
  } catch (e) {
    tokenMsg.value = String(e);
  }
}

async function onSubmitLogs() {
  submitMsg.value = null;
  submitting.value = true;
  try {
    const result = await invoke<SubmitResult>('submit_logs_to_github', {
      days: submitDays.value,
    });
    submitMsg.value = `${t('settings.submitOk')} #${result.issueNumber} ${result.htmlUrl}`;
  } catch (e) {
    submitMsg.value = String(e);
  } finally {
    submitting.value = false;
  }
}

// 启动时拉一次 PAT 状态:Settings 页是首次能拿到结果的地方。
void refreshTokenStatus();

// 关于:版本号从 store 读(App.vue hydrate 时已经从后端 get_app_version 拿到)。
const GITHUB_URL = 'https://github.com/gh-pixel-95813/MacMate';
const LICENSE_NAME = 'MIT';

// 主题选项:跟随系统 / 浅色 / 暗色,与 store.theme 对齐。
const themeOptions: Array<{ value: Theme; labelKey: string }> = [
  { value: 'system', labelKey: 'theme.system' },
  { value: 'light', labelKey: 'theme.light' },
  { value: 'dark', labelKey: 'theme.dark' },
];
</script>

<template>
  <div class="p-6">
    <h1 class="text-2xl font-bold text-gray-900 dark:text-gray-100">
      {{ t('page.settings') }}
    </h1>

    <!-- 分区 1:通用 -->
    <section class="mt-6">
      <h2 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
        {{ t('settings.general') }}
      </h2>
      <div
        class="mt-3 space-y-4 rounded-2xl border border-gray-200/60 bg-white/70 p-5 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/70"
      >
        <!-- 语言切换 -->
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
              {{ t('settings.language') }}
            </h3>
          </div>
          <select
            :value="appStore.locale"
            class="rounded-md border border-gray-300 bg-white px-3 py-1 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-zinc-700 dark:bg-zinc-800 dark:text-gray-100"
            @change="onLocaleChange"
          >
            <option value="zh-CN">简体中文</option>
            <option value="en-US">English</option>
          </select>
        </div>
        <!-- 主题切换 -->
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
              {{ t('settings.theme') }}
            </h3>
          </div>
          <div class="flex overflow-hidden rounded-md border border-gray-300 dark:border-zinc-700">
            <button
              v-for="opt in themeOptions"
              :key="opt.value"
              type="button"
              class="px-3 py-1 text-xs font-medium transition-colors"
              :class="
                appStore.theme === opt.value
                  ? 'bg-blue-500 text-white'
                  : 'bg-white text-gray-600 hover:bg-gray-100 dark:bg-zinc-800 dark:text-gray-300 dark:hover:bg-zinc-700'
              "
              @click="onThemeChange(opt.value)"
            >
              {{ t(opt.labelKey) }}
            </button>
          </div>
        </div>
      </div>
    </section>

    <!-- 分区 2:清理(深度清理开关 + 大文件阈值) -->
    <section class="mt-6">
      <h2 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
        {{ t('settings.cleaning') }}
      </h2>
      <div
        class="mt-3 space-y-4 rounded-2xl border border-gray-200/60 bg-white/70 p-5 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/70"
      >
        <!-- 深度清理开关(Task 8,保留) -->
        <div class="flex items-center gap-4">
          <div class="min-w-0 flex-1">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
              {{ t('settings.deepClean') }}
            </h3>
            <p class="mt-1 text-xs text-gray-500 dark:text-zinc-400">
              {{ deepCleanOn ? t('settings.deepCleanOn') : t('settings.deepCleanOff') }}
            </p>
          </div>
          <button
            type="button"
            role="switch"
            :aria-checked="deepCleanOn"
            class="relative inline-flex h-6 w-11 shrink-0 cursor-pointer items-center rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 dark:focus:ring-offset-zinc-900"
            :class="deepCleanOn ? 'bg-red-600' : 'bg-gray-300 dark:bg-zinc-600'"
            @click="toggleDeepClean"
          >
            <span
              class="inline-block h-5 w-5 transform rounded-full bg-white shadow transition-transform"
              :class="deepCleanOn ? 'translate-x-5' : 'translate-x-0.5'"
            />
          </button>
        </div>
        <div
          v-if="deepCleanOn"
          class="rounded-lg border border-amber-300 bg-amber-50 p-3 text-sm text-amber-800 dark:border-amber-800 dark:bg-amber-950/40 dark:text-amber-300"
        >
          {{ t('settings.deepCleanWarning') }}
        </div>

        <!-- 大文件扫描阈值 -->
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0">
            <h3 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
              {{ t('settings.largeFileThreshold') }}
            </h3>
            <p class="mt-1 text-xs text-gray-500 dark:text-zinc-400">
              {{ t('settings.largeFileThresholdHint') }}
            </p>
          </div>
          <input
            type="number"
            min="1"
            :value="appStore.largeFileThresholdMb"
            class="w-24 rounded-md border border-gray-300 bg-white px-3 py-1 text-right text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-zinc-700 dark:bg-zinc-800 dark:text-gray-100"
            @change="onThresholdChange"
          />
        </div>
      </div>
    </section>

    <!-- 分区 3:数据 -->
    <section class="mt-6">
      <h2 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
        {{ t('settings.data') }}
      </h2>
      <div
        class="mt-3 space-y-3 rounded-2xl border border-gray-200/60 bg-white/70 p-5 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/70"
      >
        <div class="flex items-center justify-between gap-4">
          <span class="text-sm text-gray-700 dark:text-gray-300">
            {{ t('settings.clearHistory') }}
          </span>
          <button
            type="button"
            :disabled="clearingHistory"
            class="rounded-md border border-gray-300 px-3 py-1 text-xs font-medium text-gray-600 transition-colors hover:bg-gray-100 disabled:opacity-50 dark:border-zinc-700 dark:text-gray-300 dark:hover:bg-zinc-800"
            @click="onClearHistory"
          >
            {{ clearingHistory ? '…' : t('dashboard.clearHistory') }}
          </button>
        </div>
        <p v-if="clearHistoryMsg" class="text-xs text-gray-500 dark:text-gray-400">
          {{ clearHistoryMsg }}
        </p>

        <div class="flex items-center justify-between gap-4">
          <span class="text-sm text-gray-700 dark:text-gray-300">
            {{ t('settings.openConfigDir') }}
          </span>
          <button
            type="button"
            class="rounded-md border border-gray-300 px-3 py-1 text-xs font-medium text-gray-600 transition-colors hover:bg-gray-100 dark:border-zinc-700 dark:text-gray-300 dark:hover:bg-zinc-800"
            @click="onOpenConfigDir"
          >
            {{ t('settings.openConfigDir') }}
          </button>
        </div>
        <p v-if="openDirMsg" class="text-xs text-gray-500 dark:text-gray-400">
          {{ openDirMsg }}
        </p>
      </div>
    </section>

    <!-- 分区 4:日志与上报(Task 11) -->
    <section class="mt-6">
      <h2 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
        {{ t('settings.loggingAndReport') }}
      </h2>
      <div
        class="mt-3 space-y-4 rounded-2xl border border-gray-200/60 bg-white/70 p-5 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/70"
      >
        <!-- 状态提示 -->
        <div
          class="rounded-lg border border-blue-200 bg-blue-50 p-3 text-xs text-blue-800 dark:border-blue-800 dark:bg-blue-950/40 dark:text-blue-300"
        >
          <p>
            {{ t('settings.reportIntro') }}
          </p>
          <p v-if="tokenStatus" class="mt-1">
            <span v-if="tokenStatus.configured">{{ t('settings.tokenConfigured') }}</span>
            <span v-else>{{ t('settings.tokenNotConfigured') }}</span>
            <span class="ml-1 text-gray-500 dark:text-gray-400">
              ({{ tokenStatus.targetRepo }})
            </span>
          </p>
        </div>

        <!-- PAT 输入 -->
        <div class="space-y-2">
          <label
            for="pat-input"
            class="block text-xs font-semibold text-gray-700 dark:text-gray-300"
          >
            {{ t('settings.tokenLabel') }}
          </label>
          <input
            id="pat-input"
            v-model="tokenInput"
            type="password"
            autocomplete="off"
            :placeholder="t('settings.tokenPlaceholder')"
            class="w-full rounded-md border border-gray-300 bg-white px-3 py-1.5 text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-zinc-700 dark:bg-zinc-800 dark:text-gray-100"
          />
          <div class="flex gap-2">
            <button
              type="button"
              class="rounded-md border border-gray-300 px-3 py-1 text-xs font-medium text-gray-600 transition-colors hover:bg-gray-100 dark:border-zinc-700 dark:text-gray-300 dark:hover:bg-zinc-800"
              @click="onSaveToken"
            >
              {{ t('settings.tokenSave') }}
            </button>
            <button
              v-if="tokenStatus?.configured"
              type="button"
              class="rounded-md border border-gray-300 px-3 py-1 text-xs font-medium text-red-600 transition-colors hover:bg-red-50 dark:border-red-800 dark:text-red-400 dark:hover:bg-red-950/40"
              @click="onClearToken"
            >
              {{ t('settings.tokenClear') }}
            </button>
          </div>
          <p v-if="tokenMsg" class="text-xs text-gray-500 dark:text-gray-400">
            {{ tokenMsg }}
          </p>
        </div>

        <!-- 日志上报 -->
        <div class="space-y-2 border-t border-gray-200/60 pt-3 dark:border-zinc-800/60">
          <label
            for="days-input"
            class="block text-xs font-semibold text-gray-700 dark:text-gray-300"
          >
            {{ t('settings.reportDaysLabel') }}
          </label>
          <div class="flex items-center gap-3">
            <input
              id="days-input"
              v-model.number="submitDays"
              type="number"
              min="1"
              max="30"
              class="w-24 rounded-md border border-gray-300 bg-white px-3 py-1.5 text-right text-sm text-gray-900 focus:outline-none focus:ring-2 focus:ring-blue-500 dark:border-zinc-700 dark:bg-zinc-800 dark:text-gray-100"
            />
            <button
              type="button"
              :disabled="submitting || !tokenStatus?.configured"
              class="rounded-md border border-blue-500 bg-blue-500 px-4 py-1.5 text-xs font-medium text-white transition-colors hover:bg-blue-600 disabled:cursor-not-allowed disabled:opacity-50"
              @click="onSubmitLogs"
            >
              {{ submitting ? '…' : t('settings.reportSubmit') }}
            </button>
          </div>
          <p v-if="submitMsg" class="break-all text-xs text-gray-500 dark:text-gray-400">
            {{ submitMsg }}
          </p>
          <p class="text-xs text-gray-400 dark:text-gray-500">
            {{ t('settings.reportPrivacy') }}
          </p>
        </div>
      </div>
    </section>

    <!-- 分区 5:关于 -->
    <section class="mt-6">
      <h2 class="text-sm font-semibold text-gray-900 dark:text-gray-100">
        {{ t('settings.about') }}
      </h2>
      <div
        class="mt-3 space-y-3 rounded-2xl border border-gray-200/60 bg-white/70 p-5 backdrop-blur-xl dark:border-zinc-800/60 dark:bg-zinc-900/70"
      >
        <div class="flex items-center justify-between gap-4">
          <span class="text-sm text-gray-700 dark:text-gray-300">{{ t('settings.version') }}</span>
          <span class="text-sm font-semibold text-gray-900 dark:text-gray-100">
            v{{ appStore.appVersion }}
          </span>
        </div>
        <div class="flex items-center justify-between gap-4">
          <span class="text-sm text-gray-700 dark:text-gray-300">
            {{ t('settings.githubRepo') }}
          </span>
          <a
            :href="GITHUB_URL"
            target="_blank"
            rel="noopener noreferrer"
            class="text-sm font-medium text-blue-600 hover:underline dark:text-blue-400"
          >
            {{ GITHUB_URL }}
          </a>
        </div>
        <div class="flex items-center justify-between gap-4">
          <span class="text-sm text-gray-700 dark:text-gray-300">{{ t('settings.license') }}</span>
          <span class="text-sm font-semibold text-gray-900 dark:text-gray-100">
            {{ LICENSE_NAME }}
          </span>
        </div>
      </div>
    </section>
  </div>
</template>
