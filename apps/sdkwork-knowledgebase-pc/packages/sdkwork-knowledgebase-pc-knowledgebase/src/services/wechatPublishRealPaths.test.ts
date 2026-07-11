import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

function readSource(relativePath: string): string {
  return readFileSync(new URL(relativePath, import.meta.url), 'utf8');
}

describe('WeChat publish real integration paths', () => {
  it('uses real media surfaces without fixed demo assets or timer-driven content', () => {
    const source = readSource('../WechatPublishPage.tsx');

    expect(source).toContain('<WechatAiImageModal');
    expect(source).toContain("setAssetLibraryTab('image')");
    expect(source).not.toMatch(
      /images\.unsplash\.com|sample-videos\.com|soundhelix\.com|styleArtMap|pickedImage/,
    );
    expect(source).not.toMatch(
      /continuationChunks|contentParagraphs|demo UX delay|new Promise\s*\([^)]*setTimeout/,
    );
    expect(source).not.toContain('shouldUseKnowledgebaseDemoFallback');
  });

  it('does not retain fake draft success or local simulated streaming handlers', () => {
    const source = readSource('../WechatPublishPage.tsx');

    expect(source).not.toMatch(
      /if\s*\(!isKnowledgebaseApiAvailable\(\)\)\s*{[\s\S]{0,500}toast\.success/,
    );
    expect(source).not.toMatch(
      /handleStreamingContinue|handleCreateNewArticleAndWrite|wechatStreamContinueSuccess|wechatStreamDraftSuccess/,
    );
  });
});
