import type { DocumentExportContent, DocumentExportContentProvider, DocumentExportSourceKind } from './types';

export const DEFAULT_EXPORT_TITLE = '无标题';

export function stripHtmlText(html: string): string {
  if (!html.trim()) {
    return '';
  }
  const doc = new DOMParser().parseFromString(html, 'text/html');
  return doc.body.textContent?.replace(/\u00a0/g, ' ').trim() ?? '';
}

export function hasExportableContent(content: DocumentExportContent): boolean {
  const markdown = content.markdown?.trim() ?? '';
  const plainText = stripHtmlText(content.html);
  if (content.sourceKind === 'markdown') {
    return Boolean(markdown || plainText);
  }
  return Boolean(plainText || markdown);
}

export function resolveExportMarkdown(content: DocumentExportContent): string {
  const markdown = content.markdown?.trim() ?? '';
  if (markdown) {
    return markdown;
  }
  return stripHtmlText(content.html);
}

export interface EditorExportBinding {
  title: string;
  getHtml: () => string;
  getMarkdown?: () => string;
  getPlainText?: () => string;
  sourceKind?: DocumentExportSourceKind;
  isSourceMode?: boolean;
  sourceCode?: string;
}

function escapeHtmlText(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

export function createEditorExportContentProvider(
  binding: EditorExportBinding | (() => EditorExportBinding | null),
): DocumentExportContentProvider {
  return () => {
    const resolved = typeof binding === 'function' ? binding() : binding;
    if (!resolved) {
      return null;
    }

    const title = resolved.title.trim() || DEFAULT_EXPORT_TITLE;

    if (resolved.isSourceMode && resolved.sourceCode !== undefined) {
      const source = resolved.sourceCode;
      return {
        title,
        html: `<pre><code>${escapeHtmlText(source)}</code></pre>`,
        markdown: source,
        sourceKind: 'markdown',
      };
    }

    const html = resolved.getHtml();
    const markdown =
      resolved.getMarkdown?.() ??
      resolved.getPlainText?.() ??
      stripHtmlText(html);

    return {
      title,
      html,
      markdown,
      sourceKind: resolved.sourceKind ?? (resolved.getMarkdown ? 'markdown' : 'richtext'),
    };
  };
}

function blobToDataUrl(blob: Blob): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result ?? ''));
    reader.onerror = () => reject(reader.error ?? new Error('Failed to read blob.'));
    reader.readAsDataURL(blob);
  });
}

export async function inlineBlobImagesInHtml(html: string): Promise<string> {
  if (!html.includes('blob:')) {
    return html;
  }

  const doc = new DOMParser().parseFromString(`<div id="export-root">${html}</div>`, 'text/html');
  const root = doc.getElementById('export-root');
  if (!root) {
    return html;
  }

  const images = Array.from(root.querySelectorAll('img'));
  await Promise.all(
    images.map(async (image) => {
      const src = image.getAttribute('src');
      if (!src?.startsWith('blob:')) {
        return;
      }
      try {
        const response = await fetch(src);
        const blob = await response.blob();
        image.setAttribute('src', await blobToDataUrl(blob));
      } catch {
        // Keep original src; export may still succeed for other images.
      }
    }),
  );

  return root.innerHTML;
}

export async function prepareExportHtml(html: string): Promise<string> {
  return inlineBlobImagesInHtml(html);
}

const IMAGE_LOAD_TIMEOUT_MS = 8000;

export async function prepareExportImages(container: HTMLElement): Promise<number> {
  const images = Array.from(container.querySelectorAll('img'));
  if (images.length === 0) {
    return 0;
  }

  await Promise.all(
    images.map(async (image) => {
      const src = image.getAttribute('src');
      if (!src?.startsWith('blob:')) {
        return;
      }
      try {
        const response = await fetch(src);
        const blob = await response.blob();
        image.setAttribute('src', await blobToDataUrl(blob));
      } catch {
        // Keep original src; load listener below will count as failure.
      }
    }),
  );

  let failedCount = 0;
  await Promise.all(
    images.map(
      (image) =>
        new Promise<void>((resolve) => {
          let settled = false;
          const finish = (ok: boolean) => {
            if (settled) {
              return;
            }
            settled = true;
            if (!ok) {
              failedCount += 1;
            }
            resolve();
          };

          const timeout = window.setTimeout(() => finish(false), IMAGE_LOAD_TIMEOUT_MS);

          if (image.complete && image.naturalWidth > 0) {
            window.clearTimeout(timeout);
            finish(true);
            return;
          }

          image.addEventListener(
            'load',
            () => {
              window.clearTimeout(timeout);
              finish(true);
            },
            { once: true },
          );
          image.addEventListener(
            'error',
            () => {
              window.clearTimeout(timeout);
              finish(false);
            },
            { once: true },
          );
        }),
    ),
  );
  return failedCount;
}
