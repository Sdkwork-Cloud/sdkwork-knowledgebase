import {
  getKnowledgebaseAppSdkClient,
  isKnowledgebaseApiAvailable,
} from './knowledgebaseApiRegistry';
import type { KnowledgeDocumentContent } from '@sdkwork/knowledgebase-app-sdk';

export type KnowledgeDocumentContentPayload = KnowledgeDocumentContent;

export async function fetchKnowledgeDocumentContent(
  documentId: number,
): Promise<KnowledgeDocumentContentPayload | null> {
  if (!isKnowledgebaseApiAvailable()) {
    return null;
  }

  try {
    const sdk = getKnowledgebaseAppSdkClient();
    return await sdk.client.knowledge.documents.content.retrieve(documentId);
  } catch {
    return null;
  }
}
