import '@tauri-apps/api';

declare global {
  interface Window {
    __TAURI__: typeof import('@tauri-apps/api');
  }
}