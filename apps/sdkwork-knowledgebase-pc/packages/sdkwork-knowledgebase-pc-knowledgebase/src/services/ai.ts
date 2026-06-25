import { isBlank } from '@sdkwork/sdkwork-knowledgebase-pc-commons/stringUtils';
import {
  isKnowledgebaseApiAvailable,
  KnowledgebaseErrorCodes,
  shouldUseKnowledgebaseDemoFallback,
  throwKnowledgebaseError,
} from 'sdkwork-knowledgebase-pc-core';

import type { McpToolCall } from './mcpAgent';

import {
  buildEditorActionPrompt,
  sendKnowledgeAgentMessage,
  synthesizeKnowledgeSearchAnswer,
} from './knowledgeAgentChatService';
import * as KnowledgeMediaTaskService from './knowledgeMediaTaskService';

type ChatToolCallPayload = Pick<McpToolCall, 'name' | 'arguments'> &
  Partial<Pick<McpToolCall, 'status' | 'result'>>;

export class AIService {
  static async handleAIAction(
    action: string,
    text: string,
    context: string,
    customPrompt?: string,
  ): Promise<string> {
    if (isKnowledgebaseApiAvailable()) {
      const prompt = buildEditorActionPrompt(action, text, context, customPrompt);
      return sendKnowledgeAgentMessage(prompt);
    }

    if (!shouldUseKnowledgebaseDemoFallback()) {
      throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE_AI);
    }

    await new Promise((resolve) => setTimeout(resolve, 600));
    const textSnippet = text.length > 60 ? `${text.substring(0, 60)}...` : text;

