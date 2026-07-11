import { readFileSync } from 'node:fs';
import React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import { describe, expect, it, vi } from 'vitest';
import { KnowledgebaseErrorCodes } from 'sdkwork-knowledgebase-pc-core';

import { McpConsolePanel } from '../components/McpConsolePanel';
import { McpAgentService } from './mcpAgent';

vi.mock('react-i18next', () => ({
  useTranslation: () => ({
    t: (key: string) => key,
  }),
}));

function readSource(relativePath: string): string {
  return readFileSync(new URL(relativePath, import.meta.url), 'utf8');
}

describe('MCP agent execution boundary', () => {
  it('rejects with a typed unavailable error when no composed SDK execution surface exists', async () => {
    const operation = Promise.resolve().then(() =>
      McpAgentService.processUserQuery('Apply a layout tool'),
    );

    await expect(operation).rejects.toMatchObject({
      code: KnowledgebaseErrorCodes.API_UNAVAILABLE_SDK,
    });
  });

  it('renders the MCP console as unavailable without active tools or fake connections', () => {
    const markup = renderToStaticMarkup(
      React.createElement(McpConsolePanel, {
        isOpen: true,
        onToggle: vi.fn(),
        isTyping: false,
        onTriggerQuickTool: vi.fn(),
      }),
    );

    expect(markup).toContain('errors:api.unavailable.sdk');
    expect(markup).not.toContain('activeStatus');
    expect(markup).not.toContain('generate_headlines');
    expect(markup).not.toContain('connectionEstablished');
  });

  it('contains no local rule engine or fabricated tool success', () => {
    const source = readSource('./mcpAgent.ts');

    expect(source).not.toMatch(
      /generate_headlines|insert_article_block|mock styling injector|status:\s*'success'/i,
    );
    expect(source).toContain('KnowledgebaseErrorCodes.API_UNAVAILABLE_SDK');
  });

  it('makes the direct UI consumer surface real errors without timer simulations', () => {
    const source = readSource('../AiAssistantPanel.tsx');

    expect(source).toContain('resolveUserFacingErrorMessage');
    expect(source).toContain('McpAgentService.processUserQuery');
    expect(source).not.toMatch(
      /shouldUseKnowledgebaseDemoFallback|status:\s*'success'|<insert_to_note>|setTimeout|setInterval/,
    );
  });
});
