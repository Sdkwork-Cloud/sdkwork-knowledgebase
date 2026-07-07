export interface KnowledgeGitImportRequest {
  spaceId: string;
  repoUrl: string;
  branch?: string | null;
  idempotencyKey: string;
  gitAccessToken?: string | null;
}
