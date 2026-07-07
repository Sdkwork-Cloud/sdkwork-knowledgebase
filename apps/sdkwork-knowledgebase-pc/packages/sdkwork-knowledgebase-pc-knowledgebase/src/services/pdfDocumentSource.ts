import {
  KnowledgebaseErrorCodes,
  throwKnowledgebaseError,
} from 'sdkwork-knowledgebase-pc-core';
import {
  decodeBinaryResourcePayload,
  type HostAdapter,
} from 'sdkwork-knowledgebase-pc-core/host/hostAdapter';

export type PdfDocumentSource =
  | { kind: 'url'; url: string }
  | { kind: 'bytes'; data: Uint8Array };

/** Inline or navigable URL sources that react-pdf can load directly. */
export function isDirectPdfUrl(source: string): boolean {
  const trimmed = source.trim();
  return (
    trimmed.startsWith('blob:') ||
    trimmed.startsWith('data:') ||
    trimmed.startsWith('http://') ||
    trimmed.startsWith('https://') ||
    trimmed.startsWith('//') ||
    trimmed.startsWith('/')
  );
}

/** OS filesystem paths that must be read through the desktop host. */
export function isLocalFilePath(source: string): boolean {
  const trimmed = source.trim();
  if (!trimmed) return false;
  if (trimmed.startsWith('file://')) return true;
  if (/^[a-zA-Z]:[\\/]/.test(trimmed)) return true;
  if (trimmed.startsWith('\\\\')) return true;

  // Unix absolute paths outside SPA asset routes.
  if (trimmed.startsWith('/') && !isAppAssetPath(trimmed)) {
    return /^\/(?:Users|home|var|tmp|opt|mnt|private|Volumes)\//.test(trimmed);
  }

  return false;
}

function isAppAssetPath(source: string): boolean {
  return /^\/(?:samples|assets|static|app|api|backend|knowledge|admin|files|media)(?:\/|$)/i.test(
    source.trim()
  );
}

export function normalizePdfUrl(source: string): string {
  const trimmed = source.trim();
  if (!trimmed) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.PDF_URL_REQUIRED);
  }

  if (
    trimmed.startsWith('blob:') ||
    trimmed.startsWith('data:') ||
    trimmed.startsWith('http://') ||
    trimmed.startsWith('https://')
  ) {
    return trimmed;
  }

  if (trimmed.startsWith('//')) {
    return `${globalThis.location?.protocol ?? 'https:'}${trimmed}`;
  }

  return new URL(trimmed, globalThis.location?.origin ?? 'http://localhost').href;
}

export function resolveInitialPdfSource(source: string | undefined): PdfDocumentSource | null {
  const trimmed = source?.trim();
  if (!trimmed) return null;

  if (isLocalFilePath(trimmed)) {
    return null;
  }

  if (isDirectPdfUrl(trimmed)) {
    return { kind: 'url', url: normalizePdfUrl(trimmed) };
  }

  // Bare relative paths such as "docs/guide.pdf".
  return { kind: 'url', url: normalizePdfUrl(trimmed) };
}

async function fetchViaNativeHost(url: string, host: HostAdapter): Promise<Uint8Array> {
  const payload = await host.fetchBinaryResource(url);
  return decodeBinaryResourcePayload(payload);
}

async function readLocalPath(path: string, host: HostAdapter): Promise<Uint8Array> {
  const payload = await host.readLocalResource(path);
  return decodeBinaryResourcePayload(payload);
}

/** Load local filesystem PDF bytes (desktop host required). */
export async function loadLocalPdfSource(
  source: string,
  host: HostAdapter
): Promise<PdfDocumentSource> {
  if (!host.isNativeHost) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.DESKTOP_ONLY);
  }
  return { kind: 'bytes', data: await readLocalPath(source, host) };
}

/**
 * Fallback when direct URL rendering fails (CORS, blocked network, etc.).
 * Keeps URL as the primary path; only escalates when display fails.
 */
export async function loadPdfSourceFallback(
  source: string,
  host: HostAdapter
): Promise<PdfDocumentSource> {
  const trimmed = source.trim();
  if (!trimmed) {
    throwKnowledgebaseError(KnowledgebaseErrorCodes.PDF_URL_REQUIRED);
  }

  if (isLocalFilePath(trimmed)) {
    return loadLocalPdfSource(trimmed, host);
  }

  const url = normalizePdfUrl(trimmed);

  if (host.isNativeHost) {
    try {
      return { kind: 'bytes', data: await fetchViaNativeHost(url, host) };
    } catch (nativeError) {
      console.warn('Native PDF fetch failed, retrying via WebView fetch.', nativeError);
    }
  }

  throwKnowledgebaseError(KnowledgebaseErrorCodes.DESKTOP_ONLY);
}

export function toReactPdfFile(source: PdfDocumentSource): string | Uint8Array {
  if (source.kind === 'url') {
    return source.url;
  }
  return source.data;
}
