import type { i18n } from 'i18next';
import enUS from './i18n/en-US/intelligence/provider/providerAdmin.json';
import zhCN from './i18n/zh-CN/intelligence/provider/providerAdmin.json';

const NAMESPACE = 'providerAdmin';

export function registerProviderAdminI18n(instance: i18n): void {
  if (!instance.hasResourceBundle('en-US', NAMESPACE)) {
    instance.addResourceBundle('en-US', NAMESPACE, enUS, true, false);
  }
  if (!instance.hasResourceBundle('zh-CN', NAMESPACE)) {
    instance.addResourceBundle('zh-CN', NAMESPACE, zhCN, true, false);
  }
}
