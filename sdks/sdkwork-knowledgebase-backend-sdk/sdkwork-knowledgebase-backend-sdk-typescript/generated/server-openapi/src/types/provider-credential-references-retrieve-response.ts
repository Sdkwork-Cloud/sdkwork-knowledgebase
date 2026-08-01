import type { KnowledgeEngineProviderCredentialReference } from './knowledge-engine-provider-credential-reference';

export interface ProviderCredentialReferencesRetrieveResponse {
  code: 0;
  data: unknown & { item: KnowledgeEngineProviderCredentialReference; };
  /** Server-owned request correlation id. */
  traceId: string;
}
