import { describe, expect, it } from 'vitest';
import { createTiptapExportContentProvider } from './editorDocumentExport';

describe('createTiptapExportContentProvider', () => {
  it('uses source markdown in source mode', () => {
    const getContent = createTiptapExportContentProvider({
      title: 'Note',
      mode: 'markdown',
      isSourceMode: true,
      isSplitMode: false,
      sourceCode: '# Hello',
      getHtml: () => '<p>ignored</p>',
    });

    const content = getContent();
    expect(content?.sourceKind).toBe('markdown');
    expect(content?.markdown).toBe('# Hello');
  });

  it('uses editor html in visual mode', () => {
    const getContent = createTiptapExportContentProvider({
      title: 'Note',
      mode: 'richtext',
      isSourceMode: false,
      isSplitMode: false,
      sourceCode: '',
      getHtml: () => '<p><strong>Hi</strong></p>',
      getMarkdown: () => '**Hi**',
    });

    const content = getContent();
    expect(content?.sourceKind).toBe('richtext');
    expect(content?.html ?? '').toMatch(/<strong>Hi<\/strong>/);
    expect(content?.markdown).toBe('**Hi**');
  });
});
