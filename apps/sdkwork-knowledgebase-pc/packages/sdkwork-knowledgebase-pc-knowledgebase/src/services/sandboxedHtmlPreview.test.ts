/** @vitest-environment jsdom */

import { describe, expect, it } from 'vitest';

import { buildSandboxedHtmlPreview } from './sandboxedHtmlPreview';

describe('sandboxed HTML preview', () => {
  it('removes active content and applies a deny-by-default policy', () => {
    const preview = buildSandboxedHtmlPreview(`
      <html><head><script>parent.__TAURI__</script></head>
      <body onload="alert(1)"><form action="https://example.com"><input /></form></body></html>
    `);

    expect(preview).toContain("default-src 'none'");
    expect(preview).toContain("script-src 'none'");
    expect(preview).not.toContain('<script');
    expect(preview).not.toContain('onload=');
    expect(preview).not.toContain('<form');
  });
});
