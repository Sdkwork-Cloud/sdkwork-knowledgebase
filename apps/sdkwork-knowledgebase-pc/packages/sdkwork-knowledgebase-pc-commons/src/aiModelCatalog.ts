export interface AiVendor {
  id: string;
  name: string;
}

export interface AiModelInfo {
  id: string;
  name: string;
}

export const AI_VENDORS: AiVendor[] = [
  { id: 'google', name: 'Google' },
  { id: 'openai', name: 'OpenAI' },
  { id: 'anthropic', name: 'Anthropic' },
  { id: 'deepseek', name: 'DeepSeek' }
];

export const AI_MODELS: Record<string, AiModelInfo[]> = {
  google: [
    { id: 'gemini-1.5-pro', name: 'Gemini 1.5 Pro' },
    { id: 'gemini-1.5-flash', name: 'Gemini 1.5 Flash' }
  ],
  openai: [
    { id: 'gpt-4o', name: 'GPT-4o' },
    { id: 'gpt-4o-mini', name: 'GPT-4o Mini' },
    { id: 'o1-mini', name: 'o1 Mini' },
    { id: 'o3-mini', name: 'o3 Mini' }
  ],
  anthropic: [
    { id: 'claude-3-5-sonnet', name: 'Claude 3.5 Sonnet' },
    { id: 'claude-3-opus', name: 'Claude 3 Opus' }
  ],
  deepseek: [
    { id: 'deepseek-chat', name: 'DeepSeek V3' },
    { id: 'deepseek-reasoner', name: 'DeepSeek R1' }
  ]
};

export const AI_DEFAULT_VENDOR = 'google';
export const AI_DEFAULT_MODEL = AI_MODELS.google[0];

export const AI_ACTIVE_VENDOR_KEY = 'ai-active-vendor';
export const AI_ACTIVE_MODEL_KEY = 'ai-active-model';
