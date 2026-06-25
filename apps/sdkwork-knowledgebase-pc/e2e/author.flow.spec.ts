import { test, expect } from '@playwright/test';

import {
  createE2eMockTelemetry,
  setupKnowledgebaseE2ePage,
} from './knowledgeApiMocks';

test.describe('Knowledgebase PC author flow', () => {
  test('creates a note, edits content, and auto-saves through ingest', async ({ page }) => {
    const telemetry = createE2eMockTelemetry();
    await setupKnowledgebaseE2ePage(page, telemetry);

    await page.goto('/');
    await expect(page.getByTestId('knowledgebase-pc-app-shell')).toBeVisible({ timeout: 30_000 });

    await page.getByTestId('knowledgebase-pc-add-menu-trigger').click();
    await page.getByRole('menuitem', { name: 'New Note' }).click();

    const editor = page.locator('.ProseMirror').first();
    await expect(editor).toBeVisible({ timeout: 15_000 });

    const draftText = 'Launch readiness author flow auto-save verification.';
    await editor.click();
    await editor.fill(draftText);

    await expect
      .poll(() => telemetry.ingestPayloads.some((payload) => payload.includes(draftText)), {
        timeout: 15_000,
        message: 'expected debounced document save to call ingest with edited content',
      })
      .toBe(true);

    expect(telemetry.createdDocumentIds.length).toBeGreaterThan(0);
  });
});
