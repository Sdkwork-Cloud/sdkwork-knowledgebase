import { test, expect } from '@playwright/test';

import {
  setupKnowledgebaseE2ePage,
} from './knowledgeApiMocks';

test.describe('Knowledgebase PC shell', () => {
  test('redirects unauthenticated visitors to the login route', async ({ page }) => {
    await page.route('**/app/v3/api/**', async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({}),
      });
    });

    await page.goto('/');
    await expect(page).toHaveURL(/\/auth\/login/);
    await expect(page.getByTestId('knowledgebase-pc-auth-shell')).toBeVisible();
  });

  test('loads the authenticated workspace shell with SDK-backed knowledge APIs mocked', async ({
    page,
  }) => {
    const telemetry = await setupKnowledgebaseE2ePage(page);

    await page.goto('/');
    await expect(page).toHaveURL('/');

    const shell = page.getByTestId('knowledgebase-pc-app-shell');
    await expect(shell).toBeVisible({ timeout: 30_000 });
    await expect(page.getByTitle('My Knowledge Base')).toBeVisible();
    await expect.poll(() => telemetry.requestedPaths).toContain(
      'GET /app/v3/api/knowledge/spaces/1',
    );
    await expect(page.getByRole('heading', { name: 'E2E Knowledge Base' })).toBeVisible({
      timeout: 15_000,
    });
  });
});
