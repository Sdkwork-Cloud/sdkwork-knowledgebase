import { useEffect } from 'react';

import type { KnowledgebasePcRuntime } from '../runtime/KnowledgebaseRuntimeProvider.js';
import { DEFAULT_SESSION_STORAGE_KEY } from './sessionStore.js';
import type { SessionStore } from './sessionStore.js';

type KnowledgebaseIamRuntimeLike = {
  service?: {
    iam?: {
      users?: {
        current?: {
          retrieve?: () => Promise<unknown>;
        };
      };
    };
    auth?: {
      sessions?: {
        current?: {
          retrieve?: () => Promise<unknown>;
          delete?: () => Promise<unknown>;
        };
      };
    };
  };
  contextStore?: {
    clear?: () => Promise<void> | void;
  };
};

export function signOutKnowledgebaseAccount(session: SessionStore): void {
  session.clearSession();
  clearPersistedKnowledgebaseRuntimeState();
}

function clearPersistedKnowledgebaseRuntimeState(): void {
  if (typeof window === 'undefined') {
    return;
  }

  const keys = [
    DEFAULT_SESSION_STORAGE_KEY,
    'app-active-tab',
    'app-search-view-active',
  ];
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

export function useHydrateKnowledgebaseAccount(runtime: KnowledgebasePcRuntime): void {
  useEffect(() => {
    const snapshot = runtime.session.getSnapshot();
    if (!snapshot.accessToken && !snapshot.authToken) {
      return;
    }

    const iamRuntime = runtime.auth?.iamRuntime as KnowledgebaseIamRuntimeLike | undefined;
    if (!iamRuntime) {
      return;
    }

    let cancelled = false;

    void (async () => {
      try {
        if (!runtime.session.getSnapshot().user?.displayName) {
          await iamRuntime.service?.iam?.users?.current?.retrieve?.();
        }
      } catch {
        // Best-effort profile hydration; auth gate handles incomplete sessions.
      }

      if (cancelled || runtime.session.getSnapshot().context?.tenantId) {
        return;
      }

      try {
        await iamRuntime.service?.auth?.sessions?.current?.retrieve?.();
      } catch {
        // Best-effort tenant hydration after login.
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [runtime]);
}

export async function signOutKnowledgebaseSession(
  runtime: KnowledgebasePcRuntime,
): Promise<void> {
  const iamRuntime = runtime.auth?.iamRuntime as KnowledgebaseIamRuntimeLike | undefined;

  try {
    await iamRuntime?.contextStore?.clear?.();
  } catch {
    // Continue with local session cleanup.
  }

  try {
    const deleteCurrentSession = iamRuntime?.service?.auth?.sessions?.current?.delete;
    if (typeof deleteCurrentSession === 'function') {
      await deleteCurrentSession.call(iamRuntime.service?.auth?.sessions?.current);
      return;
    }
  } catch {
    // Continue with local session cleanup.
  }

  signOutKnowledgebaseAccount(runtime.session);
}
