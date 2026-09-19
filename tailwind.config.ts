import type { Config } from 'tailwindcss';

// Tailwind CSS 配置:暗色模式采用 class 策略
export default {
  darkMode: 'class',
  content: ['./index.html', './src/**/*.{vue,ts,tsx}'],
  theme: {
    extend: {},
  },
  plugins: [],
} satisfies Config;
