export interface WikiPageSummary {
  title: string;
  slug: string;
  pageType: string;
  logicalPath: string;
  summary: string;
  sourceCount: number;
  updatedAt: string;
  tags: string[];
}
