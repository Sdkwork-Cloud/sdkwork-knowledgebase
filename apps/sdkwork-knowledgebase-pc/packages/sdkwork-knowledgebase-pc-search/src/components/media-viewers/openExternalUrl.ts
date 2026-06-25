import { dispatchOpenInAppBrowser } from '@sdkwork/sdkwork-knowledgebase-pc-commons';

export function openExternalUrl(
  url: string,
  title: string | undefined,
  onOpenWebLink?: (url: string, title?: string) => void
) {
  if (onOpenWebLink) {
    onOpenWebLink(url, title);
    return;
  }
  dispatchOpenInAppBrowser({ url, title });
}
