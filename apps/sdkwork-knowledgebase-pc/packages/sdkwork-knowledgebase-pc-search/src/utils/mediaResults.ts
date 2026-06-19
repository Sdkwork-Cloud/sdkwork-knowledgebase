import type { SearchMediaCategory, SearchMediaItem, SearchRelatedMedia } from '../types';

export type SearchMediaTab = 'answer' | SearchMediaCategory;

export interface MediaTabDef {
  id: SearchMediaTab;
  label: string;
  count: number;
}

const CATEGORY_KEYS: Record<Exclude<SearchMediaTab, 'answer'>, keyof SearchRelatedMedia> = {
  image: 'images',
  video: 'videos',
  audio: 'audio',
  music: 'music',
  product: 'products'
};

export function getMediaTabDefs(media?: SearchRelatedMedia): MediaTabDef[] {
  if (!media) return [{ id: 'answer', label: '回答', count: 0 }];

  const tabs: MediaTabDef[] = [{ id: 'answer', label: '回答', count: 0 }];

  if (media.images.length > 0) tabs.push({ id: 'image', label: '图片', count: media.images.length });
  if (media.videos.length > 0) tabs.push({ id: 'video', label: '视频', count: media.videos.length });
  if (media.audio.length > 0) tabs.push({ id: 'audio', label: '音频', count: media.audio.length });
  if (media.music?.length) tabs.push({ id: 'music', label: '音乐', count: media.music.length });
  if (media.products.length > 0) tabs.push({ id: 'product', label: '商品', count: media.products.length });

  return tabs;
}

export function getMediaItemsForTab(
  media: SearchRelatedMedia | undefined,
  tab: SearchMediaTab
): SearchMediaItem[] {
  if (!media || tab === 'answer') return [];
  const key = CATEGORY_KEYS[tab];
  return media[key] ?? [];
}

export function countRelatedMedia(media?: SearchRelatedMedia): number {
  if (!media) return 0;
  return (
    media.images.length +
    media.videos.length +
    media.audio.length +
    (media.music?.length ?? 0) +
    media.products.length
  );
}

export function hasRelatedMedia(media?: SearchRelatedMedia): boolean {
  return countRelatedMedia(media) > 0;
}

export function getRelatedMediaTotal(media?: SearchRelatedMedia): number {
  return countRelatedMedia(media);
}
