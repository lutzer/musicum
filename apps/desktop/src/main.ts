import './base';
import './shell';
import './views/welcome-view';
import './views/file-list-view';
import './views/file-detail-view';
import './views/clips-view';
import { viewRegistry } from './plugin-api/registry';
import { coreApi } from './core-api';
import { loadPlugins } from './plugin-loader/loader';
import { convertFileSrc } from './plugin-loader/convert-fs-src';
import { router } from './shell/router';

viewRegistry.register('__built_in__', {
  id: 'welcome',
  title: 'Welcome',
  element: 'mus-welcome-view',
});
viewRegistry.register('__built_in__', {
  id: 'files',
  title: 'Files',
  element: 'mus-file-list-view',
});
viewRegistry.register('__built_in__', {
  id: 'files-detail',
  title: 'File',
  element: 'mus-file-detail-view',
  sidebar: false,
});
viewRegistry.register('__built_in__', {
  id: 'clips',
  title: 'Clips',
  element: 'mus-clips-view',
});

router.ensureDefault();

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
