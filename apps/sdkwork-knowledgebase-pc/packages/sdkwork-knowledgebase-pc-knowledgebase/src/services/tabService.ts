import { DocumentMeta } from './document';

export class TabCacheService {
  private static STORAGE_KEY = 'app-tabs-cache-v2';

  /**
   * Defines a unified service and methods to facilitate future desktop application compatibility.
   * Caches the list of opened tabs and the currently active tab for each Knowledge Base.
   */

  private static loadCache(): Record<string, { docs: DocumentMeta[], activeId: string | null }> {
    try {
      const data = localStorage.getItem(this.STORAGE_KEY);
      return data ? JSON.parse(data) : {};
    } catch {
      return {};
    }
  }

  private static saveCache(data: Record<string, { docs: DocumentMeta[], activeId: string | null }>) {
    localStorage.setItem(this.STORAGE_KEY, JSON.stringify(data));
  }

  // Handle a new kb structure gracefully.
  public static initKb(kbId: string): void {
    const cache = this.loadCache();
    if (!cache[kbId]) {
      cache[kbId] = { docs: [], activeId: null };
      this.saveCache(cache);
    }
  }

  public static getOpenDocs(kbId: string): DocumentMeta[] {
    const cache = this.loadCache();
    return cache[kbId]?.docs || [];
  }

  public static getActiveDocId(kbId: string): string | null {
    const cache = this.loadCache();
    return cache[kbId]?.activeId || null;
  }

  public static saveOpenDocs(kbId: string, docs: DocumentMeta[]): void {
    const cache = this.loadCache();
    if (!cache[kbId]) cache[kbId] = { docs: [], activeId: null };
    cache[kbId].docs = docs;
    this.saveCache(cache);
  }

  public static saveActiveDocId(kbId: string, activeId: string | null): void {
    const cache = this.loadCache();
    if (!cache[kbId]) cache[kbId] = { docs: [], activeId: null };
    cache[kbId].activeId = activeId;
    this.saveCache(cache);
  }

  public static openDoc(kbId: string, doc: DocumentMeta): void {
    const cache = this.loadCache();
    if (!cache[kbId]) cache[kbId] = { docs: [], activeId: null };
    
    // Add to open tabs if not a folder and not already present
    if (doc.type !== 'folder') {
      if (!cache[kbId].docs.some(d => d.id === doc.id)) {
        cache[kbId].docs.push(doc);
      }
    }
    
    cache[kbId].activeId = doc.id;
    this.saveCache(cache);
  }

  public static closeDoc(kbId: string, docId: string): { remainingDocs: DocumentMeta[], nextActiveId: string | null } {
    const cache = this.loadCache();
    if (!cache[kbId]) return { remainingDocs: [], nextActiveId: null };

    const index = cache[kbId].docs.findIndex(d => d.id === docId);
    if (index === -1) return { remainingDocs: cache[kbId].docs, nextActiveId: cache[kbId].activeId };

    const remainingDocs = cache[kbId].docs.filter(d => d.id !== docId);
    let nextActiveId = cache[kbId].activeId;

    if (cache[kbId].activeId === docId) {
      if (remainingDocs.length > 0) {
        const newIndex = Math.min(index, remainingDocs.length - 1);
        nextActiveId = remainingDocs[newIndex].id;
      } else {
        nextActiveId = null;
      }
    }

    cache[kbId].docs = remainingDocs;
    cache[kbId].activeId = nextActiveId;
    this.saveCache(cache);

    return { remainingDocs, nextActiveId };
  }

  public static closeOthers(kbId: string, docId: string): { remainingDocs: DocumentMeta[] } {
    const cache = this.loadCache();
    if (!cache[kbId]) return { remainingDocs: [] };

    const docToKeep = cache[kbId].docs.find(d => d.id === docId);
    const remainingDocs = docToKeep ? [docToKeep] : [];
    
    cache[kbId].docs = remainingDocs;
    if (docToKeep) {
      cache[kbId].activeId = docId;
    } else {
      cache[kbId].activeId = null;
    }
    
    this.saveCache(cache);
    return { remainingDocs };
  }

  public static closeAll(kbId: string): void {
    const cache = this.loadCache();
    if (!cache[kbId]) return;
    cache[kbId].docs = [];
    cache[kbId].activeId = null;
    this.saveCache(cache);
  }

  public static closeToRight(kbId: string, docId: string): { remainingDocs: DocumentMeta[], nextActiveId: string | null } {
    const cache = this.loadCache();
    if (!cache[kbId]) return { remainingDocs: [], nextActiveId: null };

    const index = cache[kbId].docs.findIndex(d => d.id === docId);
    if (index === -1) return { remainingDocs: cache[kbId].docs, nextActiveId: cache[kbId].activeId };

    const remainingDocs = cache[kbId].docs.slice(0, index + 1);
    let nextActiveId = cache[kbId].activeId;

    // Check if the currently active doc was removed
    if (nextActiveId && !remainingDocs.some(d => d.id === nextActiveId)) {
        nextActiveId = docId;
    }

    cache[kbId].docs = remainingDocs;
    cache[kbId].activeId = nextActiveId;
    this.saveCache(cache);

    return { remainingDocs, nextActiveId };
  }
}
