import { isBlank } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';
import { getKnowledgebaseAppSdkClient } from 'sdkwork-knowledgebase-pc-core';

import { waitForIngestJob } from './knowledgeIngestService';

const MAX_IMPORT_FILES = 64;
const MAX_FILE_BYTES = 512 * 1024;
const IMPORTABLE_EXTENSIONS = new Set([
  '.md',
  '.markdown',
  '.txt',
  '.json',
  '.yaml',
  '.yml',
  '.toml',
  '.csv',
  '.ts',
  '.tsx',
  '.js',
  '.jsx',
  '.py',
  '.java',
  '.go',
  '.rs',
  '.html',
  '.htm',
  '.css',
  '.xml',
]);

export interface GitImportProgress {
  phase: 'resolve' | 'list' | 'fetch' | 'ingest' | 'done';
  message: string;
  importedCount?: number;
  totalCount?: number;
}

export interface GitImportResult {
  importedCount: number;
  skippedCount: number;
}

interface ParsedGitHubRepo {
  owner: string;
  repo: string;
}

function requireSdkClient() {
  const sdk = getKnowledgebaseAppSdkClient();
  if (!sdk) {
    throw new Error('Knowledgebase app SDK is not configured.');
  }
  return sdk.client;
}

function spaceIdFromKbId(kbId: string): number {
  const spaceId = Number(kbId);
  if (!Number.isFinite(spaceId) || spaceId <= 0) {
    throw new Error(`Invalid knowledge space id: ${kbId}`);
  }
  return spaceId;
}

function normalizeBranch(branch: string): string {
  const trimmed = branch.trim();
  return trimmed || 'main';
}

export function parseGitHubRepoUrl(repoUrl: string): ParsedGitHubRepo {
  const trimmed = repoUrl.trim();
  const match = /^https?:\/\/github\.com\/([^/]+)\/([^/]+?)(?:\.git)?\/?$/i.exec(trimmed);
  if (!match) {
    throw new Error('Only public HTTPS GitHub repository URLs are supported for API import.');
  }
  return {
    owner: decodeURIComponent(match[1]),
    repo: decodeURIComponent(match[2]),
  };
}

function buildAuthHeaders(accessToken?: string): HeadersInit {
  const headers: Record<string, string> = {
    Accept: 'application/vnd.github+json',
    'X-GitHub-Api-Version': '2022-11-28',
  };
  const token = accessToken?.trim();
  if (token) {
    headers.Authorization = `Bearer ${token}`;
  }
  return headers;
}

function isImportablePath(path: string, size?: number | null): boolean {
  if (path.includes('..') || path.startsWith('/')) {
    return false;
  }
  if (typeof size === 'number' && size > MAX_FILE_BYTES) {
    return false;
  }
  const lower = path.toLowerCase();
  const dot = lower.lastIndexOf('.');
  if (dot < 0) {
    return false;
  }
  return IMPORTABLE_EXTENSIONS.has(lower.slice(dot));
}

function buildIdempotencyKey(spaceId: number, repoKey: string, path: string): string {
  const raw = `git-import-${spaceId}-${repoKey}-${path}`;
  return raw.replace(/[^a-zA-Z0-9._-]/g, '-').slice(0, 128);
}

async function resolveGitHubBranchSha(
  owner: string,
  repo: string,
  branch: string,
  headers: HeadersInit,
): Promise<string> {
  const response = await fetch(
    `https://api.github.com/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/branches/${encodeURIComponent(branch)}`,
    { headers },
  );
  if (!response.ok) {
    const detail = response.status === 404
      ? `branch "${branch}" was not found`
      : `GitHub API returned HTTP ${response.status}`;
    throw new Error(`Failed to resolve Git branch: ${detail}`);
  }
  const payload = await response.json() as { commit?: { sha?: string } };
  const sha = payload.commit?.sha;
  if (!sha) {
    throw new Error('Failed to resolve Git branch commit SHA.');
  }
  return sha;
}

