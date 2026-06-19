import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  Download,
  Image as ImageIcon,
  Info,
  Maximize2,
  RotateCcw,
  RotateCw,
  ZoomIn,
  ZoomOut
} from 'lucide-react';
import { resolveMediaPreviewUrl } from '../../utils/searchMediaViewerBridge';
import { MediaSourceChip } from './shared/MediaSourceChip';
import type { SearchMediaViewerContentProps } from './types';

const ZOOM_LEVELS = [1, 1.25, 1.5, 2, 2.5, 3, 4] as const;

interface PanOffset {
  x: number;
  y: number;
}

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
  const [rotation, setRotation] = useState(0);
  const [loaded, setLoaded] = useState(false);
  const [naturalSize, setNaturalSize] = useState<{ width: number; height: number } | null>(null);
  const [fitSize, setFitSize] = useState<{ width: number; height: number } | null>(null);
  const [panOffset, setPanOffset] = useState<PanOffset>({ x: 0, y: 0 });
  const [isPanning, setIsPanning] = useState(false);
  const [showInfo, setShowInfo] = useState(false);
  const panStartRef = useRef<{ x: number; y: number; offsetX: number; offsetY: number } | null>(null);

  const zoom = ZOOM_LEVELS[zoomIndex];
  const isZoomed = zoomIndex > 0;
  const isRotated = rotation !== 0;

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

  const megapixelsLabel = useMemo(() => {
    if (!layoutSize) return null;
    const mp = (layoutSize.width * layoutSize.height) / 1_000_000;
    if (mp < 0.1) return null;
    return mp >= 1 ? `${mp.toFixed(1)} MP` : `${Math.round(mp * 1000)} KP`;
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
    setRotation(0);
    setPanOffset({ x: 0, y: 0 });
  }, [previewUrl, item.id]);

  useEffect(() => {
    recalcFit();
    const canvas = canvasRef.current;
    if (!canvas) return;
    const observer = new ResizeObserver(() => recalcFit());
    observer.observe(canvas);
    return () => observer.disconnect();
  }, [recalcFit]);

  useEffect(() => {
    if (!isZoomed) {
      setPanOffset({ x: 0, y: 0 });
    }
  }, [isZoomed]);

  useEffect(() => {
    if (rotation !== 0 && !isZoomed) {
      setPanOffset({ x: 0, y: 0 });
    }
  }, [rotation, isZoomed]);

  const handleDownload = () => {
    if (!previewUrl) return;
    const link = document.createElement('a');
    link.href = previewUrl;
    link.download = item.title;
    link.target = '_blank';
    link.rel = 'noopener noreferrer';
    link.click();
  };

  const cycleZoom = useCallback(() => {
    setZoomIndex((value) => (value + 1) % ZOOM_LEVELS.length);
  }, []);

  const rotateLeft = useCallback(() => {
    setRotation((value) => (value - 90) % 360);
    setPanOffset({ x: 0, y: 0 });
  }, []);

  const rotateRight = useCallback(() => {
    setRotation((value) => (value + 90) % 360);
    setPanOffset({ x: 0, y: 0 });
  }, []);

  const resetTransforms = useCallback(() => {
    setZoomIndex(0);
    setRotation(0);
    setPanOffset({ x: 0, y: 0 });
  }, []);

  const handleWheel = useCallback((event: React.WheelEvent<HTMLDivElement>) => {
    event.preventDefault();
    setZoomIndex((value) => {
      if (event.deltaY < 0) return Math.min(ZOOM_LEVELS.length - 1, value + 1);
      return Math.max(0, value - 1);
    });
  }, []);

  const handlePanStart = useCallback(
    (event: React.MouseEvent<HTMLDivElement>) => {
      if (!isZoomed && !isRotated) return;
      event.preventDefault();
      setIsPanning(true);
      panStartRef.current = {
        x: event.clientX,
        y: event.clientY,
        offsetX: panOffset.x,
        offsetY: panOffset.y
      };
    },
    [isZoomed, isRotated, panOffset.x, panOffset.y]
  );

  const handlePanMove = useCallback(
    (event: React.MouseEvent<HTMLDivElement>) => {
      if (!isPanning || !panStartRef.current) return;
      const deltaX = event.clientX - panStartRef.current.x;
      const deltaY = event.clientY - panStartRef.current.y;
      setPanOffset({
        x: panStartRef.current.offsetX + deltaX,
        y: panStartRef.current.offsetY + deltaY
      });
    },
    [isPanning]
  );

  const handlePanEnd = useCallback(() => {
    setIsPanning(false);
    panStartRef.current = null;
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
          transform: `translate(${panOffset.x}px, ${panOffset.y}px) scale(${zoom}) rotate(${rotation}deg)`
        } as React.CSSProperties)
      : layoutSize
        ? ({
            aspectRatio: `${layoutSize.width} / ${layoutSize.height}`,
            transform: `translate(${panOffset.x}px, ${panOffset.y}px) scale(${zoom}) rotate(${rotation}deg)`
          } as React.CSSProperties)
        : ({
            transform: `translate(${panOffset.x}px, ${panOffset.y}px) scale(${zoom}) rotate(${rotation}deg)`
          } as React.CSSProperties);

  const stageCursor = isPanning ? 'grabbing' : isZoomed || isRotated ? 'grab' : 'default';

  return (
    <div className="search-image-viewer">
      <div
        ref={canvasRef}
        className={`search-image-viewer__stage ${isZoomed ? 'search-image-viewer__stage--zoomed' : ''} ${isPanning ? 'search-image-viewer__stage--panning' : ''}`}
        style={{ cursor: stageCursor }}
        onDoubleClick={cycleZoom}
        onWheel={handleWheel}
        onMouseDown={handlePanStart}
        onMouseMove={handlePanMove}
        onMouseUp={handlePanEnd}
        onMouseLeave={handlePanEnd}
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
              onClick={rotateLeft}
              title="向左旋转"
            >
              <RotateCcw className="w-4 h-4" />
            </button>
            <button
              type="button"
              className="search-image-viewer__tool-btn"
              onClick={rotateRight}
              title="向右旋转"
            >
              <RotateCw className="w-4 h-4" />
            </button>
            <button
              type="button"
              className="search-image-viewer__tool-btn"
              onClick={resetTransforms}
              disabled={!isZoomed && !isRotated}
              title="重置视图"
            >
              <Maximize2 className="w-4 h-4" />
            </button>
            <button
              type="button"
              className={`search-image-viewer__tool-btn ${showInfo ? 'search-image-viewer__tool-btn--active' : ''}`}
              onClick={() => setShowInfo((value) => !value)}
              title="图片信息"
              aria-pressed={showInfo}
            >
              <Info className="w-4 h-4" />
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

        {showInfo && (
          <div className="search-image-viewer__info-panel">
            <div className="search-image-viewer__info-row">
              <span className="search-image-viewer__info-label">标题</span>
              <span className="search-image-viewer__info-value" title={item.title}>{item.title}</span>
            </div>
            {dimensionsLabel && (
              <div className="search-image-viewer__info-row">
                <span className="search-image-viewer__info-label">尺寸</span>
                <span className="search-image-viewer__info-value">{dimensionsLabel} px</span>
              </div>
            )}
            {megapixelsLabel && (
              <div className="search-image-viewer__info-row">
                <span className="search-image-viewer__info-label">像素</span>
                <span className="search-image-viewer__info-value">{megapixelsLabel}</span>
              </div>
            )}
            {aspectLabel && (
              <div className="search-image-viewer__info-row">
                <span className="search-image-viewer__info-label">比例</span>
                <span className="search-image-viewer__info-value">{aspectLabel}</span>
              </div>
            )}
            {item.source === 'kb' && item.kbId && (
              <div className="search-image-viewer__info-row">
                <span className="search-image-viewer__info-label">来源</span>
                <span className="search-image-viewer__info-value">知识库文档</span>
              </div>
            )}
            {item.snippet && (
              <div className="search-image-viewer__info-row search-image-viewer__info-row--snippet">
                <span className="search-image-viewer__info-label">描述</span>
                <span className="search-image-viewer__info-value">{item.snippet}</span>
              </div>
            )}
          </div>
        )}

        <div className="search-image-viewer__chrome search-image-viewer__chrome--bottom">
          <p className="search-image-viewer__title">{item.title}</p>
          {item.snippet && !showInfo && <p className="search-image-viewer__snippet">{item.snippet}</p>}
          {(isZoomed || isRotated) && (
            <p className="search-image-viewer__hint">
              {isZoomed && `缩放 ${Math.round(zoom * 100)}% · 拖动平移 · 滚轮调整`}
              {isZoomed && isRotated && ' · '}
              {isRotated && `旋转 ${rotation}°`}
            </p>
          )}
        </div>
      </div>
    </div>
  );
}
