import { DocumentService } from '@packages/sdkwork-knowledgebase-pc-knowledgebase/src/services/document';
import { synthesizeKnowledgeSearchAnswer } from '@packages/sdkwork-knowledgebase-pc-knowledgebase/src/services/knowledgeAgentChatService';
import { isBlank } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';
import { getKnowledgebaseAppSdkClient, getKnowledgebaseTenantId, isKnowledgebaseApiAvailable, readRegisteredSpaces } from 'sdkwork-knowledgebase-pc-core';
import { buildRelatedMedia } from './buildRelatedMedia';
import type { SearchSource } from '../types';

export interface SearchQueryOptions {
  webSearchEnabled: boolean;
}

const MAX_DOC_SOURCES = 5;
const MAX_KB_SOURCES = 2;
const MAX_OKF_SOURCES = 2;
const MAX_WEB_SOURCES = 5;
const INCLUDE_PUBLIC_WEB_METADATA_KEY = 'includePublicWeb';
const PUBLIC_WEB_TOP_K_METADATA_KEY = 'publicWebTopK';

async function buildWebSources(query: string): Promise<SearchSource[]> {
  const tenantId = getKnowledgebaseTenantId();
  const sdk = getKnowledgebaseAppSdkClient();
  if (!tenantId || !sdk || isBlank(query)) {
    return [];
  }

  const registry = readRegisteredSpaces(tenantId);
  if (registry.length === 0) {
    return [];
  }

  const bindings = registry.slice(0, 1).map((entry, index) => ({
    spaceId: String(entry.spaceId),
    priority: index,
    topK: 1,
  }));

  try {
    const result = await sdk.client.knowledge.retrievals.create({
      query: query.trim(),
      bindings,
      includeCitations: true,
      includeTrace: false,
      topK: 1,
      metadata: [
        { key: INCLUDE_PUBLIC_WEB_METADATA_KEY, value: 'true' },
        { key: PUBLIC_WEB_TOP_K_METADATA_KEY, value: String(MAX_WEB_SOURCES) },
      ],
    });

    const sources: SearchSource[] = [];
    for (const hit of result.hits) {
      const sourceUri = hit.citation?.sourceUri?.trim();
      if (hit.retrievalMethod !== 'external' || !sourceUri) {
        continue;
      }
      sources.push({
        id: `web-${sources.length}-${hit.documentId}`,
        title: hit.title,
        type: 'web',
        url: sourceUri,
        snippet: hit.content,
      });
      if (sources.length >= MAX_WEB_SOURCES) {
        break;
      }
    }
    return sources;
  } catch (error) {
    console.warn('[SearchQueryEngine] public web retrieval failed.', error);
    return [];
  }
}

