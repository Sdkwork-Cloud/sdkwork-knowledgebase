import { useLocalStorage } from './hooks';
import {
  AI_ACTIVE_MODEL_KEY,
  AI_ACTIVE_VENDOR_KEY,
  AI_DEFAULT_MODEL,
  AI_DEFAULT_VENDOR,
  AI_MODELS,
  normalizeAiModelSelection,
  type AiModelInfo
} from './aiModelCatalog';

export function useAiModelSelection() {
  const [activeVendor, setActiveVendor] = useLocalStorage(AI_ACTIVE_VENDOR_KEY, AI_DEFAULT_VENDOR);
  const [activeModel, setActiveModel] = useLocalStorage<AiModelInfo>(
    AI_ACTIVE_MODEL_KEY,
    AI_DEFAULT_MODEL
  );
  const selection = normalizeAiModelSelection(activeVendor, activeModel);

  return {
    activeVendor: selection.vendorId,
    setActiveVendor,
    activeModel: selection.model,
    setActiveModel,
    modelsForVendor: AI_MODELS[selection.vendorId] ?? AI_MODELS[AI_DEFAULT_VENDOR]
  };
}
