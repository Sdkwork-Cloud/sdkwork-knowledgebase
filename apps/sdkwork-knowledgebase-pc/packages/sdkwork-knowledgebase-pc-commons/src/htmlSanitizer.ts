import DOMPurify from 'dompurify';

const EDITOR_ALLOWED_TAGS = [
  'p', 'br', 'strong', 'em', 's', 'u', 'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
  'blockquote', 'ul', 'ol', 'li', 'a', 'img', 'code', 'pre', 'span', 'div',
  'table', 'thead', 'tbody', 'tr', 'th', 'td', 'hr', 'sub', 'sup',
];

const EDITOR_ALLOWED_ATTR = [
  'href', 'target', 'rel', 'src', 'alt', 'title', 'class', 'style',
  'width', 'height', 'colspan', 'rowspan', 'data-type', 'data-id',
  'data-title', 'data-image-url', 'data-nickname', 'data-app-id',
  'data-page-path', 'controls',
];

const UNSAFE_STYLE_PATTERNS = [
  /url\s*\(/i,
  /expression\s*\(/i,
  /javascript:/i,
  /@import/i,
  /behavior\s*:/i,
  /-moz-binding/i,
];

function isUnsafeInlineStyle(value: string): boolean {
  const normalized = value.trim().toLowerCase();
  return UNSAFE_STYLE_PATTERNS.some((pattern) => pattern.test(normalized));
}

let editorSanitizerHookInstalled = false;

function ensureEditorSanitizerHook(): void {
  if (editorSanitizerHookInstalled || typeof window === 'undefined') {
    return;
  }
  DOMPurify.addHook('uponSanitizeAttribute', (_node, data) => {
    if (data.attrName === 'style' && isUnsafeInlineStyle(data.attrValue)) {
      data.attrValue = '';
    }
  });
  editorSanitizerHookInstalled = true;
}

export function sanitizeEditorHtml(html: string): string {
  if (!html) {
    return '';
  }
  ensureEditorSanitizerHook();
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS: EDITOR_ALLOWED_TAGS,
    ALLOWED_ATTR: EDITOR_ALLOWED_ATTR,
    ALLOW_DATA_ATTR: true,
    FORBID_TAGS: ['script', 'iframe', 'object', 'embed', 'form', 'input', 'button'],
    FORBID_ATTR: ['onerror', 'onload', 'onclick', 'onmouseover'],
  });
}

export function sanitizePreviewHtml(html: string): string {
  return sanitizeEditorHtml(html);
}

export function escapeHtmlText(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}
