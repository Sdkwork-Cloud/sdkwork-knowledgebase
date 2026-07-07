import type { IngestionJob } from 'sdkwork-knowledgebase-pc-core';
import {
  KnowledgebaseErrorCodes,
  requireKnowledgebaseAppSdkHttpClient,
  throwKnowledgebaseError,
} from 'sdkwork-knowledgebase-pc-core';
import { listKnowledgeBrowserNodesPage } from './knowledgeBrowserListService';

const INGEST_POLL_INTERVAL_MS = 250;
const INGEST_POLL_TIMEOUT_MS = 30_000;
const INGEST_RESOLVE_ATTEMPTS = 6;
const INGEST_RESOLVE_RETRY_MS = 300;

export async function waitForIngestJob(jobId: string | number): Promise<IngestionJob> {
  const client = requireKnowledgebaseAppSdkHttpClient();
  const deadline = Date.now() + INGEST_POLL_TIMEOUT_MS;
  let job = await client.knowledge.ingests.retrieve(String(jobId));

  while (job.state === 'queued' || job.state === 'running') {
    if (Date.now() >= deadline) {
      throwKnowledgebaseError(KnowledgebaseErrorCodes.INGEST_FAILED);
    }
    await new Promise((resolve) => setTimeout(resolve, INGEST_POLL_INTERVAL_MS));
    job = await client.knowledge.ingests.retrieve(String(jobId));
  }

  if (job.state === 'failed') {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.INGEST_FAILED, {
      cause: job.errorMessage ?? undefined,
    });
  }
  if (job.state === 'cancelled') {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.INGEST_FAILED);
  }

  return job;
}

async function findDocumentByTitleInBrowser(
  spaceId: string,
  normalizedTitle: string,
  fresh: boolean,
): Promise<{ id: string; title: string } | null> {
  let cursor: string | null = null;
  do {
    const page = await listKnowledgeBrowserNodesPage(spaceId, null, {
      cursor,
      fresh,
      pageSize: 100,
    });
    const node = page.items.find(
      (entry) => entry.name === normalizedTitle && entry.documentId,
    );
    if (node?.documentId) {
      return { id: String(node.documentId), title: node.name };
    }
    cursor = page.hasMore ? page.nextCursor : null;
  } while (cursor);
  return null;
}

export async function resolveIngestedDocument(
  spaceId: string,
  title: string,
): Promise<{ id: string; title: string }> {
  const normalizedTitle = title.trim();

  for (let attempt = 0; attempt < INGEST_RESOLVE_ATTEMPTS; attempt += 1) {
    if (attempt > 0) {
      await new Promise((resolve) => setTimeout(resolve, INGEST_RESOLVE_RETRY_MS));
    }

    const fromBrowser = await findDocumentByTitleInBrowser(
      spaceId,
      normalizedTitle,
      attempt > 0,
    );
    if (fromBrowser) {
      return fromBrowser;
    }
  }

  throwKnowledgebaseError(KnowledgebaseErrorCodes.INGEST_FAILED);
}
