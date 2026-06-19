import React, { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import {
  buildMasonryLayout,
  estimateMasonryThumbHeight,
  MASONRY_GAP_PX,
  MASONRY_MIN_COLUMN_WIDTH,
  type MasonryLayout,
  type MasonryMediaVariant
} from '../utils/searchMediaMasonryLayout';

export interface SearchMediaMasonryItem {
  id: string;
  mediaWidth?: number;
  mediaHeight?: number;
  bodyHeight?: number;
}

export interface SearchMediaMasonryProps<T extends SearchMediaMasonryItem> {
  variant: MasonryMediaVariant;
  items: T[];
  renderItem: (item: T, style: React.CSSProperties) => React.ReactNode;
}

function defaultBodyHeight(variant: MasonryMediaVariant): number {
  switch (variant) {
    case 'video':
      return 92;
    case 'product':
      return 104;
    default:
      return 68;
  }
}

function resolveMediaSize(item: SearchMediaMasonryItem, variant: MasonryMediaVariant) {
  if (item.mediaWidth && item.mediaHeight) {
    return { width: item.mediaWidth, height: item.mediaHeight };
  }

  if (variant === 'product') {
    return { width: 1, height: 1 };
  }

  if (variant === 'video') {
    return { width: 16, height: 9 };
  }

  return { width: 4, height: 3 };
}

function buildEstimatedHeights<T extends SearchMediaMasonryItem>(
  items: T[],
  columnWidth: number,
  variant: MasonryMediaVariant
): number[] {
  return items.map((item) => {
    const media = resolveMediaSize(item, variant);
    const thumbHeight = estimateMasonryThumbHeight(columnWidth, media.width, media.height);
    const bodyHeight = item.bodyHeight ?? defaultBodyHeight(variant);
    return thumbHeight + bodyHeight;
  });
}

export function SearchMediaMasonry<T extends SearchMediaMasonryItem>({
  variant,
  items,
  renderItem
}: SearchMediaMasonryProps<T>) {
  const containerRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<Map<string, HTMLDivElement>>(new Map());
  const [layout, setLayout] = useState<MasonryLayout>({
    columnCount: 1,
    columnWidth: 0,
    containerHeight: 0,
    placements: []
  });
  const [measuredHeights, setMeasuredHeights] = useState<number[] | null>(null);

  const minColumnWidth = MASONRY_MIN_COLUMN_WIDTH[variant];

  const recalcLayout = useCallback(
    (width: number, heights: number[]) => {
      setLayout(buildMasonryLayout(width, heights, minColumnWidth, MASONRY_GAP_PX));
    },
    [minColumnWidth]
  );

  const estimatedHeights = useMemo(() => {
    const width = layout.columnWidth || containerRef.current?.clientWidth || 0;
    if (width <= 0) {
      return items.map(() => defaultBodyHeight(variant) + 120);
    }
    return buildEstimatedHeights(items, width, variant);
  }, [items, layout.columnWidth, variant]);

  const syncMeasuredHeights = useCallback(() => {
    const next = items.map((item) => {
      const node = itemRefs.current.get(item.id);
      return node?.offsetHeight ?? 0;
    });

    if (next.some((height) => height > 0)) {
      setMeasuredHeights(next);
    }
  }, [items]);

  useEffect(() => {
    setMeasuredHeights(null);
  }, [items]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const update = () => {
      const width = container.clientWidth;
      if (width <= 0) return;

      const heights = items.map((_, index) => {
        const measured = measuredHeights?.[index] ?? 0;
        return measured > 0 ? measured : estimatedHeights[index];
      });
      recalcLayout(width, heights);
    };

    update();
    const observer = new ResizeObserver(update);
    observer.observe(container);
    return () => observer.disconnect();
  }, [estimatedHeights, items, measuredHeights, recalcLayout]);

  useEffect(() => {
    if (!layout.placements.length) return;
    const frame = window.requestAnimationFrame(syncMeasuredHeights);
    return () => window.cancelAnimationFrame(frame);
  }, [layout.placements, syncMeasuredHeights, items]);

  useEffect(() => {
    const observers: ResizeObserver[] = [];

    items.forEach((item) => {
      const node = itemRefs.current.get(item.id);
      if (!node) return;
      const observer = new ResizeObserver(() => syncMeasuredHeights());
      observer.observe(node);
      observers.push(observer);
    });

    return () => {
      observers.forEach((observer) => observer.disconnect());
    };
  }, [items, layout.placements, syncMeasuredHeights]);

  const getItemStyle = useCallback(
    (index: number): React.CSSProperties => {
      const placement = layout.placements[index];
      if (!placement || layout.columnWidth <= 0) {
        return { visibility: 'hidden' };
      }

      const left = placement.column * (layout.columnWidth + MASONRY_GAP_PX);
      return {
        position: 'absolute',
        top: placement.top,
        left,
        width: layout.columnWidth
      };
    },
    [layout.columnWidth, layout.placements]
  );

  return (
    <div
      ref={containerRef}
      className={`search-media-masonry search-media-masonry--${variant}`}
      style={{ height: layout.containerHeight > 0 ? layout.containerHeight : undefined }}
    >
      {items.map((item, index) => (
        <div
          key={item.id}
          ref={(node) => {
            if (node) {
              itemRefs.current.set(item.id, node);
            } else {
              itemRefs.current.delete(item.id);
            }
          }}
          className="search-media-masonry__item"
          style={getItemStyle(index)}
        >
          {renderItem(item, {
            width: '100%'
          })}
        </div>
      ))}
    </div>
  );
}
