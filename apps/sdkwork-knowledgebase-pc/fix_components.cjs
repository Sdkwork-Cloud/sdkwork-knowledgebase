const fs = require('fs');
const file = 'packages/sdkwork-knowledgebase-pc-knowledgebase/src/components.tsx';
let data = fs.readFileSync(file, 'utf8');

// replace local storage of openDocs and activeDoc with normal useState, and add TabCacheService integration.

data = data.replace(
  `import { DocumentService, FolderNode, DocumentMeta, KnowledgeBase } from './services/document';`,
  `import { DocumentService, FolderNode, DocumentMeta, KnowledgeBase } from './services/document';\nimport { TabCacheService } from './services/tabService';`
);

data = data.replace(
  `  const [activeDoc, setActiveDoc] = useLocalStorage<DocumentMeta | null>('app-active-doc', null);\n  const [openDocs, setOpenDocs] = useLocalStorage<DocumentMeta[]>('app-open-docs', []);`,
  `  const [activeDoc, setActiveDoc] = useState<DocumentMeta | null>(null);\n  const [openDocs, setOpenDocs] = useState<DocumentMeta[]>([]);`
);

const handleSelectDocOld = `
  const handleSelectDoc = useCallback(async (doc: DocumentMeta) => {
    if (!doc) return;
    setActiveDoc(doc);
    setSelectedDocIds(new Set()); // Clear multi-selection when clicking a single doc
    
    // Add to open tabs if not a folder and not already present
    if (doc.type !== 'folder') {
      setOpenDocs(prev => {
        if (prev.some(d => d.id === doc.id)) {
          return prev;
        }
        return [...prev, doc];
      });
    }

    if (doc.type === 'richtext' || doc.type === 'code' || doc.type === 'markdown') {
      setDocContent('Loading...');
      const content = await DocumentService.getDocumentContent(doc.id);
      setDocContent(content);
    } else {
      setDocContent('');
    }
  }, []);
`;

const handleSelectDocNew = `
  const handleSelectDoc = useCallback(async (doc: DocumentMeta, currentKbId?: string) => {
    if (!doc) return;
    setActiveDoc(doc);
    setSelectedDocIds(new Set()); // Clear multi-selection when clicking a single doc
    
    const kbId = currentKbId || activeKb?.id || doc.kbId;
    if (kbId) {
      TabCacheService.openDoc(kbId, doc);
      setOpenDocs(TabCacheService.getOpenDocs(kbId));
    } else {
      // Fallback
      if (doc.type !== 'folder') {
        setOpenDocs(prev => {
          if (prev.some(d => d.id === doc.id)) return prev;
          return [...prev, doc];
        });
      }
    }

    if (doc.type === 'richtext' || doc.type === 'code' || doc.type === 'markdown') {
      setDocContent('Loading...');
      const content = await DocumentService.getDocumentContent(doc.id);
      setDocContent(content);
    } else {
      setDocContent('');
    }
  }, [activeKb]);
`;
data = data.replace(handleSelectDocOld.trim(), handleSelectDocNew.trim());

const handleSelectKbOld = `
  const handleSelectKb = useCallback((kb: KnowledgeBase, preserveState = false) => {
    setActiveKb(kb);
    if (!preserveState) {
      setActiveDoc(null);
      setOpenDocs([]); // Clear open tabs when changing knowledge bases
      setActiveTab('kb'); // Force switch back to KB docs view upon selecting a KB
    }
    setLoadingDocs(true);
    DocumentService.getDocuments(kb.id).then(data => {
      setDocs(data);
      setLoadingDocs(false);
      if (preserveState && activeDoc) {
        // Just reload the active content if we're preserving
        const existingDocInList = activeDoc; // activeDoc from closure
        handleSelectDoc(existingDocInList);
      } else if (data.length > 0) {
        const firstDoc = data.find(item => item.type !== 'folder') as DocumentMeta;
        if (firstDoc) {
          handleSelectDoc(firstDoc);
        } else if (data[0].type === 'folder' && (data[0] as FolderNode).children.length > 0) {
          handleSelectDoc((data[0] as FolderNode).children[0] as DocumentMeta);
        }
      }
    });
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [handleSelectDoc, setActiveKb, setActiveDoc, setOpenDocs]);
`;

const handleSelectKbNew = `
  const handleSelectKb = useCallback((kb: KnowledgeBase, preserveState = false) => {
    setActiveKb(kb);
    setActiveTab('kb'); // Force switch back to KB docs view upon selecting a KB
    TabCacheService.initKb(kb.id);
    
    const cachedDocs = TabCacheService.getOpenDocs(kb.id);
    const cachedActiveId = TabCacheService.getActiveDocId(kb.id);
    setOpenDocs(cachedDocs);
    setActiveDoc(null);
    setDocContent('');
    
    setLoadingDocs(true);
    DocumentService.getDocuments(kb.id).then(data => {
      setDocs(data);
      setLoadingDocs(false);
      
      const flatDocs: DocumentMeta[] = [];
      const flatten = (nodes: any[]) => {
        nodes.forEach(n => {
           if (n.type === 'folder') {
              if (n.children) flatten(n.children);
           } else {
              flatDocs.push(n);
           }
        });
      }
      flatten(data);

      if (cachedActiveId) {
        const docToOpen = flatDocs.find(d => d.id === cachedActiveId);
        if (docToOpen) handleSelectDoc(docToOpen, kb.id);
      } else if (cachedDocs.length > 0) {
        const docToOpen = flatDocs.find(d => d.id === cachedDocs[0].id) || cachedDocs[0];
        handleSelectDoc(docToOpen, kb.id);
      } else if (!preserveState && data.length > 0) {
        const firstDoc = data.find(item => item.type !== 'folder') as DocumentMeta;
        if (firstDoc) {
          handleSelectDoc(firstDoc, kb.id);
        } else if (data[0].type === 'folder' && (data[0] as FolderNode).children?.length > 0) {
          handleSelectDoc((data[0] as FolderNode).children[0] as DocumentMeta, kb.id);
        }
      }
    });
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [handleSelectDoc, setActiveKb, setActiveDoc, setOpenDocs]);
`;
data = data.replace(handleSelectKbOld.trim(), handleSelectKbNew.trim());

