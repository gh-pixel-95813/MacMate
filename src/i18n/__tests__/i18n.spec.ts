import { describe, expect, it } from 'vitest';
import { createI18n } from 'vue-i18n';
import zhCN from '@/i18n/zh-CN.json';
import enUS from '@/i18n/en-US.json';

// 核心导航 / 页面 / 通用键:两种语言文件必须结构一致。
const navKeys = [
  'dashboard',
  'systemJunk',
  'uninstaller',
  'largeFiles',
  'privacy',
  'settings',
] as const;
const commonKeys = ['scan', 'clean', 'cancel', 'confirm', 'total', 'size', 'path'] as const;

describe('i18n message files', () => {
  it('zh-CN contains all nav/page/common core keys', () => {
    navKeys.forEach((k) => {
      expect(zhCN.nav[k]).toBeTruthy();
      expect(zhCN.page[k]).toBeTruthy();
    });
    commonKeys.forEach((k) => {
      expect(zhCN.common[k]).toBeTruthy();
    });
  });

  it('en-US contains all nav/page/common core keys', () => {
    navKeys.forEach((k) => {
      expect(enUS.nav[k]).toBeTruthy();
      expect(enUS.page[k]).toBeTruthy();
    });
    commonKeys.forEach((k) => {
      expect(enUS.common[k]).toBeTruthy();
    });
  });

  it('returns the localized nav.dashboard after switching locale', () => {
    const i18n = createI18n({
      legacy: false,
      locale: 'zh-CN',
      fallbackLocale: 'en-US',
      messages: { 'zh-CN': zhCN, 'en-US': enUS },
    });
    expect(i18n.global.t('nav.dashboard')).toBe('仪表盘');
    // composition 模式下 locale 为 WritableComputedRef,需通过 .value 切换。
    i18n.global.locale.value = 'en-US';
    expect(i18n.global.t('nav.dashboard')).toBe('Dashboard');
  });
});
