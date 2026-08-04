import './base';
import './shell';
import './views/welcome-view';
import './views/files-view';
import './views/clips-view';
import { viewRegistry } from './plugin-api/registry';
import { coreApi } from './core-api';
import { loadPlugins } from './plugin-loader/loader';
import { convertFileSrc } from './plugin-loader/convert-fs-src';

viewRegistry.register('__built_in__', {
  id: 'welcome',
  title: 'Welcome',
  element: 'mus-welcome-view',
});
viewRegistry.register('__built_in__', {
  id: 'files',
  title: 'Files',
  element: 'mus-files-view',
});
viewRegistry.register('__built_in__', {
  id: 'clips',
  title: 'Clips',
  element: 'mus-clips-view',
});

if (!window.location.hash) {
  window.location.hash = '#welcome';
}

async function bootstrap() {
  try {
    const manifests = await coreApi.listPlugins();
    await loadPlugins({
      manifests,
      importer: (url) => import(/* @vite-ignore */ url),
      convertPath: convertFileSrc,
      coreApi,
    });
  } catch (e) {
    console.error('[musicum] plugin loading failed', e);
  }
}

bootstrap();
