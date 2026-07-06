import { viewRegistry, slotRegistry } from '../plugin-api/registry';
import type {
  CoreApi,
  PluginContext,
  PluginManifest,
  PluginModule,
} from '../plugin-api/types';

export const SUPPORTED_API_VERSION = 1;

export interface LoadPluginsOpts {
  manifests: PluginManifest[];
  importer: (url: string) => Promise<unknown>;
  convertPath: (path: string) => string;
  coreApi: CoreApi;
}

export async function loadPlugins(opts: LoadPluginsOpts): Promise<void> {
  for (const manifest of opts.manifests) {
    if (manifest.apiVersion !== SUPPORTED_API_VERSION) {
      console.warn(
        `[musicum] plugin "${manifest.id}" apiVersion ${manifest.apiVersion} ` +
        `does not match host ${SUPPORTED_API_VERSION}; skipping`,
      );
      continue;
    }
    const url = opts.convertPath(manifest.entryPath);
    try {
      const mod = (await opts.importer(url)) as { default?: PluginModule };
      const impl = mod.default;
      if (!impl || typeof impl.register !== 'function') {
        console.error(`[musicum] plugin "${manifest.id}" has no default export with register()`);
        continue;
      }
      const ctx: PluginContext = {
        manifest,
        registerView: view => viewRegistry.register(manifest.id, view),
        registerSlot: (slotId, entry) => slotRegistry.register(manifest.id, slotId, entry),
        coreApi: opts.coreApi,
      };
      await impl.register(ctx);
    } catch (e) {
      console.error(`[musicum] plugin "${manifest.id}" failed to load:`, e);
    }
  }
}
