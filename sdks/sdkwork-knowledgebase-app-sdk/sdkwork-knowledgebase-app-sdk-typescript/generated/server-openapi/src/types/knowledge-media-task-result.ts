export interface KnowledgeMediaTaskResult {
  success: boolean;
  url?: string | null;
  resolution?: string | null;
  text?: string | null;
  suggestions: string[];
  similars: string[];
}
