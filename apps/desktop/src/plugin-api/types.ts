export type PluginSource = 'bundled' | 'user';

export interface PluginManifest {
  id: string;
  name: string;
  version: string;
  description?: string;
  entryPath: string;
  source: PluginSource;
  apiVersion: number;
}

export interface FileModel {
  id: string;
  slug: string;
  name: string;
  path: string;
  duration: number;
  sample_rate: number;
  channels: number;
  mime_type: string;
  hash: string;
  mtime: string;
  size_bytes: number;
  created_at: string;
  updated_at: string;
}

export interface ClipModel {
  id: string;
  slug: string;
  file_id: string;
  title: string;
  processors: unknown;
  duration: number | null;
  notes: string;
  created_at: string;
  updated_at: string;
}

export interface FileListItem { file: FileModel; clips: ClipModel[]; }
export interface ClipListItem { clip: ClipModel; file: FileModel; }

export interface ViewDescriptor {
  id: string;
  title: string;
  icon?: string;
  element: string;
}

export interface SlotEntry {
  id: string;
  element: string;
  order?: number;
}

export interface RegisteredView extends ViewDescriptor {
  bundleId: string;
}

export interface RegisteredSlotEntry extends SlotEntry {
  bundleId: string;
  slotId: string;
}

export interface CoreApi {
  getAppInfo(): Promise<{ name: string; version: string }>;
  getLibraryDir(): Promise<string>;
  listPlugins(): Promise<PluginManifest[]>;
  listFiles(): Promise<FileListItem[]>;
  listClips(): Promise<ClipListItem[]>;
}

export interface PluginContext {
  manifest: PluginManifest;
  registerView(view: ViewDescriptor): void;
  registerSlot(slotId: string, entry: SlotEntry): void;
  coreApi: CoreApi;
}

export interface PluginModule {
  register(ctx: PluginContext): void | Promise<void>;
}
