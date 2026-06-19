import { useCallback, useEffect } from 'react';
import { useTranslation } from 'react-i18next';

import { listenDesktopOpenSettings, syncTrayLocale } from './settingsDesktopBridge';
import type { SettingsTabId } from './settingsModalConstants';

export function useDesktopHostIntegration(
  isDesktopRuntime: boolean,
  openSettings: (tab?: SettingsTabId | string) => void,
): void {
  const { t, i18n } = useTranslation('shell');

  const pushTrayLocale = useCallback(() => {
    if (!isDesktopRuntime) {
      return;
    }

    void syncTrayLocale({
      showLabel: t('trayShowWindow'),
      settingsLabel: t('trayOpenSettings'),
      quitLabel: t('trayQuit'),
      tooltip: t('trayTooltip'),
    });
  }, [isDesktopRuntime, t]);

  useEffect(() => {
    pushTrayLocale();
  }, [pushTrayLocale]);

  useEffect(() => {
    if (!isDesktopRuntime) {
      return undefined;
    }

    const onLanguageChanged = () => {
      pushTrayLocale();
    };

    i18n.on('languageChanged', onLanguageChanged);
    return () => {
      i18n.off('languageChanged', onLanguageChanged);
    };
  }, [i18n, isDesktopRuntime, pushTrayLocale]);

  useEffect(() => {
    if (!isDesktopRuntime) {
      return undefined;
    }

    let unlisten: (() => void) | undefined;
    void listenDesktopOpenSettings(() => {
      openSettings('general');
    }).then((dispose) => {
      unlisten = dispose;
    });

    return () => {
      unlisten?.();
    };
  }, [isDesktopRuntime, openSettings]);
}
