import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '~': resolve(__dirname),
      '@': resolve(__dirname),
      '#imports': resolve(__dirname, 'tests/mocks/imports.ts'),
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    include: ['app/**/*.spec.ts', 'app/**/*.test.ts'],
    setupFiles: './tests/setup.ts',
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      include: ['app/**/*.ts', 'app/**/*.vue'],
      exclude: [
        'node_modules/',
        'dist/',
        '.output/',
        '**/*.config.*',
        'types/**',
        'tests/**',
        'app/**/*.d.ts',
        'app/plugins/**',
        'app/middleware/**',
        'app/layouts/**',
        'app/pages/**',
      ],
      thresholds: {
        statements: 60,
        branches: 60,
        functions: 60,
        lines: 60,
      },
    },
  },
})