    switch (action) {
      case 'summary':
        return `### 📝 内容精华摘要提炼\n\n根据您选中的文本内容，提炼出以下核心要点：\n\n1. **核心主题**：提出了针对近期关键要素的核心见解。\n2. **关键事实**：基于当前选中的片段进行信息归并与结构层提炼，消除了冗余信息。\n3. **核心主张**：建议采取敏捷的设计原则和模块化的封装开发行为。\n\n> 💡 *选中文本摘要完毕。原字数：${text.length}字，提炼后压缩比：35%*`;
      case 'translate': {
        const isChinese = /[\u4e00-\u9fa5]/.test(text);
        if (isChinese) {
          return `### 🌐 Translated Content (EN)\n\n*The original text has been professionally translated as follows:*\n\n"${textSnippet}"  \n➡  \n"**The selected text primarily deliberates on building a highly extensible RAG architecture, with unified state management and responsive component boundaries to facilitate rapid iteration of downstream service SDK integrations.**"`;
        }
        return `### 🌐 译文参考 (ZH)\n\n*源文本已自动进行学术级流畅重构翻译：*\n\n"${textSnippet}"  \n➡  \n"**选中的代码及文本主要用于描述一套优雅的反应式状态流机制。在状态生命周期中，多维组件能够确保高度的解耦与完美的视图呈现。**"`;
      }
      case 'expand':
        return `### 📈 正文深度扩写\n\n在“**${textSnippet}**”的原意基础上，进行了深层次的思想拓展与表达优化：\n\n从架构设计与长期维护的角度来看，这一论点不仅解决了当下的核心诉求，还为系统的横向无限扩容打下了极其坚实的基础。`;
      case 'polish':
        return `### ✨ 文字润色与措辞雕琢\n\n*已将您选中的段落升华至黄金级阅读语感：*\n\n在“${textSnippet}”，我们对其遣词造句、节奏重音进行了全面重塑。`;
      case 'continue':
        return `${text}\n\n在上述论点往后延展的自然语境中，接下来我们非常有必要探讨如何通过端到端的动态对齐和持续敏捷监控来保障这一设计的高保真度落地。`;
      case 'custom':
        return `### 🤖 自定义指令智能执行\n\n*根据指令「${customPrompt || '润色与格式化'}」对文本进行的深度处理已完成。*`;
      default:
        return `### ✨ 处理成功\n\n已经针对当前选中文本完成了智能精细化加工，您可直接一键覆盖文本。`;
    }
  }

  static async generateChatResponse(
    message: string,
    context?: string,
    references?: string,
  ): Promise<{ result: string; toolCalls?: ChatToolCallPayload[] }> {
    if (isKnowledgebaseApiAvailable()) {
      const prompt = buildEditorActionPrompt(
        'chat',
        message,
        [context, references].filter((entry) => !isBlank(entry)).join('\n'),
      );
      const result = await sendKnowledgeAgentMessage(prompt);
      return { result, toolCalls: [] };
    }

    if (!shouldUseKnowledgebaseDemoFallback()) {
      throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE_CHAT);
    }

    await new Promise((resolve) => setTimeout(resolve, 800));
    const query = message.trim().toLowerCase();

    if (
      query.includes('诊断')
      || query.includes('检查')
      || query.includes('排版分析')
      || query.includes('敏感词')
      || query.includes('字数统计')
    ) {
      return {
        result: '我已经为您启动了 editorial diagnostic 质量自检服务。',
        toolCalls: [
          { name: 'run_editorial_diagnostic', arguments: { triggerMode: 'conversational' }, status: 'success' },
        ],
      };
    }

    return {
      result: `### 🤖 微信智能体助手 (Offline Demo)\n\n您好！当前未连接 Knowledgebase API，以下为本地演示回复。\n\n如需真实 AI 能力，请登录并启动后端服务。`,
      toolCalls: [],
    };
  }

  static async streamRewrite(htmlContent: string, onChunk: (chunk: string) => void): Promise<string> {
    if (isKnowledgebaseApiAvailable()) {
      const rewritten = await sendKnowledgeAgentMessage(
        `请将以下 HTML 内容重写为结构更清晰、语气更专业的 Markdown/HTML，保留语义：\n\n${htmlContent}`,
      );
      onChunk(rewritten);
      return rewritten;
    }

    if (!shouldUseKnowledgebaseDemoFallback()) {
      throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE_STREAM_REWRITE);
    }

    const chunks = [
      `<h1>✨ 科技跃迁：深度解构智能化新纪元</h1>`,
      `<p>在瞬息万变的数字化浪潮中，大语言模型与自主智能体的崛起正在以前所未有的速度重谱生产力边界。</p>`,
    ];

    let currentHtml = '';
    for (const part of chunks) {
      currentHtml += part;
      onChunk(currentHtml);
      await new Promise((resolve) => setTimeout(resolve, 120));
    }
    return currentHtml;
  }

  static async speechToText(
    audioUrl: string,
    context?: KnowledgeMediaTaskService.MediaTaskContext,
  ): Promise<string> {
    if (isKnowledgebaseApiAvailable()) {
      return KnowledgeMediaTaskService.runSpeechToTextTask(audioUrl, context);
    }

    if (!shouldUseKnowledgebaseDemoFallback()) {
      throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE_SPEECH);
    }

    await new Promise((resolve) => setTimeout(resolve, 2000));
    return '这是一段语音转文字的离线演示结果。';
  }

  static async generateImage(
    prompt: string,
    aspectMode: string,
    styleMode: string,
    context?: KnowledgeMediaTaskService.MediaTaskContext,
  ): Promise<{ url: string; resolution: string; suggestions: string[]; similars: string[] }> {
    if (isKnowledgebaseApiAvailable()) {
      return KnowledgeMediaTaskService.runImageGenerationTask(
        prompt,
        aspectMode,
        styleMode,
        context,
      );
    }

    if (!shouldUseKnowledgebaseDemoFallback()) {
      throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE_IMAGE);
    }

    await new Promise((resolve) => setTimeout(resolve, 1500));
    return {
      url: 'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?q=80&w=1024&auto=format&fit=crop',
      resolution: '1024x1024',
      suggestions: ['尝试赛博朋克风格', '调整为夜晚时间', '增加更多细节'],
      similars: [],
    };
  }

  static async synthesizeSearchAnswer(query: string, sourcesText: string): Promise<string> {
    if (isKnowledgebaseApiAvailable()) {
      return synthesizeKnowledgeSearchAnswer(query, sourcesText);
    }

    if (!shouldUseKnowledgebaseDemoFallback()) {
      throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE_SEARCH);
    }

    return `根据检索到的资料，关于「${query.trim()}」的要点如下：\n\n${sourcesText || '（暂无可用引用来源）'}`;
  }
}
