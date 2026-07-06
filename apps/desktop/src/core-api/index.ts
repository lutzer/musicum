import { invoke } from '@tauri-apps/api/core';
import type { AppInfo, PluginManifest } from './types';
import type { CoreApi } from '../plugin-api/types';

export const coreApi: CoreApi = {
  getAppInfo: () => invoke<AppInfo>('get_app_info'),
  listPlugins: () => invoke<PluginManifest[]>('list_plugins'),
};
