import { invoke } from '@tauri-apps/api/core';
import type {
  AppInfo,
  ClipListItem,
  FileListItem,
  FileMetadataModel,
  PluginManifest,
} from './types';
import type { CoreApi } from '../plugin-api/types';

export const coreApi: CoreApi = {
  getAppInfo: () => invoke<AppInfo>('get_app_info'),
  getLibraryDir: () => invoke<string>('get_library_dir'),
  listPlugins: () => invoke<PluginManifest[]>('list_plugins'),
  listFiles: () => invoke<FileListItem[]>('list_files'),
  listClips: () => invoke<ClipListItem[]>('list_clips'),
  getFileBySlug: (slug) =>
    invoke<FileListItem>('get_file_with_clips_by_slug', { slug }),
  getFileMetadata: (fileId) =>
    invoke<FileMetadataModel | null>('get_file_metadata', { fileId }),
  setFileNotes: (slug, notes) =>
    invoke<void>('set_file_notes', { slug, notes }),
  setFileTags: (slug, tags) =>
    invoke<void>('set_file_tags', { slug, tags }),
};
