export interface SearchSource {
  id: string;
  title: string;
  type: 'web' | 'doc' | 'kb';
  url?: string;
  snippet: string;
  kbId?: string;
  /** Document id when type is doc */
  docId?: string;
  kbTitle?: string;
  docType?: 'richtext' | 'code' | 'markdown' | 'file' | 'image' | 'audio' | 'video' | 'folder' | 'pdf' | 'music';
  parentId?: string | null;
  author?: string;
  updatedAt?: string;
}

export interface SearchNavigateToFilePayload {
  kbId: string;
  docId: string;
  title: string;
  type: NonNullable<SearchSource['docType']>;
  kbTitle?: string;
  author?: string;
  updatedAt?: string;
  parentId?: string | null;
}

export interface SearchNavigateToKbPayload {
  kbId: string;
  kbTitle?: string;
}

export type SearchMediaCategory = 'image' | 'video' | 'audio' | 'music' | 'product';

/** Timed line for synced lyrics (music) or transcript/subtitles (audio recordings). */
export interface MediaTimedLine {
  startTime: number;
  endTime?: number;
  /** Speaker name for meeting minutes / multi-speaker transcripts. */
  speaker?: string;
  text: string;
}

export type SearchMediaAudioKind = 'podcast' | 'recording' | 'speech';

export interface SearchMediaItem {
  id: string;
  category: SearchMediaCategory;
  title: string;
  thumbnailUrl?: string;
  previewUrl?: string;
  source: 'kb' | 'web';
  url?: string;
  kbId?: string;
  docId?: string;
  docType?: SearchSource['docType'];
  snippet?: string;
  artist?: string;
  /** Synced lyrics lines (music). */
  lyrics?: MediaTimedLine[];
  /** Synced transcript / subtitles (audio, especially recordings). */
  transcript?: MediaTimedLine[];
  /** Audio subtype for UI labelling. */
  audioKind?: SearchMediaAudioKind;
  price?: string;
  originalPrice?: string;
  merchant?: string;
  rating?: number;
  reviewCount?: number;
  duration?: string;
  description?: string;
  tags?: string[];
  highlights?: string[];
  galleryUrls?: string[];
  shippingNote?: string;
  imageWidth?: number;
  imageHeight?: number;
  /** Declared video frame size for adaptive player layout. */
  videoWidth?: number;
  videoHeight?: number;
  /** Declared cover art size for audio/music list and player. */
  coverWidth?: number;
  coverHeight?: number;
  specs?: Array<{ label: string; value: string }>;
}

export interface SearchRelatedMedia {
  images: SearchMediaItem[];
  videos: SearchMediaItem[];
  audio: SearchMediaItem[];
  music: SearchMediaItem[];
  products: SearchMediaItem[];
}

export type SearchMediaTab = 'answer' | SearchMediaCategory;

export interface SearchMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  timestamp: string;
  sources?: SearchSource[];
  relatedMedia?: SearchRelatedMedia;
  isSearching?: boolean;
  searchSteps?: {
    id: string;
    label: string;
    status: 'idle' | 'running' | 'success' | 'failed';
  }[];
}

export interface SearchSession {
  id: string;
  title: string;
  createdAt: string;
  messages: SearchMessage[];
  webSearchEnabled?: boolean;
  deepThinkEnabled?: boolean;
}

export interface SearchModuleProps {
  onGoToKb: (payload: SearchNavigateToKbPayload) => void;
  onGoToFile: (payload: SearchNavigateToFilePayload) => void;
  onOpenWebLink?: (url: string, title?: string) => void;
}

export type SearchComposerVariant = 'hero' | 'chat';