async function listGitHubImportPaths(
  owner: string,
  repo: string,
  branchSha: string,
  headers: HeadersInit,
): Promise<string[]> {
  const response = await fetch(
    `https://api.github.com/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/git/trees/${branchSha}?recursive=1`,
    { headers },
  );
  if (!response.ok) {
    throw new Error(`Failed to list repository tree: HTTP ${response.status}`);
  }
  const payload = await response.json() as {
    tree?: Array<{ path?: string; type?: string; size?: number }>;
  };
  const paths = (payload.tree ?? [])
    .filter((entry) => entry.type === 'blob' && entry.path && isImportablePath(entry.path, entry.size))
    .map((entry) => entry.path as string)
    .sort((left, right) => left.localeCompare(right));

  if (paths.length === 0) {
    throw new Error('No importable text or markdown files were found in the repository.');
  }
  return paths.slice(0, MAX_IMPORT_FILES);
}

async function fetchGitHubFileContent(
  owner: string,
  repo: string,
  branch: string,
  path: string,
  headers: HeadersInit,
): Promise<string> {
  const response = await fetch(
    `https://api.github.com/repos/${encodeURIComponent(owner)}/${encodeURIComponent(repo)}/contents/${path.split('/').map(encodeURIComponent).join('/')}?ref=${encodeURIComponent(branch)}`,
    {
      headers: {
        ...Object.fromEntries(new Headers(headers).entries()),
        Accept: 'application/vnd.github.raw',
      },
    },
  );
  if (!response.ok) {
    throw new Error(`Failed to fetch "${path}": HTTP ${response.status}`);
  }
  const text = await response.text();
  if (text.length > MAX_FILE_BYTES) {
    throw new Error(`File "${path}" exceeds the ${MAX_FILE_BYTES} byte import limit.`);
  }
  if (isBlank(text)) {
    throw new Error(`File "${path}" is empty.`);
  }
  return text;
}

function titleFromPath(path: string): string {
  const segments = path.split('/');
  return segments[segments.length - 1] || path;
}

export async function importGitRepository(
  kbId: string,
  repoUrl: string,
  branch: string = 'main',
  accessToken?: string,
  onProgress?: (progress: GitImportProgress) => void,
): Promise<GitImportResult> {
  const parsed = parseGitHubRepoUrl(repoUrl);
  const normalizedBranch = normalizeBranch(branch);
  const spaceId = spaceIdFromKbId(kbId);
  const client = requireSdkClient();
  const headers = buildAuthHeaders(accessToken);
  const repoKey = `${parsed.owner}/${parsed.repo}@${normalizedBranch}`;

  onProgress?.({
    phase: 'resolve',
    message: `Resolving branch "${normalizedBranch}"…`,
  });
  const branchSha = await resolveGitHubBranchSha(parsed.owner, parsed.repo, normalizedBranch, headers);

  onProgress?.({
    phase: 'list',
    message: 'Listing importable repository files…',
  });
  const paths = await listGitHubImportPaths(parsed.owner, parsed.repo, branchSha, headers);

  let importedCount = 0;
  let skippedCount = 0;

  for (let index = 0; index < paths.length; index += 1) {
    const path = paths[index];
    onProgress?.({
      phase: 'fetch',
      message: `Fetching ${path}…`,
      importedCount,
      totalCount: paths.length,
    });

    try {
      const content = await fetchGitHubFileContent(
        parsed.owner,
        parsed.repo,
        normalizedBranch,
        path,
        headers,
      );

      onProgress?.({
        phase: 'ingest',
        message: `Ingesting ${path}…`,
        importedCount,
        totalCount: paths.length,
      });

      await client.knowledge.ingests.create({
        spaceId,
        title: titleFromPath(path),
        payloadMarkdown: content,
        idempotencyKey: buildIdempotencyKey(spaceId, repoKey, path),
      }).then(async (job) => {
        if (job.state !== 'succeeded') {
          await waitForIngestJob(job.id);
        }
      });
      importedCount += 1;
    } catch (error) {
      console.warn(`[KnowledgeGitImportService] skipped "${path}"`, error);
      skippedCount += 1;
    }
  }

  if (importedCount === 0) {
    throw new Error('Git import did not ingest any files. Check repository access and file types.');
  }

  onProgress?.({
    phase: 'done',
    message: `Imported ${importedCount} file(s).`,
    importedCount,
    totalCount: paths.length,
  });

  return { importedCount, skippedCount };
}
