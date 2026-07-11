import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  KnowledgebaseErrorCodes,
  setKnowledgebaseApiEnabled,
} from 'sdkwork-knowledgebase-pc-core';

import { AIService } from './ai';

afterEach(() => {
  setKnowledgebaseApiEnabled(false);
  vi.unstubAllEnvs();
  vi.useRealTimers();
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
});
