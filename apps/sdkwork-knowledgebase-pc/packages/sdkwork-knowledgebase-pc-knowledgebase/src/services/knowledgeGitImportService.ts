import {
  KnowledgebaseErrorCodes,
  requireKnowledgebaseAppSdkHttpClient,
  requireNonEmptyString,
  requireRegisteredSpaceId,
} from 'sdkwork-knowledgebase-pc-core';

import { invalidateKnowledgeBrowserNodeCacheForSpaceIds } from './knowledgeBrowserListService';

export interface GitImportProgress {
  phase: 'resolve' | 'ingest' | 'done';
  message: string;
  importedCount?: number;
  skippedCount?: number;
  totalCount?: number;
}

export interface GitImportResult {
  importedCount: number;
  skippedCount: number;
}

function requireSdkClient() {
  return requireKnowledgebaseAppSdkHttpClient();
}

function normalizeBranch(branch: string): string {
  const trimmed = branch.trim();
  return trimmed || 'main';
}

function buildImportIdempotencyKey(spaceId: string, repoUrl: string, branch: string): string {
  const raw = `git-import-${spaceId}-${repoUrl.trim()}-${branch}`;
  return raw.replace(/[^a-zA-Z0-9._-]/g, '-').slice(0, 128);
}

export async function importGitRepository(
  kbId: string,
  repoUrl: string,
  branch: string = 'main',
  accessToken?: string,
  onProgress?: (progress: GitImportProgress) => void,
): Promise<GitImportResult> {
  const trimmedRepoUrl = requireNonEmptyString(repoUrl, KnowledgebaseErrorCodes.REPO_URL_REQUIRED);

  const normalizedBranch = normalizeBranch(branch);
  const spaceId = requireRegisteredSpaceId(kbId);
  const client = requireSdkClient();
  const idempotencyKey = buildImportIdempotencyKey(spaceId, trimmedRepoUrl, normalizedBranch);
  const trimmedAccessToken = accessToken?.trim();

  onProgress?.({
    phase: 'resolve',
    message: `Resolving repository on branch "${normalizedBranch}"\u2026`,
  });

  onProgress?.({
    phase: 'ingest',
    message: 'Importing repository files on the server\u2026',
  });

  const result = await client.knowledge.gitImports.create({
    spaceId,
    repoUrl: trimmedRepoUrl,
    branch: normalizedBranch,
    idempotencyKey,
    gitAccessToken: trimmedAccessToken || undefined,
  });

  onProgress?.({
    phase: 'done',
    message: `Imported ${result.importedCount} file(s).`,
    importedCount: result.importedCount,
    skippedCount: result.skippedCount,
  });

  invalidateKnowledgeBrowserNodeCacheForSpaceIds(spaceId);
  return result;
}
