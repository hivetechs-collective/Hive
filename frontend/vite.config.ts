import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'path';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5173,
    strictPort: true,
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: true,
    minify: process.env.TAURI_ENV_DEBUG ? false : 'esbuild',
    target: process.env.TAURI_ENV_PLATFORM === 'windows' 
      ? 'chrome105' 
      : 'safari13',
  },
});