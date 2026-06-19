import { useEffect } from 'react';

import type { KnowledgebasePcRuntime } from 'sdkwork-knowledgebase-pc-core';

import type { KnowledgebaseIamRuntime } from './knowledgebaseIamRuntime';
import { signOutKnowledgebaseAccount } from './knowledgebaseAccountViewModel';

export function useHydrateKnowledgebaseAccount(runtime: KnowledgebasePcRuntime): void {
  useEffect(() => {
    const snapshot = runtime.session.getSnapshot();
    if (!snapshot.accessToken && !snapshot.authToken) {
      return;
    }

    const iamRuntime = runtime.auth?.iamRuntime as KnowledgebaseIamRuntime | undefined;
    if (!iamRuntime) {
      return;
    }

    let cancelled = false;

    void (async () => {
      try {
        if (!runtime.session.getSnapshot().user?.displayName) {
          await iamRuntime.service.iam.users.current.retrieve();
        }
      } catch {
        // Best-effort profile hydration; auth gate handles incomplete sessions.
      }

      if (cancelled || runtime.session.getSnapshot().context?.tenantId) {
        return;
      }

      try {
        await iamRuntime.service.auth.sessions.current.retrieve();
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
  const iamRuntime = runtime.auth?.iamRuntime as KnowledgebaseIamRuntime | undefined;

  try {
    await iamRuntime?.contextStore?.clear?.();
  } catch {
    // Continue with local session cleanup.
  }

  try {
    const deleteCurrentSession = iamRuntime?.service?.auth?.sessions?.current?.delete;
    if (typeof deleteCurrentSession === 'function') {
      await deleteCurrentSession.call(iamRuntime.service.auth.sessions.current);
      return;
    }
  } catch {
    // Continue with local session cleanup.
  }

  signOutKnowledgebaseAccount(runtime.session);
}
