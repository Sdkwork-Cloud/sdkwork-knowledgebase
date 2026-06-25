import { useEffect, useState } from 'react';
import { isKnowledgebaseApiAvailable } from 'sdkwork-knowledgebase-pc-core';

import type { DocumentMeta } from '../services/document';
import { DocumentService } from '../services/document';

const HYDRATABLE_TYPES = new Set<DocumentMeta['type']>([
  'image',
  'video',
  'audio',
  'music',
  'pdf',
  'file',
]);

export function useHydratedViewerDocument(activeDoc: DocumentMeta | null): DocumentMeta | null {
  const [viewDoc, setViewDoc] = useState<DocumentMeta | null>(activeDoc);

  useEffect(() => {
    if (!activeDoc) {
      setViewDoc(null);
      return undefined;
    }

    let cancelled = false;
    setViewDoc(activeDoc);

    const needsHydration =
      isKnowledgebaseApiAvailable()
      && !activeDoc.url
      && HYDRATABLE_TYPES.has(activeDoc.type);

    if (!needsHydration) {
      return () => {
        cancelled = true;
      };
    }

    DocumentService.hydrateDocumentForViewer(activeDoc)
      .then((hydrated) => {
        if (!cancelled) {
          setViewDoc(hydrated);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setViewDoc(activeDoc);
        }
      });

    return () => {
      cancelled = true;
    };
  }, [activeDoc]);

  return viewDoc;
}
