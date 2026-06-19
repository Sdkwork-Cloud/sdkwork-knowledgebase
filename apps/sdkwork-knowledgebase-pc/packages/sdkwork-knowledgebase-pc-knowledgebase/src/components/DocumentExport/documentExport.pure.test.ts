import assert from 'node:assert/strict';
import { describe, it } from 'node:test';
import { buildExportFileName, sanitizeExportBaseName } from './documentExportSave';
import { encodeBytesBase64 } from './exportRuntime';

describe('documentExportSave', () => {
  it('sanitizeExportBaseName replaces invalid path characters', () => {
    assert.equal(sanitizeExportBaseName('note:draft?.pdf'), 'note_draft_.pdf');
  });

  it('sanitizeExportBaseName falls back when title is blank', () => {
    assert.equal(sanitizeExportBaseName('   '), 'document');
  });

  it('buildExportFileName appends extension once', () => {
    assert.equal(buildExportFileName('My Note', 'pdf'), 'My Note.pdf');
    assert.equal(buildExportFileName('My Note.pdf', 'pdf'), 'My Note.pdf');
  });
});

describe('exportRuntime.encodeBytesBase64', () => {
  it('round-trips small payloads', () => {
    const input = new Uint8Array([72, 101, 108, 108, 111]);
    const encoded = encodeBytesBase64(input);
    assert.equal(encoded, 'SGVsbG8=');
  });

  it('handles payloads larger than one chunk', () => {
    const input = new Uint8Array(0x8001);
    input.fill(65);
    const encoded = encodeBytesBase64(input);
    assert.equal(Buffer.from(encoded, 'base64').length, input.length);
  });
});
