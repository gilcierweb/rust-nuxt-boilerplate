import { defineConfig, devices } from '@playwright/test'

// The e2e suite runs against the node-server build, the same artifact CI
// serves: CI starts `node .output/server/index.mjs` on port 3000 and lets
// the webServer stay undefined. Locally the same command runs here, with the
// port overridable, because the default 3000 is often held by another
// project of the same workspace and a reused foreign server would silently
// test the wrong app.
const e2ePort = process.env.E2E_PORT ?? '3000'
const e2eBaseURL = process.env.E2E_BASE_URL ?? `http://localhost:${e2ePort}`

export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 0 : 0,
  workers: process.env.CI ? 4 : undefined,
  reporter: process.env.CI ? [['html'], ['list']] : 'html',
  use: {
    baseURL: e2eBaseURL,
    trace: process.env.CI ? 'retain-on-failure' : 'on-first-retry',
    video: false,
    screenshot: 'only-on-failure',
    actionTimeout: 10_000,
    navigationTimeout: 15_000,
  },

  webServer: process.env.CI
    ? undefined
    : {
        command: `NITRO_PRESET=node-server pnpm run build && HOST=127.0.0.1 PORT=${e2ePort} node .output/server/index.mjs`,
        port: Number(e2ePort),
        timeout: 180_000,
        reuseExistingServer: true,
      },

  projects: process.env.CI
    ? [
        {
          name: 'chromium',
          use: { ...devices['Desktop Chrome'] },
        },
      ]
    : [
        {
          name: 'chromium',
          use: { ...devices['Desktop Chrome'] },
        },
        {
          name: 'firefox',
          use: { ...devices['Desktop Firefox'] },
        },
        {
          name: 'safari',
          use: { ...devices['Desktop Safari'] },
        },
      ],
})
