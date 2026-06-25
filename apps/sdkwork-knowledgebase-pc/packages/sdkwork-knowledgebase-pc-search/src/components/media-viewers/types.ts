import type { SearchMediaCategory, SearchMediaItem } from '../../types';
import type { MediaPlayMode } from '../../utils/mediaPlaylist';

export type MediaViewerLayoutMode = 'expanded' | 'minimized';

export interface MediaPlaylistControls {
  items: SearchMediaItem[];
  activeIndex: number;
  playMode: MediaPlayMode;
  shuffleEnabled: boolean;
  canGoPrevious: boolean;
  canGoNext: boolean;
  isPlaylistOpen: boolean;
  autoPlay?: boolean;
  onSelectTrack: (index: number) => void;
  onPreviousTrack: () => void;
  onNextTrack: () => void;
  onPlayModeChange: (mode: MediaPlayMode) => void;
  onShuffleToggle: () => void;
  onPlaylistToggle: () => void;
  onTrackEnded: () => void;
  onAutoPlayConsumed?: () => void;
  onPlaybackActiveChange?: (isPlaying: boolean) => void;
}

export interface SearchMediaViewerContentProps {
  item: SearchMediaItem;
  category?: SearchMediaCategory;
  layoutMode?: MediaViewerLayoutMode;
  onOpenWebLink?: (url: string, title?: string) => void;
  playlist?: MediaPlaylistControls;
}

export function supportsMediaViewerMinimize(category: SearchMediaCategory): boolean {
  return category === 'video' || category === 'audio' || category === 'music';
}
