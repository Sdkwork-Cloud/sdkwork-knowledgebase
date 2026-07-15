import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  KnowledgebaseErrorCodes,
  setKnowledgebaseApiEnabled,
} from 'sdkwork-knowledgebase-pc-core';

const {
  buildEditorActionPrompt,
  sendKnowledgeAgentMessage,
  synthesizeKnowledgeSearchAnswer,
} = vi.hoisted(() => ({
  buildEditorActionPrompt: vi.fn(() => 'scoped group prompt'),
  sendKnowledgeAgentMessage: vi.fn(),
  synthesizeKnowledgeSearchAnswer: vi.fn(),
}));

vi.mock('./knowledgeAgentChatService', () => ({
  buildEditorActionPrompt,
  sendKnowledgeAgentMessage,
  synthesizeKnowledgeSearchAnswer,
}));

import { AIService } from './ai';
import { resolveKnowledgebaseWorkspaceAiScope } from '../workspaceMode';

afterEach(() => {
  setKnowledgebaseApiEnabled(false);
  vi.unstubAllEnvs();
  vi.useRealTimers();
  vi.clearAllMocks();
});

describe('AI service fail-closed behavior', () => {
  it('does not synthesize media when demo mode is enabled', async () => {
    setKnowledgebaseApiEnabled(false);
    vi.stubEnv('VITE_SDKWORK_KNOWLEDGEBASE_ENABLE_DEMO_MODE', 'true');

    await expect(
      AIService.generateImage('prompt', '1:1', 'default'),
    ).rejects.toMatchObject({
      code: KnowledgebaseErrorCodes.API_UNAVAILABLE_IMAGE,
    });
    await expect(
      AIService.speechToText('https://media.example.invalid/audio.mp3'),
    ).rejects.toMatchObject({
      code: KnowledgebaseErrorCodes.API_UNAVAILABLE_SPEECH,
    });
  });

  it('fails closed for a fixed group space before any primary-space fallback', async () => {
    setKnowledgebaseApiEnabled(true);
    sendKnowledgeAgentMessage.mockResolvedValue('group response');

    const scope = resolveKnowledgebaseWorkspaceAiScope('ephemeral-fixed', 'group-space-42');
    await expect(
      AIService.generateChatResponse('question', 'group document', 'group reference', scope),
    ).rejects.toMatchObject({
      code: KnowledgebaseErrorCodes.API_UNAVAILABLE_CHAT,
    });

    expect(sendKnowledgeAgentMessage).not.toHaveBeenCalled();
  });

  it('rejects an ephemeral workspace without an authorized group space', () => {
    expect(() => resolveKnowledgebaseWorkspaceAiScope('ephemeral-fixed', undefined)).toThrow(
      'requires an active space',
    );
  });
});
