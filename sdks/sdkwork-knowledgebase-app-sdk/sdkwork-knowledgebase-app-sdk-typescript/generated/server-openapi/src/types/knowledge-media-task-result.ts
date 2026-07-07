export interface KnowledgeMediaTaskResult {
  accepted: true;
  status: 'completed';
  url?: string | null;
  resolution?: string | null;
  text?: string | null;
  suggestions: string[];
  similars: string[];
}
