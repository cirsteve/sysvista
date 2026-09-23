import { defineConfig } from 'vitest/config'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  cacheDir: '.vite',
  plugins: [react(), tailwindcss()],
  test: {
    projects: [
      { test: { name: 'unit', include: ['src/**/*.test.ts'], environment: 'node' } },
      { test: { name: 'dom', include: ['src/**/*.dom.test.tsx'], environment: 'jsdom', setupFiles: ['./src/test/setup.dom.ts'] } },
    ],
  },
})
