const BROWSER_BASE_PATH_SEGMENT_PATTERN = /^[A-Za-z0-9._~-]+$/u;

/**
 * Normalizes the public browser mount path shared by Vite and BrowserRouter.
 * It intentionally accepts only plain, percent-decoding-free URL segments so
 * configuration cannot smuggle an origin, query, fragment, or traversal path
 * into client-side routing.
 */
export function normalizeKnowledgebaseBrowserBasePath(value: string | undefined): string {
  const candidate = value?.trim();
  if (!candidate || candidate === '/') {
    return '/';
  }
  if (
    !candidate.startsWith('/')
    || candidate.startsWith('//')
    || candidate.includes('\\')
    || candidate.includes(':')
    || candidate.includes('?')
    || candidate.includes('#')
    || candidate.includes('%')
  ) {
    return '/';
  }

  const segments = candidate.split('/').filter(Boolean);
  if (
    segments.length === 0
    || segments.some((segment) => (
      segment === '.'
      || segment === '..'
      || !BROWSER_BASE_PATH_SEGMENT_PATTERN.test(segment)
    ))
  ) {
    return '/';
  }

  return '/' + segments.join('/');
}

/** Vite requires a trailing slash for non-root public mount paths. */
export function toKnowledgebaseViteBasePath(value: string | undefined): string {
  const basePath = normalizeKnowledgebaseBrowserBasePath(value);
  return basePath === '/' ? '/' : basePath + '/';
}
