import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Download, Image as ImageIcon, Maximize2, ZoomIn, ZoomOut } from 'lucide-react';
import { resolveMediaPreviewUrl } from '../../utils/searchMediaViewerBridge';
import { MediaSourceChip } from './shared/MediaSourceChip';
import type { SearchMediaViewerContentProps } from './types';

const ZOOM_LEVELS = [1, 1.5, 2] as const;

function computeFitSize(
  containerWidth: number,
  containerHeight: number,
  imageWidth: number,
  imageHeight: number
): { width: number; height: number } {
  if (containerWidth <= 0 || containerHeight <= 0 || imageWidth <= 0 || imageHeight <= 0) {
    return { width: 0, height: 0 };
  }

  const scale = Math.min(containerWidth / imageWidth, containerHeight / imageHeight);
  return {
    width: Math.max(1, Math.round(imageWidth * scale)),
    height: Math.max(1, Math.round(imageHeight * scale))
  };
}

export function SearchImageViewerContent({ item }: SearchMediaViewerContentProps) {
  const previewUrl = resolveMediaPreviewUrl(item);
  const canvasRef = useRef<HTMLDivElement>(null);
  const [zoomIndex, setZoomIndex] = useState(0);
  const [loaded, setLoaded] = useState(false);
  const [naturalSize, setNaturalSize] = useState<{ width: number; height: number } | null>(null);
  const [fitSize, setFitSize] = useState<{ width: number; height: number } | null>(null);

  const zoom = ZOOM_LEVELS[zoomIndex];
  const isZoomed = zoomIndex > 0;

  const layoutSize = useMemo(() => {
    if (item.imageWidth && item.imageHeight) {
      return { width: item.imageWidth, height: item.imageHeight };
    }
    if (naturalSize) return naturalSize;
    return null;
  }, [item.imageHeight, item.imageWidth, naturalSize]);

  const dimensionsLabel = useMemo(() => {
    if (!layoutSize) return null;
    return `${layoutSize.width} × ${layoutSize.height}`;
  }, [layoutSize]);

  const aspectLabel = useMemo(() => {
    if (!layoutSize) return null;
    const ratio = layoutSize.width / layoutSize.height;
    if (ratio > 2.2) return '超宽图';
    if (ratio > 1.35) return '横图';
    if (ratio < 0.55) return '超长竖图';
    if (ratio < 0.82) return '竖图';
    if (ratio > 0.92 && ratio < 1.08) return '方图';
    return '标准比例';
  }, [layoutSize]);

  const recalcFit = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas || !layoutSize) return;
    const next = computeFitSize(canvas.clientWidth, canvas.clientHeight, layoutSize.width, layoutSize.height);
    setFitSize(next.width > 0 && next.height > 0 ? next : null);
  }, [layoutSize]);

  useEffect(() => {
    setLoaded(false);
    setNaturalSize(null);
    setFitSize(null);
    setZoomIndex(0);
  }, [previewUrl, item.id]);

  useEffect(() => {
    recalcFit();
    const canvas = canvasRef.current;
    if (!canvas) return;
    const observer = new ResizeObserver(() => recalcFit());
    observer.observe(canvas);
    return () => observer.disconnect();
  }, [recalcFit]);

  const handleDownload = () => {
    if (!previewUrl) return;
    const link = document.createElement('a');
    link.href = previewUrl;
    link.download = item.title;
    link.target = '_blank';
    link.rel = 'noopener noreferrer';
    link.click();
  };

  const resetZoom = useCallback(() => setZoomIndex(0), []);

  const cycleZoom = useCallback(() => {
    setZoomIndex((value) => (value + 1) % ZOOM_LEVELS.length);
  }, []);

  const handleWheel = useCallback((event: React.WheelEvent<HTMLDivElement>) => {
    event.preventDefault();
    setZoomIndex((value) => {
      if (event.deltaY < 0) return Math.min(ZOOM_LEVELS.length - 1, value + 1);
      return Math.max(0, value - 1);
    });
  }, []);

  if (!previewUrl) {
    return (
      <div className="search-image-viewer search-image-viewer--empty">
        <ImageIcon className="w-12 h-12 opacity-35" />
        <p>暂无预览图</p>
      </div>
    );
  }

  const frameStyle =
    fitSize && layoutSize
      ? ({
          width: `${fitSize.width}px`,
          height: `${fitSize.height}px`,
          aspectRatio: `${layoutSize.width} / ${layoutSize.height}`,
          transform: `scale(${zoom})`
        } as React.CSSProperties)
      : layoutSize
        ? ({ aspectRatio: `${layoutSize.width} / ${layoutSize.height}`, transform: `scale(${zoom})` } as React.CSSProperties)
        : ({ transform: `scale(${zoom})` } as React.CSSProperties);

  return (
    <div className="search-image-viewer">
      <div
        ref={canvasRef}
        className={`search-image-viewer__stage ${isZoomed ? 'search-image-viewer__stage--zoomed' : ''}`}
        onDoubleClick={cycleZoom}
        onWheel={handleWheel}
      >
        {!loaded && <div className="search-image-viewer__loader" aria-hidden />}

        <div className="search-image-viewer__chrome search-image-viewer__chrome--top">
          <MediaSourceChip item={item} variant="on-dark" />
          {aspectLabel && <span className="search-image-viewer__shape">{aspectLabel}</span>}
          {dimensionsLabel && <span className="search-image-viewer__meta">{dimensionsLabel}</span>}
          <div className="search-image-viewer__tools">
            <button
              type="button"
              className="search-image-viewer__tool-btn"
              onClick={() => setZoomIndex((value) => Math.max(0, value - 1))}
              disabled={zoomIndex === 0}
              title="缩小"
            >
              <ZoomOut className="w-4 h-4" />
            </button>
            <span className="search-image-viewer__zoom-label">{Math.round(zoom * 100)}%</span>
            <button
              type="button"
              className="search-image-viewer__tool-btn"
              onClick={() => setZoomIndex((value) => Math.min(ZOOM_LEVELS.length - 1, value + 1))}
              disabled={zoomIndex === ZOOM_LEVELS.length - 1}
              title="放大"
            >
              <ZoomIn className="w-4 h-4" />
            </button>
            <button
              type="button"
              className="search-image-viewer__tool-btn"
              onClick={resetZoom}
              disabled={!isZoomed}
              title="适应窗口"
            >
              <Maximize2 className="w-4 h-4" />
            </button>
            <button type="button" className="search-image-viewer__tool-btn" onClick={handleDownload} title="下载图片">
              <Download className="w-4 h-4" />
            </button>
          </div>
        </div>

        <div className="search-image-viewer__frame" style={frameStyle}>
          <img
            src={previewUrl}
            alt={item.title}
            className={`search-image-viewer__image ${loaded ? 'search-image-viewer__image--loaded' : ''}`}
            onLoad={(event) => {
              const img = event.currentTarget;
              if (!item.imageWidth || !item.imageHeight) {
                setNaturalSize({
                  width: img.naturalWidth,
                  height: img.naturalHeight
                });
              }
              setLoaded(true);
            }}
            draggable={false}
          />
        </div>

        <div className="search-image-viewer__chrome search-image-viewer__chrome--bottom">
          <p className="search-image-viewer__title">{item.title}</p>
          {item.snippet && <p className="search-image-viewer__snippet">{item.snippet}</p>}
        </div>
      </div>
    </div>
  );
}
