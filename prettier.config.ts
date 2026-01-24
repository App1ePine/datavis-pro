// prettier.config.ts

import type { Config } from 'prettier';

const config: Config = {
  semi: true,
  singleQuote: true, // true 为使用单引号
  useTabs: false, // false 为使用 space
  tabWidth: 2, // 每个 tab 为 2 个 space
  printWidth: 120, // 每行最多 120 个字符
  arrowParens: 'always', // always 为箭头函数参数总是添加括号
  endOfLine: 'lf', // lf 为尾部换行
  bracketSpacing: true, // true 为在对象字面量的括号之间添加空格
  trailingComma: 'es5',
};

export default config;