const handleCloseDocOld = `
  const handleCloseDoc = useCallback((docId: string) => {
    setOpenDocs(prev => {
      const index = prev.findIndex(d => d.id === docId);
      if (index === -1) return prev;
      const nextList = prev.filter(d => d.id !== docId);
      
      // If closing the active note, select an adjacent one
      if (activeDoc && activeDoc.id === docId) {
        if (nextList.length > 0) {
          const newIndex = Math.min(index, nextList.length - 1);
          const newActive = nextList[newIndex];
          handleSelectDoc(newActive);
        } else {
          setActiveDoc(null);
          setDocContent('');
        }
      }
      return nextList;
    });
  }, [activeDoc, handleSelectDoc]);
`;

const handleCloseDocNew = `
  const handleCloseDoc = useCallback((docId: string) => {
    if (!activeKb) return;
    const { remainingDocs, nextActiveId } = TabCacheService.closeDoc(activeKb.id, docId);
    setOpenDocs(remainingDocs);
    if (!nextActiveId) {
      setActiveDoc(null);
      setDocContent('');
    } else if (activeDoc?.id !== nextActiveId) {
      const nextDoc = remainingDocs.find(d => d.id === nextActiveId);
      if (nextDoc) handleSelectDoc(nextDoc, activeKb.id);
    }
  }, [activeKb, activeDoc, handleSelectDoc]);
`;
data = data.replace(handleCloseDocOld.trim(), handleCloseDocNew.trim());

const handleCloseOthersOld = `
  const handleCloseOthers = useCallback((docId: string) => {
    setOpenDocs(prev => {
      const target = prev.find(d => d.id === docId);
      if (!target) return prev;
      if (!activeDoc || activeDoc.id !== docId) {
        handleSelectDoc(target);
      }
      return [target];
    });
  }, [activeDoc, handleSelectDoc]);
`;

const handleCloseOthersNew = `
  const handleCloseOthers = useCallback((docId: string) => {
    if (!activeKb) return;
    const { remainingDocs } = TabCacheService.closeOthers(activeKb.id, docId);
    setOpenDocs(remainingDocs);
    if (!activeDoc || activeDoc.id !== docId) {
      const target = remainingDocs.find(d => d.id === docId);
      if (target) handleSelectDoc(target, activeKb.id);
    }
  }, [activeKb, activeDoc, handleSelectDoc]);
`;
data = data.replace(handleCloseOthersOld.trim(), handleCloseOthersNew.trim());

const handleCloseToRightOld = `
  const handleCloseToRight = useCallback((docId: string) => {
    setOpenDocs(prev => {
      const index = prev.findIndex(d => d.id === docId);
      if (index === -1) return prev;
      const nextList = prev.slice(0, index + 1);
      
      // If active doc was closed, select the target docId
      if (activeDoc) {
        const stillOpen = nextList.some(d => d.id === activeDoc.id);
        if (!stillOpen) {
          const target = prev.find(d => d.id === docId);
          if (target) handleSelectDoc(target);
        }
      }
      return nextList;
    });
  }, [activeDoc, handleSelectDoc]);
`;

const handleCloseToRightNew = `
  const handleCloseToRight = useCallback((docId: string) => {
    if (!activeKb) return;
    const { remainingDocs, nextActiveId } = TabCacheService.closeToRight(activeKb.id, docId);
    setOpenDocs(remainingDocs);
    if (!nextActiveId) {
       setActiveDoc(null);
       setDocContent('');
    } else if (activeDoc?.id !== nextActiveId) {
       const target = remainingDocs.find(d => d.id === nextActiveId);
       if (target) handleSelectDoc(target, activeKb.id);
    }
  }, [activeKb, activeDoc, handleSelectDoc]);
`;
data = data.replace(handleCloseToRightOld.trim(), handleCloseToRightNew.trim());

const handleCloseAllOld = `
  const handleCloseAll = useCallback(() => {
    setOpenDocs([]);
    setActiveDoc(null);
    setDocContent('');
  }, []);
`;

const handleCloseAllNew = `
  const handleCloseAll = useCallback(() => {
    if (activeKb) {
      TabCacheService.closeAll(activeKb.id);
    }
    setOpenDocs([]);
    setActiveDoc(null);
    setDocContent('');
  }, [activeKb]);
`;
data = data.replace(handleCloseAllOld.trim(), handleCloseAllNew.trim());

fs.writeFileSync(file, data, 'utf8');
