import { defineConfig } from 'vite';
import { resolve } from 'node:path';

export default defineConfig({
  clearScreen: false,
  server: { port: 5173, strictPort: true },
  build: {
    target: 'es2022',
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        main: resolve(__dirname, 'index.html'),
        'vendor-lit': resolve(__dirname, 'node_modules/lit/index.js'),
        'vendor-lit-decorators': resolve(__dirname, 'node_modules/lit/decorators.js'),
        'vendor-plugin-api': resolve(__dirname, 'src/vendor/plugin-api-entry.ts'),
      },
      output: {
        entryFileNames: (chunk) => (chunk.name.startsWith('vendor-')
          ? `vendor/${chunk.name.replace(/^vendor-/, '')}.js`
          : 'assets/[name]-[hash].js'),
        chunkFileNames: 'assets/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash][extname]',
      },
    },
  },
});
