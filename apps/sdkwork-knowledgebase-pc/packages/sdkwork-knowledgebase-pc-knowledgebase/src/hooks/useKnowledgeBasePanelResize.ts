import { useEffect } from 'react';

interface PanelResizeState {
  isDraggingAi: boolean;
  isDraggingKbs: boolean;
  isDraggingDocs: boolean;
  kbsWidth: number;
  setAiWidth: (width: number) => void;
  setKbsWidth: (width: number) => void;
  setDocsWidth: (width: number) => void;
  setIsDraggingAi: (dragging: boolean) => void;
  setIsDraggingKbs: (dragging: boolean) => void;
  setIsDraggingDocs: (dragging: boolean) => void;
}

export function useKnowledgeBasePanelResize({
  isDraggingAi,
  isDraggingKbs,
  isDraggingDocs,
  kbsWidth,
  setAiWidth,
  setKbsWidth,
  setDocsWidth,
  setIsDraggingAi,
  setIsDraggingKbs,
  setIsDraggingDocs,
}: PanelResizeState): void {
  useEffect(() => {
    const handleMouseMove = (e: MouseEvent) => {
      if (isDraggingAi) {
        const newWidth = document.body.clientWidth - e.clientX;
        if (newWidth > 200 && newWidth < 800) {
          setAiWidth(newWidth);
        }
      } else if (isDraggingKbs) {
        const kWidth = e.clientX - 64;
        if (kWidth > 150 && kWidth < 500) {
          setKbsWidth(kWidth);
        }
      } else if (isDraggingDocs) {
        const dWidth = e.clientX - 64 - kbsWidth;
        if (dWidth > 200 && dWidth < 600) {
          setDocsWidth(dWidth);
        }
      }
    };
    const handleMouseUp = () => {
      setIsDraggingAi(false);
      setIsDraggingKbs(false);
      setIsDraggingDocs(false);
    };

    if (isDraggingAi || isDraggingKbs || isDraggingDocs) {
      document.addEventListener('mousemove', handleMouseMove);
      document.addEventListener('mouseup', handleMouseUp);
      document.body.style.cursor = 'col-resize';
      document.body.style.userSelect = 'none';
    } else {
      document.body.style.cursor = '';
      document.body.style.userSelect = '';
    }
    return () => {
      document.removeEventListener('mousemove', handleMouseMove);
      document.removeEventListener('mouseup', handleMouseUp);
    };
  }, [
    isDraggingAi,
    isDraggingKbs,
    isDraggingDocs,
    kbsWidth,
    setAiWidth,
    setKbsWidth,
    setDocsWidth,
    setIsDraggingAi,
    setIsDraggingKbs,
    setIsDraggingDocs,
  ]);
}
