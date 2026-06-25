import { describe, expect, it } from 'vitest';
import { sanitizeEditorHtml } from './htmlSanitizer';

describe('sanitizeEditorHtml', () => {
  it('removes script tags and event handlers', () => {
    const sanitized = sanitizeEditorHtml('<p onclick="alert(1)">Hello</p><script>alert(1)</script>');
    expect(sanitized).toContain('Hello');
    expect(sanitized.toLowerCase()).not.toContain('<script');
    expect(sanitized.toLowerCase()).not.toContain('onclick');
  });

  it('strips unsafe inline styles', () => {
    const sanitized = sanitizeEditorHtml('<p style="background-image: url(javascript:alert(1))">x</p>');
    expect(sanitized).not.toContain('javascript:');
  });

  it('preserves safe structural markup', () => {
    const sanitized = sanitizeEditorHtml('<h1>Title</h1><p><strong>Body</strong></p>');
    expect(sanitized).toContain('<h1>Title</h1>');
    expect(sanitized).toContain('<strong>Body</strong>');
  });
});
