import type { KnowledgeEngineProviderCredentialReference } from './knowledge-engine-provider-credential-reference';

export interface ProviderCredentialReferencesCreateResponse201 {
  code: 0;
  data: unknown & Record<string, unknown>;
  /** Server-owned request correlation id. */
  traceId: string;
}
