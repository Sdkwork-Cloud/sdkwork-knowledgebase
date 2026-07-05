import type { KnowledgebaseHostContext } from './knowledgebaseHostPresentation';
import { buildHostKnowledgeWindowRequest } from './hostKnowledgeWindowRequest';
import { resolveKnowledgebaseHostEmbedUrl } from './resolveKnowledgebaseHostEmbedUrl';
import { getKnowledgebasePcSdkPorts } from './sdkPorts';

export interface OpenKnowledgebaseDesktopWindowOptions {
  title?: string;
  context?: KnowledgebaseHostContext;
}

export async function openKnowledgebaseDesktopWindow(
  options: OpenKnowledgebaseDesktopWindowOptions = {},
): Promise<boolean> {
  const ports = getKnowledgebasePcSdkPorts();
  const openHostKnowledgeWindow = ports.openHostKnowledgeWindow;
  if (typeof openHostKnowledgeWindow !== 'function') {
    return false;
  }

  const request = buildHostKnowledgeWindowRequest({
    url: resolveKnowledgebaseHostEmbedUrl(options.context),
    title: options.title,
    context: options.context,
  });

  return openHostKnowledgeWindow(request);
}
