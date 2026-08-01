import { formatBytes } from '@sdkwork/utils';

export const MAX_DOMAIN_VERIFICATION_TEXT_BYTES = 64 * 1024;

export function isFileWithinSizeLimit(
  file: Pick<File, 'size'>,
  maximumBytes: number,
): boolean {
  return Number.isSafeInteger(file.size)
    && file.size >= 0
    && Number.isSafeInteger(maximumBytes)
    && maximumBytes > 0
    && file.size <= maximumBytes;
}

export function formatFileSizeLimit(maximumBytes: number): string {
  return formatBytes(maximumBytes, 0);
}
