export interface OkfConceptUpsertRequest {
  spaceId: number;
  conceptId: string;
  markdown: string;
  actor: string;
  publish: boolean;
}
