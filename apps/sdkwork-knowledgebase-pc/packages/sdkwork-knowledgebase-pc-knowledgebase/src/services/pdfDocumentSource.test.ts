import { afterEach, describe, expect, it, vi } from 'vitest';
import {
  KnowledgebaseErrorCodes,
  type HostAdapter,
} from 'sdkwork-knowledgebase-pc-core';
import { loadPdfSourceFallback } from './pdfDocumentSource';

function createHostAdapter(overrides: Partial<HostAdapter> = {}): HostAdapter {
  return {
    isNativeHost: false,
    windowControl: vi.fn(),
    openExternal: vi.fn(),
    writeTextToClipboard: vi.fn(),
    fetchBinaryResource: vi.fn(),
    readLocalResource: vi.fn(),
    saveBinaryResource: vi.fn(),
    saveExportFile: vi.fn(),
    revealExportFile: vi.fn(),
    openExportFile: vi.fn(),
    locateExportFile: vi.fn(),
    ...overrides,
  };
}

describe('loadPdfSourceFallback', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it('does not use browser raw fetch when no native binary-resource port is available', async () => {
    const fetchSpy = vi.fn(async () => {
      throw new Error('raw fetch must not be called');
    });
    vi.stubGlobal('fetch', fetchSpy);
    vi.stubGlobal('window', { location: { origin: 'https://app.local' } });

    await expect(
      loadPdfSourceFallback('https://example.com/guide.pdf', createHostAdapter()),
    ).rejects.toMatchObject({
      code: KnowledgebaseErrorCodes.DESKTOP_ONLY,
    });
    expect(fetchSpy).not.toHaveBeenCalled();
  });

  it('loads remote PDF bytes through the native host binary-resource port', async () => {
    const host = createHostAdapter({
      isNativeHost: true,
      fetchBinaryResource: vi.fn(async () => ({
        dataBase64: 'JVBERi0x',
        mimeType: 'application/pdf',
        byteLength: 6,
      })),
    });

    const source = await loadPdfSourceFallback('https://example.com/guide.pdf', host);

    expect(source.kind).toBe('bytes');
    expect(Array.from(source.kind === 'bytes' ? source.data : new Uint8Array())).toEqual([
      37,
      80,
      68,
      70,
      45,
      49,
    ]);
    expect(host.fetchBinaryResource).toHaveBeenCalledWith('https://example.com/guide.pdf');
  });
});
