import {
  detectOperatingSystem,
  getExportRuntimeEnvironment,
  isDesktopExportHost,
  isSaveFilePickerAvailable,
} from './exportRuntime';
import type { DocumentExportSourceKind, DocumentPdfExportEngine } from './types';

export interface DocumentExportCapabilities {
  isNativeHost: boolean;
  canRevealInFolder: boolean;
  canOpenExportFile: boolean;
  saveAsAvailable: boolean;
  defaultSaveHint: string;
  pdfEngine: DocumentPdfExportEngine;
  pdfEngineLabel: string;
  pdfEngineDescription: string;
  platformLabel: string;
}

function resolvePdfEngine(sourceKind?: DocumentExportSourceKind): DocumentPdfExportEngine {
  if (!isDesktopExportHost()) {
    return 'canvas';
  }
  if (sourceKind === 'markdown') {
    return 'native-markdown';
  }
  if (detectOperatingSystem() === 'windows') {
    return 'native-webview';
  }
  return 'canvas';
}

function resolveCanvasDescription(sourceKind?: DocumentExportSourceKind): string {
  const os = detectOperatingSystem();
  const isDesktop = isDesktopExportHost();

  if (isDesktop && sourceKind === 'richtext' && os !== 'windows') {
    if (os === 'macos') {
      return 'macOS 富文本暂通过浏览器渲染导出；Markdown 仍可使用 Typst 矢量 PDF。';
    }
    if (os === 'linux') {
      return 'Linux 富文本暂通过浏览器渲染导出；Markdown 仍可使用 Typst 矢量 PDF。';
    }
    return '当前平台富文本暂通过浏览器渲染导出；Markdown 仍可使用 Typst 矢量 PDF。';
  }

  const runtime = getExportRuntimeEnvironment();
  if (runtime.kind === 'web-browser') {
    if (runtime.isMobile) {
      return '移动端浏览器通过 Canvas 渲染导出，部分机型可能需手动保存下载文件。';
    }
    if (runtime.browser === 'safari') {
      return 'Safari 通过 Canvas 渲染导出，复杂排版与超长文档可能略有差异。';
    }
    if (runtime.browser === 'firefox') {
      return 'Firefox 通过 Canvas 渲染导出，复杂排版可能略有差异。';
    }
  }

  return '通过浏览器 Canvas 渲染导出，复杂排版可能略有差异。';
}

const PDF_ENGINE_META: Record<
  DocumentPdfExportEngine,
  { label: string; buildDescription: (sourceKind?: DocumentExportSourceKind) => string }
> = {
  'native-markdown': {
    label: 'Typst 矢量',
    buildDescription: () => 'Markdown 通过 Typst 原生引擎导出，文字清晰可缩放。',
  },
  'native-webview': {
    label: 'WebView 高清',
    buildDescription: () => '富文本通过 WebView 所见即所得导出，保留排版样式。',
  },
  canvas: {
    label: '浏览器渲染',
    buildDescription: resolveCanvasDescription,
  },
};

function resolvePlatformLabel(): string {
  const runtime = getExportRuntimeEnvironment();
  if (runtime.kind === 'tauri-desktop') {
    switch (runtime.os) {
      case 'windows':
        return 'Windows 桌面版';
      case 'macos':
        return 'macOS 桌面版';
      case 'linux':
        return 'Linux 桌面版';
      default:
        return '桌面版';
    }
  }

  if (runtime.isMobile) {
    switch (runtime.os) {
      case 'ios':
        return 'iOS 浏览器';
      case 'android':
        return 'Android 浏览器';
      default:
        return '移动浏览器';
    }
  }

  switch (runtime.browser) {
    case 'chrome':
      return 'Chrome 浏览器';
    case 'edge':
      return 'Edge 浏览器';
    case 'firefox':
      return 'Firefox 浏览器';
    case 'safari':
      return 'Safari 浏览器';
    default:
      return 'Web 浏览器';
  }
}

export function getDocumentExportCapabilities(
  sourceKind?: DocumentExportSourceKind,
): DocumentExportCapabilities {
  const isNativeHost = isDesktopExportHost();
  const pdfEngine = resolvePdfEngine(sourceKind);
  const engineMeta = PDF_ENGINE_META[pdfEngine];
  const saveAsAvailable = isSaveFilePickerAvailable();

  return {
    isNativeHost,
    canRevealInFolder: isNativeHost,
    canOpenExportFile: isNativeHost,
    saveAsAvailable,
    defaultSaveHint: isNativeHost
      ? '默认保存至系统「下载」文件夹'
      : saveAsAvailable
        ? '浏览器将弹出保存对话框'
        : '浏览器将保存至默认下载位置',
    pdfEngine,
    pdfEngineLabel: engineMeta.label,
    pdfEngineDescription: engineMeta.buildDescription(sourceKind),
    platformLabel: resolvePlatformLabel(),
  };
}
