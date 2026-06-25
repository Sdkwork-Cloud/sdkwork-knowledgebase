import type { CSSProperties } from 'react';
import type { SearchMediaItem } from '../types';

export type MediaOrientation = 'landscape' | 'portrait' | 'square' | 'ultrawide' | 'ultratall';

export interface MediaDimensions {
  width: number;
  height: number;
}

export function getMediaOrientation(width: number, height: number): MediaOrientation {
  if (width <= 0 || height <= 0) return 'landscape';
  const ratio = width / height;
  if (ratio > 2.2) return 'ultrawide';
  if (ratio > 1.15) return 'landscape';
  if (ratio < 0.55) return 'ultratall';
  if (ratio < 0.87) return 'portrait';
  return 'square';
}

export function getMediaShapeLabel(width: number, height: number): string {
  const orientation = getMediaOrientation(width, height);
  switch (orientation) {
    case 'ultrawide':
      return '超宽视频';
    case 'ultratall':
      return '竖屏全屏';
    case 'portrait':
      return '竖屏';
    case 'square':
      return '方屏';
    default:
      return '横屏';
  }
}

export function getCoverShapeLabel(width: number, height: number): string {
  const orientation = getMediaOrientation(width, height);
  switch (orientation) {
    case 'ultrawide':
      return '宽幅封面';
    case 'ultratall':
      return '长竖封面';
    case 'portrait':
      return '竖版封面';
    case 'square':
      return '方版封面';
    default:
      return '横版封面';
  }
}

export function resolveVideoDimensions(item: SearchMediaItem): MediaDimensions {
  if (item.videoWidth && item.videoHeight) {
    return { width: item.videoWidth, height: item.videoHeight };
  }
  return { width: 1920, height: 1080 };
}

export function resolveCoverDimensions(item: SearchMediaItem): MediaDimensions {
  if (item.coverWidth && item.coverHeight) {
    return { width: item.coverWidth, height: item.coverHeight };
  }
  return { width: 320, height: 320 };
}

export function resolveImageDimensions(item: SearchMediaItem): MediaDimensions | null {
  if (item.imageWidth && item.imageHeight) {
    return { width: item.imageWidth, height: item.imageHeight };
  }
  return null;
}

export function getImageShapeLabel(width: number, height: number): string {
  const orientation = getMediaOrientation(width, height);
  switch (orientation) {
    case 'ultrawide':
      return '超宽图';
    case 'ultratall':
      return '超长竖图';
    case 'portrait':
      return '竖图';
    case 'square':
      return '方图';
    default:
      return '横图';
  }
}

export function buildContainAspectStyle(width: number, height: number): CSSProperties {
  return {
    aspectRatio: `${width} / ${height}`,
    width: `min(100cqw, calc(100cqh * ${width} / ${height}))`,
    height: `min(100cqh, calc(100cqw * ${height} / ${width}))`
  };
}

export function buildAspectRatioStyle(width: number, height: number): CSSProperties {
  return { aspectRatio: `${width} / ${height}` };
}

export function orientationClass(prefix: string, orientation: MediaOrientation): string {
  return `${prefix}--${orientation}`;
}
