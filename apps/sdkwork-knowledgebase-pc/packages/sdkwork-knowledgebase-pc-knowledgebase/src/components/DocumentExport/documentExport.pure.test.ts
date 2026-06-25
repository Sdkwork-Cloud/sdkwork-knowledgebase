import { describe, expect, it } from 'vitest';
import { buildExportFileName, sanitizeExportBaseName } from './documentExportSave';
import { encodeBytesBase64 } from './exportRuntime';

describe('documentExportSave', () => {
  it('sanitizeExportBaseName replaces invalid path characters', () => {
    expect(sanitizeExportBaseName('note:draft?.pdf')).toBe('note_draft_.pdf');
  });

  it('sanitizeExportBaseName falls back when title is blank', () => {
    expect(sanitizeExportBaseName('   ')).toBe('document');
  });

  it('buildExportFileName appends extension once', () => {
    expect(buildExportFileName('My Note', 'pdf')).toBe('My Note.pdf');
    expect(buildExportFileName('My Note.pdf', 'pdf')).toBe('My Note.pdf');
  });
});

describe('exportRuntime.encodeBytesBase64', () => {
  it('round-trips small payloads', () => {
    const input = new Uint8Array([72, 101, 108, 108, 111]);
    expect(encodeBytesBase64(input)).toBe('SGVsbG8=');
  });

  it('handles payloads larger than one chunk', () => {
    const input = new Uint8Array(0x8001);
    input.fill(65);
    expect(encodeBytesBase64(input).length).toBeGreaterThan(0);
  });
});
