import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vite';

// The mod is loaded from mod://localhost/<id>/, so every asset URL must be relative.
// `public/` (overlay.json, locales/) is copied into dist/ unchanged.
export default defineConfig({
  base: './',
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    rollupOptions: {
      input: {
        panel: fileURLToPath(new URL('./index.html', import.meta.url)),
        desktop: fileURLToPath(new URL('./desktop.html', import.meta.url)),
      },
    },
  },
});
