import DOMPurify from 'dompurify';

const PREVIEW_CSP = [
  "default-src 'none'",
  'base-uri \'none\'',
  'connect-src \'none\'',
  'font-src data:',
  'form-action \'none\'',
  'frame-src \'none\'',
  'img-src data: blob:',
  'media-src data: blob:',
  "script-src 'none'",
  "style-src 'unsafe-inline'",
].join('; ');

const FORBIDDEN_PREVIEW_TAGS = [
  'base',
  'button',
  'embed',
  'form',
  'frame',
  'iframe',
  'input',
  'link',
  'object',
  'script',
  'select',
  'textarea',
];

export function buildSandboxedHtmlPreview(html: string): string {
  const sanitized = DOMPurify.sanitize(html, {
    FORBID_ATTR: ['srcdoc'],
    FORBID_TAGS: FORBIDDEN_PREVIEW_TAGS,
    WHOLE_DOCUMENT: true,
  });
  const documentNode = new DOMParser().parseFromString(sanitized, 'text/html');
  documentNode.querySelectorAll('meta[http-equiv="Content-Security-Policy"]').forEach((node) => {
    node.remove();
  });
  const policy = documentNode.createElement('meta');
  policy.httpEquiv = 'Content-Security-Policy';
  policy.content = PREVIEW_CSP;
  documentNode.head.prepend(policy);
  return `<!doctype html>\n${documentNode.documentElement.outerHTML}`;
}
