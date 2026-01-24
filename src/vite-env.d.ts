/// <reference types="vite/client" />

declare module '*.vue' {
  import type { DefineComponent } from 'vue';
  // biome-ignore lint/complexity/noBannedTypes: <no reason>
  // biome-ignore lint/suspicious/noExplicitAny: <no reason>
  const component: DefineComponent<{}, {}, any>;
  export default component;
}
