import {
  KnowledgebaseErrorCodes,
  throwKnowledgebaseError,
} from 'sdkwork-knowledgebase-pc-core';

import type { WechatArticle } from './wechat';
import { getActiveEphemeralFixedKnowledgebaseWorkspaceSpaceId } from '../workspaceMode';

export interface McpToolCall {
  name: string;
  arguments: Record<string, unknown>;
  status?: 'running' | 'success' | 'failed';
  result?: string;
}

export interface McpAgentResponse {
  thinkingText: string;
  toolCalls: McpToolCall[];
  responseText: string;
}

export class McpAgentService {
  static async processUserQuery(
    _query: string,
    _currentArticle?: WechatArticle,
  ): Promise<McpAgentResponse> {
    if (getActiveEphemeralFixedKnowledgebaseWorkspaceSpaceId() !== null) {
      throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE_CHAT);
    }

    throwKnowledgebaseError(KnowledgebaseErrorCodes.API_UNAVAILABLE_SDK);
  }
}
