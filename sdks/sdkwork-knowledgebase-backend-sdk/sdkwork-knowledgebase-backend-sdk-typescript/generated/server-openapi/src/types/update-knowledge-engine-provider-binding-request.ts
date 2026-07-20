export interface UpdateKnowledgeEngineProviderBindingRequest {
  remoteResourceType?: string | null;
  remoteResourceId?: string | null;
  credentialReferenceId?: string | null;
  clearCredentialReference: boolean;
  expectedVersion: string;
}
