import { afterEach, describe, expect, it, vi } from 'vitest';

import type { DocumentMeta } from './document';
import { EphemeralTabCacheService } from './tabService';

const groupDocument = {
  id: 'group-document-1',
  kbId: 'group-space-42',
  title: 'Group document',
  type: 'richtext',
} as DocumentMeta;

const originalLocalStorage = Object.getOwnPropertyDescriptor(globalThis, 'localStorage');

afterEach(() => {
  if (originalLocalStorage) {
    Object.defineProperty(globalThis, 'localStorage', originalLocalStorage);
  } else {
    Reflect.deleteProperty(globalThis, 'localStorage');
  }
});

describe('EphemeralTabCacheService', () => {
  it('keeps fixed group tab metadata in memory and clears it on disposal', () => {
    const localStorage = {
      getItem: vi.fn(),
      setItem: vi.fn(),
    };
    Object.defineProperty(globalThis, 'localStorage', {
      configurable: true,
      value: localStorage,
    });

    const cache = new EphemeralTabCacheService();
    cache.initKb('group-space-42');
    cache.openDoc('group-space-42', groupDocument);

    expect(cache.getOpenDocs('group-space-42')).toEqual([groupDocument]);
    expect(cache.getActiveDocId('group-space-42')).toBe('group-document-1');
    expect(localStorage.getItem).not.toHaveBeenCalled();
    expect(localStorage.setItem).not.toHaveBeenCalled();

    cache.dispose();

    expect(cache.getOpenDocs('group-space-42')).toEqual([]);
    expect(cache.getActiveDocId('group-space-42')).toBeNull();
  });
});
