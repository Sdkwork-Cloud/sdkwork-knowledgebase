import React, { useState, useEffect } from 'react';
import { Sparkles } from 'lucide-react';
import { DocumentMeta, KnowledgeBase } from './services/document';
import { ImageViewer } from './components/players/ImageViewer';
import { VideoPlayer } from './components/players/VideoPlayer';
import { FileDownloader } from './components/players/FileDownloader';
import { MusicPlayer } from './components/players/MusicPlayer';

export interface MediaViewerProps {
  activeDoc: DocumentMeta;
  docContent?: string;
  onContentChange?: (content: string) => void;
  isTranscribing?: boolean;
  onTranscribeStart?: () => void;
  onTranscribeComplete?: (content: string) => void;
  onTitleChange?: (title: string) => void;
  activeKb?: KnowledgeBase | null;
  onUpdateDocs?: () => void;
}

export function MediaViewer({ 
  activeDoc, 
  docContent, 
  onContentChange, 
  isTranscribing,
  onTranscribeStart,
  onTranscribeComplete,
  onTitleChange,
  activeKb,
  onUpdateDocs
}: MediaViewerProps) {
  const [toastMsg, setToastMsg] = useState<string | null>(null);

  // Clear toast after timeout
  useEffect(() => {
    if (toastMsg) {
      const timer = setTimeout(() => setToastMsg(null), 3000);
      return () => clearTimeout(timer);
    }
  }, [toastMsg]);

  return (
    <div id="media-transcription-workspace" className="w-full flex-1 p-0 flex flex-col bg-[var(--color-kb-editor)] min-h-0 select-none">
      
      {/* Toast Alert Widget */}
      {toastMsg && (
        <div id="audio-toast" className="fixed top-24 left-1/2 transform -translate-x-1/2 z-[9999] bg-white dark:bg-zinc-900 border border-zinc-200 dark:border-zinc-700/80 px-4 py-2.5 rounded-xl shadow-2xl flex items-center gap-2 text-xs font-semibold text-rose-500 dark:text-rose-300 animate-bounce">
          <Sparkles size={14} className="text-rose-500 dark:text-rose-400 shrink-0" />
          <span className="text-zinc-800 dark:text-rose-300">{toastMsg}</span>
        </div>
      )}

      {/* Media Types Router */}
      <div className="flex-1 w-full bg-[var(--color-kb-editor)] overflow-hidden flex flex-col min-h-0">
        
        {/* Render Image Source */}
        {activeDoc.type === 'image' && (
          <ImageViewer activeDoc={activeDoc} activeKb={activeKb} onUpdateDocs={onUpdateDocs} onToastMessage={setToastMsg} />
        )}

        {/* Render Video Player */}
        {activeDoc.type === 'video' && (
          <VideoPlayer activeDoc={activeDoc} activeKb={activeKb} onUpdateDocs={onUpdateDocs} onToastMessage={setToastMsg} />
        )}

        {/* Render Standard Generic File */}
        {activeDoc.type === 'file' && (
          <FileDownloader activeDoc={activeDoc} />
        )}

        {/* Generic Audio/Music Player Console */}
        {(activeDoc.type === 'audio' || activeDoc.type === 'music') && (
          <MusicPlayer 
            activeDoc={activeDoc} 
            onToastMessage={setToastMsg}
            isTranscribing={isTranscribing}
            onTranscribeStart={onTranscribeStart}
            onTranscribeComplete={onTranscribeComplete}
          />
        )}

      </div>
    </div>
  );
}