async function buildOkfSources(query: string): Promise<SearchSource[]> {
  const tenantId = getKnowledgebaseTenantId();
  const sdk = getKnowledgebaseAppSdkClient();
  if (!tenantId || !sdk || isBlank(query)) {
    return [];
  }

  const registry = readRegisteredSpaces(tenantId);
  const sources: SearchSource[] = [];
  for (const entry of registry.slice(0, MAX_OKF_SOURCES)) {
    try {
      const result = await sdk.client.knowledge.okf.queries.create({
        spaceId: entry.spaceId,
        query,
      });
      const snippet = result.answerMarkdown
        .replace(/[#*_`>\[\]]/g, ' ')
        .replace(/\s+/g, ' ')
        .trim()
        .slice(0, 180);
      if (!snippet) {
        continue;
      }
      sources.push({
        id: `okf-${entry.spaceId}-${sources.length}`,
        title: `OKF · ${query}`,
        type: 'doc',
        kbId: String(entry.spaceId),
        docType: 'markdown',
        snippet: `${snippet}...`,
      });
    } catch {
      // Skip spaces without initialized OKF bundles.
    }
  }
  return sources;
}

function buildLocalSources(
  searchResults: Awaited<ReturnType<typeof DocumentService.searchAll>>
): SearchSource[] {
  const sources: SearchSource[] = [];

  for (const doc of searchResults.docs) {
    if (sources.filter((s) => s.type === 'doc').length >= MAX_DOC_SOURCES) break;
    sources.push({
      id: `doc-${doc.id}`,
      docId: doc.id,
      title: doc.title,
      type: 'doc',
      kbId: doc.kbId,
      docType: doc.type,
      parentId: doc.parentId ?? null,
      author: doc.author,
      updatedAt: doc.updatedAt,
      snippet: doc.content
        ? doc.content.replace(/<[^>]*>?/gm, '').substring(0, 160).trim() + '...'
        : `属于您知识库中的一个【${doc.type}】类型文件，作者是 "${doc.author || '未知 author'}"，更新于 ${new Date(doc.updatedAt || '').toLocaleDateString()}。`
    });
  }

  for (const kb of searchResults.kbs) {
    if (sources.filter((s) => s.type === 'kb').length >= MAX_KB_SOURCES) break;
    sources.push({
      id: `kb-${kb.id}`,
      title: kb.title,
      type: 'kb',
      kbId: kb.id,
      kbTitle: kb.title,
      snippet: `这是一个知识库分类目录（分类：${kb.type === 'personal' ? '个人' : '团队'}）。点击可以快速跳转。`
    });
  }

  return sources;
}

async function enrichKbTitles(sources: SearchSource[]) {
  try {
    const allKbs = await DocumentService.getKnowledgeBases();
    const kbTitleById = new Map(
      [...(allKbs.team ?? []), ...(allKbs.personal ?? []), ...(allKbs.public ?? [])].map((kb) => [
        kb.id,
        kb.title
      ])
    );
    sources.forEach((source) => {
      if (source.kbId && !source.kbTitle) {
        source.kbTitle = kbTitleById.get(source.kbId);
      }
    });
  } catch {
    /* optional enrichment */
  }
}

export async function generateCitationsAndResults(
  query: string,
  { webSearchEnabled }: SearchQueryOptions
): Promise<{
  sources: SearchSource[];
  relatedMedia: import('../types').SearchRelatedMedia;
  responseText: string;
}> {
  const lower = query.toLowerCase();
  let localSources: SearchSource[] = [];
  let searchDocs: Awaited<ReturnType<typeof DocumentService.searchAll>>['docs'] = [];

  try {
    const searchResults = await DocumentService.searchAll(query);
    searchDocs = searchResults.docs;
    localSources = buildLocalSources(searchResults);
  } catch (e) {
    console.warn('Failed to retrieve actual documents during search routing', e);
  }

  if (localSources.filter((source) => source.type === 'doc').length === 0) {
    const okfSources = await buildOkfSources(query);
    localSources = [...okfSources, ...localSources].slice(0, MAX_DOC_SOURCES + MAX_OKF_SOURCES);
  }

  await enrichKbTitles(localSources);

  const webSources = webSearchEnabled ? await buildWebSources(query) : [];
  if (webSearchEnabled && webSources.length === 0) {
    console.warn(
      '[SearchQueryEngine] Web search is enabled but no public web hits were returned. Enable SDKWORK_KNOWLEDGEBASE_PUBLIC_WEB_SEARCH_ENABLED on the backend or configure SDKWORK_KNOWLEDGEBASE_SEARXNG_BASE_URL.',
    );
  }

  const sources = [...localSources, ...webSources];
  const relatedMedia = buildRelatedMedia(query, searchDocs, webSearchEnabled);

  let responseText = '';
  const sourcesText = sources
    .map((s, i) => {
      const kind =
        s.type === 'doc' ? '知识库文件' : s.type === 'kb' ? '知识库目录' : '网络链接';
      return `[${i + 1}] (${kind}) "${s.title}" - ${s.snippet}`;
    })
    .join('\n');

  try {
    if (isKnowledgebaseApiAvailable()) {
      responseText = await synthesizeKnowledgeSearchAnswer(query, sourcesText);
      return { sources, relatedMedia, responseText };
    }

    const localHint =
      localSources.length > 0
        ? `已匹配 ${localSources.filter((s) => s.type === 'doc').length} 个知识库文件。`
        : '本次未匹配到知识库文件。';
    const webHint =
      webSources.length > 0
        ? `已补充 ${webSources.length} 个网络来源。`
        : webSearchEnabled
          ? '联网检索已开启，但当前环境未返回公共网页来源（请在后端启用 SDKWORK_KNOWLEDGEBASE_PUBLIC_WEB_SEARCH_ENABLED）。'
          : '';

    const searchPrompt = `你是一个专业的 AI 搜索引擎组件，名为【AI 融合检索与认知底座】。
用户发出提问/搜索词: "${query}"

检索摘要：${localHint} ${webHint}

下面是辅助参考事实（序号必须与正文引用 [n] 严格对齐）：
${sourcesText || '（无可用引用来源）'}

请你写出一篇极其优美、结构化、富有逻辑且说服力的专家级中文回答：
1. 你的语气要真诚、中性、富有行业见解，杜绝假大空无病呻吟。
2. 必须在回答的对应事实后面标出参考序号。
3. 如果有知识库文件，请明确写“在您知识库中的《xxx》…”并引用对应序号。
4. 请使用清晰的 Markdown 格式输出。
5. 在回答结尾，为用户友好提供 2-3 个深度的“追问方向”作为无序列表结束。
6. 不要输出 \`\`\`markdown 代码块包裹。`;

    const response = await fetch('/api/ai/action', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        action: 'custom',
        text: query,
        customPrompt: searchPrompt,
        context: '用户正在执行全局智能检索，当前时间：' + new Date().toLocaleString()
      })
    });

    if (response.ok) {
      const data = await response.json();
      if (data.result && data.result.length > 50) {
        responseText = data.result;
        return { sources, relatedMedia, responseText };
      }
    }
    throw new Error('No robust prompt returned, falling back');
  } catch (e) {
    console.warn(
      'API route failed during search query synthesis or key missing, applying professional generation fallback:',
      e
    );
    await new Promise((r) => setTimeout(r, 600));

    const isLocalSearch = localSources.some((s) => s.type === 'doc');
    const localDocName = localSources.find((s) => s.type === 'doc')?.title || '产品路线图';

    if (
      lower.includes('desktop') ||
      lower.includes('桌面') ||
      lower.includes('组件') ||
      lower.includes('widget') ||
      lower.includes('设计')
    ) {
      responseText = buildDesktopFallback(isLocalSearch, localDocName);
    } else {
      responseText = buildDefaultFallback(query, sources.length, isLocalSearch, localDocName);
    }
  }

  return { sources, relatedMedia, responseText };
}

