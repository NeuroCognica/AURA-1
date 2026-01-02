import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  base: './',
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 5170,       // CHANGED: Moved to 5170 to avoid conflict with Frontend (5173)
    strictPort: true, 
    host: true 
  },
  build: {
    outDir: 'dist',
    emptyOutDir: true,
  }
})
