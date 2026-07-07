import i18n from './i18n';
import { normalizeAppLanguage } from './runtime/locale';
import { getKnowledgebasePcSdkPorts } from './sdkPorts';

export function syncKnowledgebaseHostLanguage(): void {
  try {
    const ports = getKnowledgebasePcSdkPorts();
    const hostLanguage = ports.resolveHostLanguage?.();
    if (!hostLanguage) {
      return;
    }

    const nextLanguage = normalizeAppLanguage(hostLanguage);
    if (i18n.language !== nextLanguage) {
      void i18n.changeLanguage(nextLanguage);
    }
  } catch {
    // Host SDK ports may not be configured during standalone bootstrap.
  }
}

export function subscribeKnowledgebaseHostLanguage(): (() => void) | undefined {
  try {
    const ports = getKnowledgebasePcSdkPorts();
    const subscribe = ports.subscribeHostLanguage;
    if (!subscribe) {
      return undefined;
    }

    return subscribe((language) => {
      const nextLanguage = normalizeAppLanguage(language);
      if (i18n.language !== nextLanguage) {
        void i18n.changeLanguage(nextLanguage);
      }
    });
  } catch {
    return undefined;
  }
}
