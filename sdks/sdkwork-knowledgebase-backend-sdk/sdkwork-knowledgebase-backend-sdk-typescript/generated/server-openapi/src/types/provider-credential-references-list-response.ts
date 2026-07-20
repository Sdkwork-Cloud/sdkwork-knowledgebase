import type { KnowledgeEngineProviderCredentialReferencePage } from './knowledge-engine-provider-credential-reference-page';

export interface ProviderCredentialReferencesListResponse {
  code: 0;
  data: unknown & KnowledgeEngineProviderCredentialReferencePage;
  /** Server-owned request correlation id. */
  traceId: string;
}
