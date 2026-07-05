import { isBlank, trim } from '@sdkwork/utils';

import type { KnowledgebaseHostContext } from './knowledgebaseHostPresentation';

export interface HostKnowledgeWindowRequest {
  url: string;
  title: string;
  label: string;
}

export function resolveKnowledgeWindowLabel(context?: KnowledgebaseHostContext): string {
  const groupId = context?.groupId;
  if (!isBlank(groupId)) {
    return `knowledge-group-${trim(groupId!).replace(/[^a-zA-Z0-9_-]+/g, '-')}`;
  }
  return 'knowledge-host';
}

export function buildHostKnowledgeWindowRequest(options: {
  url: string;
  title?: string;
  context?: KnowledgebaseHostContext;
}): HostKnowledgeWindowRequest {
  const resolvedTitle = !isBlank(options.title)
    ? trim(options.title!)
    : !isBlank(options.context?.groupName)
      ? trim(options.context!.groupName!)
      : 'Knowledge Base';

  return {
    url: trim(options.url),
    title: resolvedTitle,
    label: resolveKnowledgeWindowLabel(options.context),
  };
}
