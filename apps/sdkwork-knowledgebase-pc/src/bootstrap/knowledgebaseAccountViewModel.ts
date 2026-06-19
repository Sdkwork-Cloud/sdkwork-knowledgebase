export {
  createKnowledgebaseAccountViewModel,
  type KnowledgebaseAccountViewModel,
} from 'sdkwork-knowledgebase-pc-core';

import { DEFAULT_SESSION_STORAGE_KEY } from 'sdkwork-knowledgebase-pc-core';
import type { SessionStore } from 'sdkwork-knowledgebase-pc-core';

export function signOutKnowledgebaseAccount(session: SessionStore): void {
  session.clearSession();
  clearPersistedKnowledgebaseRuntimeState();
}

function clearPersistedKnowledgebaseRuntimeState(): void {
  if (typeof window === 'undefined') {
    return;
  }

  const keys = [DEFAULT_SESSION_STORAGE_KEY];
  const storages: Array<Storage | undefined> = [window.localStorage, window.sessionStorage];
  for (const storage of storages) {
    if (!storage) {
      continue;
    }
    for (const key of keys) {
      try {
        storage.removeItem(key);
      } catch {
        // Ignore storage cleanup errors to keep logout resilient.
      }
    }
  }
}
