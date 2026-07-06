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
  listPlugins(): Promise<PluginManifest[]>;
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
