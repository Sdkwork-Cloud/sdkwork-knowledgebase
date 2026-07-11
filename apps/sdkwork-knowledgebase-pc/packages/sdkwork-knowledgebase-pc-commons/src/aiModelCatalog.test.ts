import { describe, expect, it } from 'vitest';

import {
  AI_DEFAULT_MODEL,
  AI_DEFAULT_VENDOR,
  AI_MODELS,
  AI_VENDORS,
  normalizeAiModelSelection,
} from './aiModelCatalog';

describe('AI model catalog', () => {
  it('advertises only the registered SDKWork Rig default without provider discovery', () => {
    expect(AI_DEFAULT_VENDOR).toBe('sdkwork-ai');
    expect(AI_DEFAULT_MODEL).toEqual({
      id: 'rig.default-chat',
      name: 'SDKWork AI',
    });
    expect(AI_VENDORS).toEqual([{ id: 'sdkwork-ai', name: 'SDKWork AI' }]);
    expect(AI_MODELS).toEqual({
      'sdkwork-ai': [{ id: 'rig.default-chat', name: 'SDKWork AI' }],
    });
    expect(normalizeAiModelSelection('google', {
      id: 'gemini-1.5-pro',
      name: 'Gemini 1.5 Pro',
    })).toEqual({
      vendorId: 'sdkwork-ai',
      model: { id: 'rig.default-chat', name: 'SDKWork AI' },
    });
  });
});
