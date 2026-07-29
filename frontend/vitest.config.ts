import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '~': resolve(__dirname),
      '@': resolve(__dirname),
      '~~': resolve(__dirname),
      '#imports': resolve(__dirname, 'tests/mocks/imports.ts'),
      // h3 is not a direct dependency — it ships with Nuxt/Nitro.
      // vitest runs outside the Nitro runtime, so we resolve it explicitly
      // from the pnpm virtual store so server utility tests can import it.
      'h3': resolve(__dirname, 'node_modules/.pnpm/h3@1.15.11/node_modules/h3'),
    },
  },
  test: {
    globals: true,
    environment: 'jsdom',
    include: ['tests/unit/**/*.test.ts', 'tests/unit/**/*.spec.ts'],
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
