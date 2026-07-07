export interface OkfConceptUpsertRequest {
  spaceId: string;
  conceptId: string;
  markdown: string;
  actor: string;
  publish: boolean;
}
