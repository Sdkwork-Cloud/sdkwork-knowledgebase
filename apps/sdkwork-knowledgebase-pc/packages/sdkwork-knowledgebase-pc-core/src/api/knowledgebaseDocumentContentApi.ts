import {
  getKnowledgebaseAppSdkClient,
  isKnowledgebaseApiAvailable,
} from './knowledgebaseApiRegistry';
import type { KnowledgeDocumentContent } from '../sdk/sdkContractTypes.js';

export type KnowledgeDocumentContentPayload = KnowledgeDocumentContent;

export async function fetchKnowledgeDocumentContent(
  documentId: string | number,
): Promise<KnowledgeDocumentContentPayload | null> {
  if (!isKnowledgebaseApiAvailable()) {
    return null;
  }

  try {
    const sdk = getKnowledgebaseAppSdkClient();
    return await sdk.client.knowledge.documents.content.list(String(documentId));
  } catch {
    return null;
  }
}
