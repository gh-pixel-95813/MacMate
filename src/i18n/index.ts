import { createI18n } from 'vue-i18n';
import zhCN from './zh-CN.json';
import enUS from './en-US.json';

// 支持的 locale 列表
const supportedLocales = ['zh-CN', 'en-US'] as const;
type SupportedLocale = (typeof supportedLocales)[number];

// 默认 locale 跟随浏览器语言,不在支持列表内则回退至 en-US
const browserLocale = (typeof navigator !== 'undefined' ? navigator.language : 'en-US') as string;
const defaultLocale: SupportedLocale = supportedLocales.includes(browserLocale as SupportedLocale)
  ? (browserLocale as SupportedLocale)
  : 'en-US';

export const i18n = createI18n({
  legacy: false,
  locale: defaultLocale,
  fallbackLocale: 'en-US',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
});
