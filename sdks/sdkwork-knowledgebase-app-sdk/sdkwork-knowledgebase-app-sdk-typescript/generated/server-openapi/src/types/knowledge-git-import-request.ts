export interface KnowledgeGitImportRequest {
  spaceId: number;
  repoUrl: string;
  branch?: string | null;
  idempotencyKey: string;
  gitAccessToken?: string | null;
}
