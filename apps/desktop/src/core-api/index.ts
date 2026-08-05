import { invoke } from '@tauri-apps/api/core';
import type {
  AppInfo,
  ClipListItem,
  FileListItem,
  PluginManifest,
} from './types';
import type { CoreApi } from '../plugin-api/types';

export const coreApi: CoreApi = {
  getAppInfo: () => invoke<AppInfo>('get_app_info'),
  getLibraryDir: () => invoke<string>('get_library_dir'),
  listPlugins: () => invoke<PluginManifest[]>('list_plugins'),
  listFiles: () => invoke<FileListItem[]>('list_files'),
  listClips: () => invoke<ClipListItem[]>('list_clips'),
};
