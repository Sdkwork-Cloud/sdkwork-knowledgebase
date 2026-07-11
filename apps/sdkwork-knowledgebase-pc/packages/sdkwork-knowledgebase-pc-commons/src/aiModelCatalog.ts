export interface AiVendor {
  id: string;
  name: string;
}

export interface AiModelInfo {
  id: string;
  name: string;
}

export const AI_VENDORS: AiVendor[] = [
  { id: 'sdkwork-ai', name: 'SDKWork AI' }
];

export const AI_MODELS: Record<string, AiModelInfo[]> = {
  'sdkwork-ai': [
    { id: 'rig.default-chat', name: 'SDKWork AI' }
  ]
};

export const AI_DEFAULT_VENDOR = 'sdkwork-ai';
export const AI_DEFAULT_MODEL = AI_MODELS[AI_DEFAULT_VENDOR][0];

export function normalizeAiModelSelection(
  vendorId: string,
  model: AiModelInfo,
): { vendorId: string; model: AiModelInfo } {
  const models = AI_MODELS[vendorId];
  const selectedModel = models?.find((candidate) => candidate.id === model.id);
  if (!selectedModel) {
    return { vendorId: AI_DEFAULT_VENDOR, model: AI_DEFAULT_MODEL };
  }
  return { vendorId, model: selectedModel };
}

export const AI_ACTIVE_VENDOR_KEY = 'ai-active-vendor';
export const AI_ACTIVE_MODEL_KEY = 'ai-active-model';
