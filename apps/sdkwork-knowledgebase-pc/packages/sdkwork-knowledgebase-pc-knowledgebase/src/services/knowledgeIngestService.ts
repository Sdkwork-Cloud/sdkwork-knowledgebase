import type { IngestionJob } from 'sdkwork-knowledgebase-pc-core';
import {
  KnowledgebaseErrorCodes,
  requireKnowledgebaseAppSdkHttpClient,
  throwKnowledgebaseError,
} from 'sdkwork-knowledgebase-pc-core';
import { normalizeSdkWorkListPage } from './sdkWorkListPage';

const INGEST_POLL_INTERVAL_MS = 250;
const INGEST_POLL_TIMEOUT_MS = 30_000;

export async function waitForIngestJob(jobId: number): Promise<IngestionJob> {
  const client = requireKnowledgebaseAppSdkHttpClient();
  const deadline = Date.now() + INGEST_POLL_TIMEOUT_MS;
  let job = await client.knowledge.ingests.retrieve(jobId);

  while (job.state === 'queued' || job.state === 'running') {
    if (Date.now() >= deadline) {
      throwKnowledgebaseError(KnowledgebaseErrorCodes.INGEST_FAILED);
    }
    await new Promise((resolve) => setTimeout(resolve, INGEST_POLL_INTERVAL_MS));
    job = await client.knowledge.ingests.retrieve(jobId);
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

export async function resolveIngestedDocument(
  spaceId: number,
  title: string,
): Promise<{ id: number; title: string }> {
  const client = requireKnowledgebaseAppSdkHttpClient();
  const documents = normalizeSdkWorkListPage(
    await client.knowledge.documents.list({ spaceId, pageSize: 200 }),
  );
  const normalizedTitle = title.trim();

  const matches = documents.items.filter(
    (document) => document.spaceId === spaceId && document.title === normalizedTitle,
  );
  if (matches.length > 0) {
    const latest = matches.reduce((left, right) => (right.id > left.id ? right : left));
    return { id: latest.id, title: latest.title };
  }

  const browser = normalizeSdkWorkListPage(
    await client.knowledge.spaces.browser.list(spaceId, {
      view: 'files',
      pageSize: 100,
    }),
  );
  const node = browser.items.find(
    (entry) => entry.name === normalizedTitle && entry.documentId,
  );
  if (node?.documentId) {
    return { id: node.documentId, title: node.name };
  }

  throwKnowledgebaseError(KnowledgebaseErrorCodes.INGEST_FAILED);
}
