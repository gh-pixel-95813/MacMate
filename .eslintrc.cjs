/* eslint-env node */
// ESLint 配置:集成 eslint-plugin-vue 与 TypeScript,放宽占位规则
module.exports = {
  root: true,
  env: {
    node: true,
    browser: true,
    es2022: true,
  },
  extends: [
    'plugin:vue/vue3-essential',
    'eslint:recommended',
    '@vue/eslint-config-typescript',
    '@vue/eslint-config-prettier',
  ],
  parserOptions: {
    ecmaVersion: 'latest',
  },
  rules: {
    'no-unused-vars': 'off',
    'vue/multi-word-component-names': 'off',
  },
  ignorePatterns: ['dist/', 'src-tauri/', 'node_modules/', 'coverage/'],
};
