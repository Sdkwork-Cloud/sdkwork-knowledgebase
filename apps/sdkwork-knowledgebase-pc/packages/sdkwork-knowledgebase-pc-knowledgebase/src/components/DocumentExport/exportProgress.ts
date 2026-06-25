import { toast } from '../ui/toast-manager';

export const EXPORT_PROGRESS_TOAST_KEY = 'document-export-progress';

export function showExportProgress(message: string) {
  toast.info({
    message,
    key: EXPORT_PROGRESS_TOAST_KEY,
    duration: 120_000,
  });
}

export function dismissExportProgress() {
  toast.dismiss(EXPORT_PROGRESS_TOAST_KEY);
}