function buildDesktopFallback(isLocalSearch: boolean, localDocName: string): string {
  return `### 🎨 桌面级智能单体组件与 AI 协同规范

在瞬息万变的智能科技浪潮中，大语言模型与自主小组件的融合正在以前所未有的速度充盈人们的生产力日常。

#### 📁 深度知识库关联 (${isLocalSearch ? `已关联《${localDocName}》` : '未发现本地文件'})
${isLocalSearch ? `根据您知识库中存储的 **《${localDocName}》** 文件，团队此前已经规划了前端样式与组件化重塑的排版打底。` : '建议建立单独的"设计方案"文件来进行沉淀。'}

---

💡 **您可以继续尝试深入探索：**
*   *如何将搜索对话结果一键保存至知识库笔记？*
*   *紧凑聊天输入框在暗色模式下如何配色？*`;
}

function buildDefaultFallback(
  query: string,
  sourceCount: number,
  isLocalSearch: boolean,
  localDocName: string
): string {
  return `### 🔍 关于"${query}"的融合检索洞察

已基于**本地知识库检索**梳理 ${sourceCount} 个可用来源。

#### 📂 本地知识库 (${isLocalSearch ? '已关联' : '无匹配'})
${isLocalSearch ? `分析显示，此讨论对齐本地文档 **《${localDocName}》**。` : '当前未命中知识库文件，请尝试补充文档或调整检索词。'}

---

🔍 **建议您接下来追问：**
*   *本地知识库中有哪些相近文章值得融汇？*
*   *如何将 GPT-style 输入框支持拖入 PDF 联合导读？*`;
}
