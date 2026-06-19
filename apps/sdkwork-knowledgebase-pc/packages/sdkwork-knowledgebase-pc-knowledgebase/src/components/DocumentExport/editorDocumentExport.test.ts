import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
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
    assert.equal(content?.sourceKind, 'markdown');
    assert.equal(content?.markdown, '# Hello');
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
    assert.equal(content?.sourceKind, 'richtext');
    assert.match(content?.html ?? '', /<strong>Hi<\/strong>/);
    assert.equal(content?.markdown, '**Hi**');
  });
});
