import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import tailwindcss from '@tailwindcss/vite';

export default defineConfig({
  plugins: [tailwindcss(), svelte()],
  build: { outDir: 'dist', emptyOutDir: true, target: 'es2022' },
  server: {
    proxy: { '/api': { target: 'http://127.0.0.1:8080', changeOrigin: false } },
  },
});
