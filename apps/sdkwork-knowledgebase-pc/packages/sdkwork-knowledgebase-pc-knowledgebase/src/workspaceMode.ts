export type KnowledgebaseWorkspaceMode = 'standard' | 'ephemeral-fixed';

export interface KnowledgebaseWorkspaceAiScope {
  persistSpaceProfileCache?: boolean;
  spaceId?: string;
}

interface EphemeralFixedWorkspaceLease {
  spaceId: string;
  token: symbol;
}

let activeEphemeralFixedWorkspace: EphemeralFixedWorkspaceLease | null = null;

export function shouldPersistKnowledgebaseWorkspaceState(
  workspaceMode: KnowledgebaseWorkspaceMode,
): boolean {
  return workspaceMode === 'standard';
}

/**
 * The standalone group window carries a server-authorized space only in
 * process memory. Services use this lease to avoid the personal-space
 * registry and all browser storage while the group workspace is mounted.
 */
export function activateEphemeralFixedKnowledgebaseWorkspace(spaceId: string): () => void {
  const normalizedSpaceId = spaceId.trim();
  if (!normalizedSpaceId) {
    throw new Error('An ephemeral fixed knowledgebase workspace requires an active space.');
  }

  const lease: EphemeralFixedWorkspaceLease = {
    spaceId: normalizedSpaceId,
    token: Symbol('ephemeral-fixed-knowledgebase-workspace'),
  };
  activeEphemeralFixedWorkspace = lease;

  return () => {
    if (activeEphemeralFixedWorkspace?.token === lease.token) {
      activeEphemeralFixedWorkspace = null;
    }
  };
}

export function getActiveEphemeralFixedKnowledgebaseWorkspaceSpaceId(): string | null {
  return activeEphemeralFixedWorkspace?.spaceId ?? null;
}

/** Group-scoped AI is disabled until its authorization path is complete. */
export function isKnowledgebaseWorkspaceAiEnabled(
  workspaceMode: KnowledgebaseWorkspaceMode,
): boolean {
  return workspaceMode === 'standard';
}

/**
 * A fixed group workspace must always carry its server-authorized space id.
 * Returning an explicit scope prevents UI actions from falling back to the
 * user's primary/personal knowledge space.
 */
export function resolveKnowledgebaseWorkspaceAiScope(
  workspaceMode: KnowledgebaseWorkspaceMode,
  spaceId: string | null | undefined,
): KnowledgebaseWorkspaceAiScope | undefined {
  if (workspaceMode === 'standard') {
    return undefined;
  }

  const normalizedSpaceId = spaceId?.trim();
  if (!normalizedSpaceId) {
    throw new Error('An ephemeral fixed knowledgebase workspace requires an active space.');
  }

  return {
    persistSpaceProfileCache: false,
    spaceId: normalizedSpaceId,
  };
}
