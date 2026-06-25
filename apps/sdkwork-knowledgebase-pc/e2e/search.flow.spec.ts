import { test, expect } from '@playwright/test';

import {
  E2E_SOURCE_DOCUMENT,
  setupKnowledgebaseE2ePage,
} from './knowledgeApiMocks';

test.describe('Knowledgebase PC search flow', () => {
  test('returns cited local results and navigates to the source document', async ({ page }) => {
    await setupKnowledgebaseE2ePage(page);

    await page.goto('/');
    await expect(page.getByTestId('knowledgebase-pc-app-shell')).toBeVisible({ timeout: 30_000 });

    await page.getByTestId('knowledgebase-pc-nav-search').click();
    await expect(page.getByText('What would you like to search today?')).toBeVisible();

    const query = 'launch readiness verification';
    await page
      .getByPlaceholder('Ask anything. Press Enter to send, Shift+Enter for a new line...')
      .fill(query);
    await page.getByTitle('Send').click();

    await expect(page.getByText(E2E_SOURCE_DOCUMENT.title)).toBeVisible({ timeout: 30_000 });

    await page.getByTestId('search-sources-toggle').click();
    await page.getByTestId('search-source-row-doc').click();

    await expect(page.getByText(E2E_SOURCE_DOCUMENT.title)).toBeVisible({ timeout: 15_000 });
  });
});
