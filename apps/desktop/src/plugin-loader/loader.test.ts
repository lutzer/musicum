import { describe, it, expect, beforeEach, vi } from 'vitest';
import { loadPlugins, SUPPORTED_API_VERSION } from './loader';
import { viewRegistry, slotRegistry } from '../plugin-api/registry';
import type { PluginManifest, PluginModule } from '../plugin-api/types';

const okManifest = (over: Partial<PluginManifest> = {}): PluginManifest => ({
  id: 'test-bundle',
  name: 'Test',
  version: '0.1.0',
  entryPath: '/plugins/test/index.js',
  source: 'user',
  apiVersion: SUPPORTED_API_VERSION,
  ...over,
});

describe('loadPlugins', () => {
  beforeEach(() => {
    viewRegistry.unregisterBundle('test-bundle');
    viewRegistry.unregisterBundle('good');
    viewRegistry.unregisterBundle('bad');
    slotRegistry.unregisterBundle('test-bundle');
  });

  it('imports each manifest and calls register with a context', async () => {
    const registerFn = vi.fn((ctx) => {
      ctx.registerView({ id: 'test.a', title: 'A', element: 'x-a' });
      ctx.registerSlot('slot-x', { id: 'e1', element: 'x-e1' });
    });
    const importer = vi.fn(async () => ({ default: { register: registerFn } as PluginModule }));

    await loadPlugins({
      manifests: [okManifest()],
      importer,
      convertPath: p => p,
      coreApi: {
        getAppInfo: async () => ({ name: '', version: '' }),
        getLibraryDir: async () => '',
        listPlugins: async () => [],
        listFiles: async () => [],
        listClips: async () => [],
        getFileBySlug: async () => ({} as any),
        getFileMetadata: async () => null,
        setFileNotes: async () => {},
        setFileTags: async () => {},
      },
    });

    expect(importer).toHaveBeenCalledWith('/plugins/test/index.js');
    expect(registerFn).toHaveBeenCalledOnce();
    expect(viewRegistry.get('test.a')?.title).toBe('A');
    expect(slotRegistry.entries('slot-x')).toHaveLength(1);
  });

  it('skips bundles with mismatched apiVersion', async () => {
    const importer = vi.fn();
    await loadPlugins({
      manifests: [okManifest({ apiVersion: 999 })],
      importer,
      convertPath: p => p,
      coreApi: {} as any,
    });
    expect(importer).not.toHaveBeenCalled();
  });

  it('continues after a failing bundle', async () => {
    const bad = okManifest({ id: 'bad', entryPath: '/bad.js' });
    const good = okManifest({ id: 'good', entryPath: '/good.js' });
    const importer = vi.fn(async (path: string) => {
      if (path === '/bad.js') throw new Error('boom');
      return { default: { register: (ctx: any) => ctx.registerView({ id: 'good.v', title: 'G', element: 'x-g' }) } };
    });
    await loadPlugins({
      manifests: [bad, good],
      importer,
      convertPath: p => p,
      coreApi: {} as any,
    });
    expect(viewRegistry.get('good.v')?.title).toBe('G');
  });

  it('skips modules without a default export', async () => {
    const importer = vi.fn(async () => ({}));
    await loadPlugins({
      manifests: [okManifest()],
      importer,
      convertPath: p => p,
      coreApi: {} as any,
    });
    // no throw is the assertion
    expect(true).toBe(true);
  });
});
