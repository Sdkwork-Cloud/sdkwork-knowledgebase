import { toast } from '../ui/toast-manager';

export interface ExportPostHintInput {
  imageLoadFailures?: number;
  usedTiledRender?: boolean;
  canvasMayBeClipped?: boolean;
}

export function emitExportPostHints(input: ExportPostHintInput): void {
  if (input.imageLoadFailures && input.imageLoadFailures > 0) {
    toast.info(`${input.imageLoadFailures} 张图片未能加载，导出结果可能不完整`);
  }
  if (input.usedTiledRender) {
    toast.info('文档较长，已启用分段渲染以保证完整导出');
  }
  if (input.canvasMayBeClipped) {
    toast.info('图片高度超出浏览器限制，底部可能被截断，建议改用 PDF 导出');
  }
}
