import { describe, expect, it } from 'vitest';

import { decodeBinaryResourcePayload } from './hostAdapter';

describe('desktop host resource bounds', () => {
  it('rejects a payload that declares more than the desktop resource limit', () => {
    expect(() => decodeBinaryResourcePayload({
      byteLength: 32 * 1024 * 1024 + 1,
      dataBase64: '',
    })).toThrow();
  });

  it('rejects a payload whose decoded size does not match its metadata', () => {
    expect(() => decodeBinaryResourcePayload({
      byteLength: 1,
      dataBase64: '',
    })).toThrow();
  });
});
