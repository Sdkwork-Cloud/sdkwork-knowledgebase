import { createHostAdapter } from 'sdkwork-knowledgebase-pc-core/host/hostAdapter';

const EXPORT_PROSE_STYLES = `
  .prose h1, .prose h2, .prose h3 { color: #09090b; font-weight: 700; margin-top: 1.6em; margin-bottom: 0.6em; }
  .prose h1 { font-size: 1.5em; letter-spacing: -0.025em; }
  .prose h2 { font-size: 1.25em; border-bottom: 1px solid #f4f4f5; padding-bottom: 0.3em; letter-spacing: -0.015em; }
  .prose h3 { font-size: 1.1em; }
  .prose p { margin-bottom: 1.1em; margin-top: 0; color: #27272a; line-height: 1.65; }
  .prose blockquote { border-left: 4.5px solid #4f46e5; padding-left: 1.1rem; color: #71717a; font-style: italic; margin-left: 0; margin-bottom: 1.1em; }
  .prose pre { background: #f8f8f9; padding: 1rem; border-radius: 8px; overflow-x: auto; font-family: monospace; font-size: 0.85em; border: 1px solid #e4e4e7; margin-bottom: 1.1em; white-space: pre-wrap; word-break: break-word; }
  .prose code { background: #f4f4f5; padding: 0.15rem 0.3rem; border-radius: 4px; font-family: monospace; font-size: 0.9em; color: #09090b; }
  .prose ul { list-style-type: disc; padding-left: 1.5rem; margin-bottom: 1.1em; }
  .prose ol { list-style-type: decimal; padding-left: 1.5rem; margin-bottom: 1.1em; }
  .prose li { margin-bottom: 0.5em; color: #27272a; }
  .prose img { max-width: 100%; border-radius: 8px; margin: 1em 0; border: 1px solid #f4f4f5; }
  .prose table { width: 100%; border-collapse: collapse; margin-bottom: 1.1em; }
  .prose th, .prose td { border: 1px solid #e4e4e7; padding: 0.5rem 0.75rem; text-align: left; }
  .prose th { background: #f8f8f9; font-weight: 600; }
`;

export interface BuildExportContainerOptions {
  title: string;
  htmlContent: string;
  /** Flat layout without card chrome — better for PDF pagination. */
  flat?: boolean;
}

export function buildExportContainer({
  title,
  htmlContent,
  flat = false,
}: BuildExportContainerOptions): HTMLDivElement {
  const container = document.createElement('div');
  container.style.position = 'absolute';
  container.style.top = '-9999px';
  container.style.left = '-9999px';
  container.style.width = '760px';
  container.className = flat
    ? 'bg-white text-zinc-800 p-10 font-sans flex flex-col gap-6'
    : 'bg-white text-zinc-800 p-10 font-sans flex flex-col gap-6 rounded-xl border border-zinc-100 shadow-lg';

  const header = document.createElement('div');
  header.className = 'border-b pb-6 flex flex-col gap-2';

  const categoryTag = document.createElement('div');
  categoryTag.className = 'text-xs font-bold uppercase tracking-wider text-indigo-600 mb-0.5';
  categoryTag.innerText = '极简知识库 · 精彩笔记分享';

  const titleEl = document.createElement('h1');
  titleEl.className = 'text-3xl font-extrabold tracking-tight text-zinc-900 m-0 leading-tight';
  titleEl.innerText = title || '无标题';

  const dateEl = document.createElement('div');
  dateEl.className = 'text-xs text-zinc-400 font-medium';
  dateEl.innerText = `生成时间：${new Date().toLocaleString()}`;

  header.appendChild(categoryTag);
  header.appendChild(titleEl);
  header.appendChild(dateEl);
  container.appendChild(header);

  const style = document.createElement('style');
  style.innerHTML = EXPORT_PROSE_STYLES;
  container.appendChild(style);

  const content = document.createElement('div');
  content.className = 'prose max-w-none text-[14.5px] leading-relaxed text-zinc-700 space-y-4';
  content.innerHTML = htmlContent;
  container.appendChild(content);

  const footer = document.createElement('div');
  footer.className = 'border-t pt-5 mt-4 flex justify-between items-center text-[11px] text-zinc-400 font-medium';

  const leftFoot = document.createElement('div');
  leftFoot.innerText = 'Power by 极简知识库 AI Writing';

  const rightFoot = document.createElement('div');
  rightFoot.className = 'italic text-indigo-500 font-semibold';
  rightFoot.innerText = 'Minimalist Knowledge Base';

  footer.appendChild(leftFoot);
  footer.appendChild(rightFoot);
  container.appendChild(footer);

  return container;
}

export async function renderExportCanvas(
  container: HTMLDivElement,
  scale = 2,
): Promise<HTMLCanvasElement> {
  document.body.appendChild(container);
  await new Promise((resolve) => setTimeout(resolve, 800));

  try {
    const html2canvas = (await import('html2canvas')).default;
    return await html2canvas(container, {
      useCORS: true,
      allowTaint: true,
      backgroundColor: '#ffffff',
      scale,
    });
  } finally {
    document.body.removeChild(container);
  }
}

export async function canvasToPdfBytes(canvas: HTMLCanvasElement): Promise<Uint8Array> {
  const { jsPDF } = await import('jspdf');
  const pdf = new jsPDF({ orientation: 'portrait', unit: 'mm', format: 'a4' });
  const pageWidth = pdf.internal.pageSize.getWidth();
  const pageHeight = pdf.internal.pageSize.getHeight();
  const pxPerMm = canvas.width / pageWidth;
  const pageHeightPx = pageHeight * pxPerMm;

  let renderedHeight = 0;
  let pageIndex = 0;

  while (renderedHeight < canvas.height) {
    const sliceHeight = Math.min(pageHeightPx, canvas.height - renderedHeight);
    const pageCanvas = document.createElement('canvas');
    pageCanvas.width = canvas.width;
    pageCanvas.height = sliceHeight;

    const ctx = pageCanvas.getContext('2d');
    if (!ctx) {
      throw new Error('Failed to create PDF page canvas.');
    }

    ctx.fillStyle = '#ffffff';
    ctx.fillRect(0, 0, pageCanvas.width, pageCanvas.height);
    ctx.drawImage(
      canvas,
      0,
      renderedHeight,
      canvas.width,
      sliceHeight,
      0,
      0,
      canvas.width,
      sliceHeight,
    );

    const imgData = pageCanvas.toDataURL('image/jpeg', 0.92);
    const sliceImgHeight = (sliceHeight * pageWidth) / canvas.width;

    if (pageIndex > 0) {
      pdf.addPage();
    }
    pdf.addImage(imgData, 'JPEG', 0, 0, pageWidth, sliceImgHeight);

    renderedHeight += sliceHeight;
    pageIndex += 1;
  }

  const buffer = pdf.output('arraybuffer');
  return new Uint8Array(buffer);
}

export async function savePdfFile(bytes: Uint8Array, suggestedName: string): Promise<void> {
  const host = createHostAdapter();
  const saved = await host.saveBinaryResource(suggestedName, bytes);
  if (saved) {
    return;
  }

  const blob = new Blob([bytes], { type: 'application/pdf' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = suggestedName;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}
