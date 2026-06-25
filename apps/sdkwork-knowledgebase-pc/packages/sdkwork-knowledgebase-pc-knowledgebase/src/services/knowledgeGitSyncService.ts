import {
  KnowledgebaseErrorCodes,
  parseKnowledgeSpaceId,
  requireKnowledgebaseAppSdkHttpClient,
  requireNonEmptyString,
} from 'sdkwork-knowledgebase-pc-core';

export interface GitSyncProgress {
  phase: 'resolve' | 'push' | 'done';
  message: string;
  syncedCount?: number;
}

export interface GitSyncResult {
  success: boolean;
  hash: string;
  syncedCount: number;
}

function requireSdkClient() {
  return requireKnowledgebaseAppSdkHttpClient();
}

function spaceIdFromKbId(kbId: string): number {
  return parseKnowledgeSpaceId(kbId);
}

function normalizeBranch(branch: string): string {
  const trimmed = branch.trim();
  return trimmed || 'main';
}

function buildSyncIdempotencyKey(
  spaceId: number,
  repoUrl: string,
  branch: string,
  commitMessage: string,
): string {
  const raw = `git-sync-${spaceId}-${repoUrl.trim()}-${branch}-${commitMessage.trim()}`;
  return raw.replace(/[^a-zA-Z0-9._-]/g, '-').slice(0, 128);
}

export async function syncGitRepository(
  kbId: string,
  repoUrl: string,
  branch: string,
  commitMessage: string,
  accessToken?: string,
  onProgress?: (progress: GitSyncProgress) => void,
): Promise<GitSyncResult> {
  const trimmedRepoUrl = requireNonEmptyString(repoUrl, KnowledgebaseErrorCodes.REPO_URL_REQUIRED);
  const trimmedCommitMessage = requireNonEmptyString(
    commitMessage,
    KnowledgebaseErrorCodes.COMMIT_MESSAGE_REQUIRED,
  );

  const normalizedBranch = normalizeBranch(branch);
  const spaceId = spaceIdFromKbId(kbId);
  const client = requireSdkClient();
  const idempotencyKey = buildSyncIdempotencyKey(
    spaceId,
    trimmedRepoUrl,
    normalizedBranch,
    trimmedCommitMessage,
  );
  const trimmedAccessToken = accessToken?.trim();

  onProgress?.({
    phase: 'resolve',
    message: `Resolving repository on branch "${normalizedBranch}"…`,
  });

  onProgress?.({
    phase: 'push',
    message: 'Pushing knowledge base documents to the remote repository…',
  });

  const result = await client.knowledge.gitSyncs.create({
    spaceId,
    repoUrl: trimmedRepoUrl,
    branch: normalizedBranch,
    commitMessage: trimmedCommitMessage,
    idempotencyKey,
    gitAccessToken: trimmedAccessToken || undefined,
  });

  onProgress?.({
    phase: 'done',
    message: `Synced ${result.syncedCount} file(s) to Git.`,
    syncedCount: result.syncedCount,
  });

  return {
    success: result.success,
    hash: result.hash,
    syncedCount: result.syncedCount,
  };
}
