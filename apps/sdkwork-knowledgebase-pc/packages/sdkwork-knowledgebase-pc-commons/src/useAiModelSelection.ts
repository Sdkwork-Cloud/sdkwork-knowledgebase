import { useLocalStorage } from './hooks';
import {
  AI_ACTIVE_MODEL_KEY,
  AI_ACTIVE_VENDOR_KEY,
  AI_DEFAULT_MODEL,
  AI_DEFAULT_VENDOR,
  AI_MODELS,
  type AiModelInfo
} from './aiModelCatalog';

export function useAiModelSelection() {
  const [activeVendor, setActiveVendor] = useLocalStorage(AI_ACTIVE_VENDOR_KEY, AI_DEFAULT_VENDOR);
  const [activeModel, setActiveModel] = useLocalStorage<AiModelInfo>(
    AI_ACTIVE_MODEL_KEY,
    AI_DEFAULT_MODEL
  );

  return {
    activeVendor,
    setActiveVendor,
    activeModel,
    setActiveModel,
    modelsForVendor: AI_MODELS[activeVendor] ?? AI_MODELS[AI_DEFAULT_VENDOR]
  };
}
