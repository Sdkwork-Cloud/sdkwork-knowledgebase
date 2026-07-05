import { isBlank, trim } from '@sdkwork/utils';

import type { KnowledgebaseHostContext } from './knowledgebaseHostPresentation';

function resolveConfiguredEmbedUrl(): string | null {
  const configured = import.meta.env.VITE_SDKWORK_KNOWLEDGEBASE_HOST_EMBED_URL;
  if (typeof configured !== 'string' || isBlank(configured)) {
    return null;
  }
  return trim(configured);
}

function appendHostContext(url: string, context?: KnowledgebaseHostContext): string {
  if (isBlank(context?.groupId) && isBlank(context?.groupName)) {
    return url;
  }

  const baseOrigin = typeof window !== 'undefined' ? window.location.origin : 'http://127.0.0.1';
  const resolved = new URL(url, baseOrigin);
  if (!isBlank(context?.groupId)) {
    resolved.searchParams.set('groupId', trim(context!.groupId!));
  }
  if (!isBlank(context?.groupName)) {
    resolved.searchParams.set('groupName', trim(context!.groupName!));
  }
  return resolved.toString();
}

export function resolveKnowledgebaseHostEmbedUrl(context?: KnowledgebaseHostContext): string {
  const configured = resolveConfiguredEmbedUrl();
  if (configured) {
    return appendHostContext(configured, context);
  }

  if (typeof window === 'undefined') {
    return appendHostContext('/host-embed/knowledge', context);
  }

  const sameOriginEmbed = new URL('/host-embed/knowledge', window.location.origin);
  return appendHostContext(sameOriginEmbed.toString(), context);
}
