import type { IngestionJob } from '@sdkwork/knowledgebase-app-sdk';
import { getKnowledgebaseAppSdkClient } from 'sdkwork-knowledgebase-pc-core';

const INGEST_POLL_INTERVAL_MS = 250;
const INGEST_POLL_TIMEOUT_MS = 30_000;

export async function waitForIngestJob(jobId: number): Promise<IngestionJob> {
  const client = getKnowledgebaseAppSdkClient().client;
  const deadline = Date.now() + INGEST_POLL_TIMEOUT_MS;
  let job = await client.knowledge.ingests.retrieve(jobId);

  while (job.state === 'queued' || job.state === 'running') {
    if (Date.now() >= deadline) {
      throw new Error(`Ingest job ${jobId} did not complete within ${INGEST_POLL_TIMEOUT_MS / 1000}s.`);
    }
    await new Promise((resolve) => setTimeout(resolve, INGEST_POLL_INTERVAL_MS));
    job = await client.knowledge.ingests.retrieve(jobId);
  }

  if (job.state === 'failed') {
    throw new Error(job.errorMessage ?? `Ingest job ${jobId} failed.`);
  }
  if (job.state === 'cancelled') {
    throw new Error(`Ingest job ${jobId} was cancelled.`);
  }

  return job;
}

export async function resolveIngestedDocument(
  spaceId: number,
  title: string,
): Promise<{ id: number; title: string }> {
  const client = getKnowledgebaseAppSdkClient().client;
  const documents = await client.knowledge.documents.list();
  const normalizedTitle = title.trim();

  const matches = documents.items.filter(
    (document) => document.spaceId === spaceId && document.title === normalizedTitle,
  );
  if (matches.length > 0) {
    const latest = matches.reduce((left, right) => (right.id > left.id ? right : left));
    return { id: latest.id, title: latest.title };
  }

  const browser = await client.knowledge.spaces.browser.list(spaceId, {
    view: 'files',
    pageSize: 100,
  });
  const node = browser.items.find(
    (entry) => entry.name === normalizedTitle && entry.documentId,
  );
  if (node?.documentId) {
    return { id: node.documentId, title: node.name };
  }

  throw new Error(`Could not resolve ingested document "${normalizedTitle}" in space ${spaceId}.`);
}
