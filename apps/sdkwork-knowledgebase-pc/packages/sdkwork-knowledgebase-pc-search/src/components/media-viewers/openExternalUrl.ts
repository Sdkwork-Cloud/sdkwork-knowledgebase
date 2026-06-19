import { dispatchOpenInAppBrowser } from '@packages/sdkwork-knowledgebase-pc-commons/src';

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
