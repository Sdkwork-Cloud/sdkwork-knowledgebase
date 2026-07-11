import { isBlank } from '@sdkwork/utils';
import {
  isKnowledgebaseApiAvailable,
  KnowledgebaseErrorCodes,
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

function requireAiApi(code: string): void {
  if (!isKnowledgebaseApiAvailable()) {
    throwKnowledgebaseError(code);
  }
}

export class AIService {
  static async handleAIAction(
    action: string,
    text: string,
    context: string,
    customPrompt?: string,
  ): Promise<string> {
    requireAiApi(KnowledgebaseErrorCodes.API_UNAVAILABLE_AI);
    const prompt = buildEditorActionPrompt(action, text, context, customPrompt);
    return sendKnowledgeAgentMessage(prompt);
  }

  static async generateChatResponse(
    message: string,
    context?: string,
    references?: string,
  ): Promise<{ result: string; toolCalls?: ChatToolCallPayload[] }> {
    requireAiApi(KnowledgebaseErrorCodes.API_UNAVAILABLE_CHAT);
    const prompt = buildEditorActionPrompt(
      'chat',
      message,
      [context, references].filter((entry) => !isBlank(entry)).join('\n'),
    );
    const result = await sendKnowledgeAgentMessage(prompt);
    return { result, toolCalls: [] };
  }

  static async streamRewrite(
    htmlContent: string,
    onChunk: (chunk: string) => void,
  ): Promise<string> {
    requireAiApi(KnowledgebaseErrorCodes.API_UNAVAILABLE_STREAM_REWRITE);
    const rewritten = await sendKnowledgeAgentMessage(
      `请将以下 HTML 内容重写为结构更清晰、语气更专业的 Markdown/HTML，保留语义：\n\n${htmlContent}`,
    );
    onChunk(rewritten);
    return rewritten;
  }

  static async speechToText(
    audioUrl: string,
    context?: KnowledgeMediaTaskService.MediaTaskContext,
  ): Promise<string> {
    requireAiApi(KnowledgebaseErrorCodes.API_UNAVAILABLE_SPEECH);
    return KnowledgeMediaTaskService.runSpeechToTextTask(audioUrl, context);
  }

  static async generateImage(
    prompt: string,
    aspectMode: string,
    styleMode: string,
    context?: KnowledgeMediaTaskService.MediaTaskContext,
  ): Promise<{ url: string; resolution: string; suggestions: string[]; similars: string[] }> {
    requireAiApi(KnowledgebaseErrorCodes.API_UNAVAILABLE_IMAGE);
    return KnowledgeMediaTaskService.runImageGenerationTask(
      prompt,
      aspectMode,
      styleMode,
      context,
    );
  }

  static async synthesizeSearchAnswer(query: string, sourcesText: string): Promise<string> {
    requireAiApi(KnowledgebaseErrorCodes.API_UNAVAILABLE_SEARCH);
    return synthesizeKnowledgeSearchAnswer(query, sourcesText);
  }
}
