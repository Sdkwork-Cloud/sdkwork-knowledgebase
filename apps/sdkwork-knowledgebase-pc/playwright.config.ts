import { defineConfig, devices } from '@playwright/test';

const previewPort = 5185;

export default defineConfig({
  testDir: './e2e',
  testMatch: /\.flow\.spec\.ts$|shell\.smoke\.spec\.ts/,
  fullyParallel: false,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: 'list',
  timeout: 60_000,
  use: {
    baseURL: `http://127.0.0.1:${previewPort}`,
    trace: 'on-first-retry',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
  webServer: {
    command: `pnpm exec vite build --mode playwright && pnpm exec vite preview --mode playwright --port ${previewPort} --strictPort --host 127.0.0.1`,
    url: `http://127.0.0.1:${previewPort}`,
    reuseExistingServer: !process.env.CI,
    timeout: 180_000,
  },
});
