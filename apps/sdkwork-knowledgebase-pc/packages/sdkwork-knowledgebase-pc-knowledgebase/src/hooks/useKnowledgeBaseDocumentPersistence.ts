import { useCallback, useEffect, useRef, type Dispatch, type SetStateAction } from 'react';
import { useTranslation } from 'react-i18next';
import type { DocumentMeta } from '../services/document';
import { DocumentService } from '../services/document';
import { toastKnowledgebaseError } from '../components/ui/toastKnowledgebaseError';

const SAVE_DEBOUNCE_MS = 800;

interface UseKnowledgeBaseDocumentPersistenceOptions {
  activeDoc: DocumentMeta | null;
  docs: unknown[];
  loadingDocs: boolean;
  setOpenDocs: Dispatch<SetStateAction<DocumentMeta[]>>;
  setActiveDoc: Dispatch<SetStateAction<DocumentMeta | null>>;
  setDocContent: Dispatch<SetStateAction<string>>;
}

export function useKnowledgeBaseDocumentPersistence({
  activeDoc,
  docs,
  loadingDocs,
  setOpenDocs,
  setActiveDoc,
  setDocContent,
}: UseKnowledgeBaseDocumentPersistenceOptions) {
  const { t } = useTranslation(['kb', 'common', 'errors']);
  const pendingByDocRef = useRef<Map<string, string>>(new Map());
  const timersByDocRef = useRef<Map<string, ReturnType<typeof setTimeout>>>(new Map());
  const saveInFlightRef = useRef<Map<string, Promise<void>>>(new Map());
  const activeDocIdRef = useRef<string | null>(null);

  const flushDocumentSave = useCallback(async (docId: string) => {
    const timer = timersByDocRef.current.get(docId);
    if (timer) {
      clearTimeout(timer);
      timersByDocRef.current.delete(docId);
    }

    const content = pendingByDocRef.current.get(docId);
    if (content === undefined) {
      return;
    }
    pendingByDocRef.current.delete(docId);

    const existing = saveInFlightRef.current.get(docId);
    if (existing) {
      await existing.catch(() => undefined);
    }

    const savePromise = (async () => {
      try {
        await DocumentService.saveDocumentContent(docId, content);
      } catch (error) {
        toastKnowledgebaseError(error, t);
        throw error;
      }
    })();

    saveInFlightRef.current.set(docId, savePromise);
    try {
      await savePromise;
    } finally {
      if (saveInFlightRef.current.get(docId) === savePromise) {
        saveInFlightRef.current.delete(docId);
      }
    }
  }, [t]);

  const scheduleDocumentSave = useCallback((docId: string, content: string) => {
    pendingByDocRef.current.set(docId, content);
    const existingTimer = timersByDocRef.current.get(docId);
    if (existingTimer) {
      clearTimeout(existingTimer);
    }
    timersByDocRef.current.set(
      docId,
      setTimeout(() => {
        void flushDocumentSave(docId);
      }, SAVE_DEBOUNCE_MS),
    );
  }, [flushDocumentSave]);

  const flushAllPendingSaves = useCallback(async () => {
    const docIds = new Set<string>([
      ...pendingByDocRef.current.keys(),
      ...timersByDocRef.current.keys(),
    ]);
    await Promise.all(Array.from(docIds, (docId) => flushDocumentSave(docId)));
  }, [flushDocumentSave]);

  const handleContentChange = useCallback((newContent: string) => {
    if (!activeDoc) {
      return;
    }
    scheduleDocumentSave(activeDoc.id, newContent);
  }, [activeDoc, scheduleDocumentSave]);

  useEffect(() => {
    const previousDocId = activeDocIdRef.current;
    const nextDocId = activeDoc?.id ?? null;
    if (previousDocId && previousDocId !== nextDocId) {
      void flushDocumentSave(previousDocId);
    }
    activeDocIdRef.current = nextDocId;
  }, [activeDoc, flushDocumentSave]);

  useEffect(() => {
    const handleBeforeUnload = () => {
      for (const docId of pendingByDocRef.current.keys()) {
        const timer = timersByDocRef.current.get(docId);
        if (timer) {
          clearTimeout(timer);
          timersByDocRef.current.delete(docId);
        }
        const content = pendingByDocRef.current.get(docId);
        if (content !== undefined) {
          pendingByDocRef.current.delete(docId);
          void DocumentService.saveDocumentContent(docId, content).catch(() => undefined);
        }
      }
    };

    window.addEventListener('beforeunload', handleBeforeUnload);
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
      void flushAllPendingSaves();
    };
  }, [flushAllPendingSaves]);

  useEffect(() => {
    if (!docs || docs.length === 0) {
      if (!loadingDocs) {
        setOpenDocs([]);
        setActiveDoc(null);
        setDocContent('');
      }
      return;
    }

    const flatIds = new Set<string>();
    const traverse = (items: any[]) => {
      items.forEach((item) => {
        flatIds.add(item.id);
        if (item.type === 'folder' && item.children) {
          traverse(item.children);
        }
      });
    };
    traverse(docs as any[]);

    setOpenDocs((prev) => {
      const filtered = prev.filter((doc) => flatIds.has(doc.id));
      if (filtered.length !== prev.length) {
        return filtered;
      }
      return prev;
    });

    if (activeDoc && !flatIds.has(activeDoc.id)) {
      void flushDocumentSave(activeDoc.id).finally(() => {
        setActiveDoc(null);
        setDocContent('');
      });
    }
  }, [
    activeDoc,
    docs,
    flushDocumentSave,
    loadingDocs,
    setActiveDoc,
    setDocContent,
    setOpenDocs,
  ]);

  return { handleContentChange };
}
