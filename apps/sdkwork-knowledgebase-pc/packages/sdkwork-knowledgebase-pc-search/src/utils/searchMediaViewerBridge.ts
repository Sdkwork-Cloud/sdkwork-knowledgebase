import type { SearchMediaCategory, SearchMediaItem } from '../types';

export const SEARCH_OPEN_MEDIA_VIEWER_EVENT = 'sdkwork-search:open-media-viewer';

export interface SearchMediaViewerOpenDetail {
  items: SearchMediaItem[];
  activeIndex: number;
  category: SearchMediaCategory;
}

export function dispatchOpenSearchMediaViewer(detail: SearchMediaViewerOpenDetail) {
  window.dispatchEvent(new CustomEvent(SEARCH_OPEN_MEDIA_VIEWER_EVENT, { detail }));
}

export function resolveMediaPreviewUrl(item: SearchMediaItem): string | undefined {
  return item.previewUrl ?? item.url ?? item.thumbnailUrl;
}

export function resolvePlaybackUrl(item: SearchMediaItem): string | undefined {
  if (item.previewUrl) return item.previewUrl;
  if (item.url && isDirectMediaUrl(item.url, item.category === 'video' ? 'video' : 'audio')) {
    return item.url;
  }
  return undefined;
}

export function getVideoEmbedUrl(rawUrl: string): string | null {
  try {
    const url = new URL(rawUrl);
    const host = url.hostname.replace(/^www\./, '');

    if (host === 'youtu.be') {
      const id = url.pathname.replace(/^\//, '').split('/')[0];
      return id ? `https://www.youtube.com/embed/${id}?rel=0` : null;
    }

    if (host === 'youtube.com' || host === 'm.youtube.com') {
      const id = url.searchParams.get('v');
      return id ? `https://www.youtube.com/embed/${id}?rel=0` : null;
    }

    if (host === 'bilibili.com' || host === 'm.bilibili.com') {
      const match = url.pathname.match(/\/video\/(BV[\w]+)/i);
      if (match?.[1]) {
        return `https://player.bilibili.com/player.html?bvid=${match[1]}&high_quality=1&autoplay=0`;
      }
    }
  } catch {
    return null;
  }

  return null;
}

export function isDirectMediaUrl(
  rawUrl: string,
  kind: 'video' | 'audio'
): boolean {
  try {
    const pathname = new URL(rawUrl).pathname.toLowerCase();
    const extensions =
      kind === 'video'
        ? ['.mp4', '.webm', '.ogg', '.mov', '.m4v']
        : ['.mp3', '.wav', '.ogg', '.m4a', '.aac', '.flac'];
    return extensions.some((ext) => pathname.endsWith(ext));
  } catch {
    return false;
  }
}

export type VideoPlaybackMode = 'embed' | 'direct' | 'none';

export function resolveVideoPlayback(item: SearchMediaItem): {
  mode: VideoPlaybackMode;
  url?: string;
  poster?: string;
} {
  const candidates = [item.previewUrl, item.url].filter(Boolean) as string[];

  for (const candidate of candidates) {
    const embedUrl = getVideoEmbedUrl(candidate);
    if (embedUrl) {
      return { mode: 'embed', url: embedUrl, poster: item.thumbnailUrl };
    }
    if (isDirectMediaUrl(candidate, 'video')) {
      return { mode: 'direct', url: candidate, poster: item.thumbnailUrl };
    }
  }

  return { mode: 'none', poster: item.thumbnailUrl };
}

export function resolveAudioPlayback(item: SearchMediaItem): string | undefined {
  const playbackUrl = resolvePlaybackUrl(item);
  if (playbackUrl && isDirectMediaUrl(playbackUrl, 'audio')) {
    return playbackUrl;
  }
  return playbackUrl && !getVideoEmbedUrl(playbackUrl) ? playbackUrl : undefined;
}
