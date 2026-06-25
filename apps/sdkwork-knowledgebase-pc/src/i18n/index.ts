import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import zhCommon from './locales/zh/common.json';
import zhKb from './locales/zh/kb.json';
import zhEditor from './locales/zh/editor.json';
import zhShell from './locales/zh/shell.json';
import zhApplet from './locales/zh/applet.json';
import zhOfficialAccount from './locales/zh/officialAccount.json';
import zhWidget from './locales/zh/widget.json';
import zhCloudDrive from './locales/zh/cloudDrive.json';
import zhMcp from './locales/zh/mcp.json';
import zhSearch from './locales/zh/search.json';
import zhErrors from './locales/zh/errors.json';

import enCommon from './locales/en/common.json';
import enKb from './locales/en/kb.json';
import enEditor from './locales/en/editor.json';
import enShell from './locales/en/shell.json';
import enApplet from './locales/en/applet.json';
import enOfficialAccount from './locales/en/officialAccount.json';
import enWidget from './locales/en/widget.json';
import enCloudDrive from './locales/en/cloudDrive.json';
import enMcp from './locales/en/mcp.json';
import enSearch from './locales/en/search.json';
import enErrors from './locales/en/errors.json';

import {
  normalizeAppLanguage,
  persistAppLanguage,
  resolveInitialAppLanguage,
} from './locale';

const initialLanguage = resolveInitialAppLanguage();

void i18n
  .use(initReactI18next)
  .init({
    resources: {
      zh: {
        common: zhCommon,
        kb: zhKb,
        knowledgebase: zhKb,
        editor: zhEditor,
        shell: zhShell,
        applet: zhApplet,
        officialAccount: zhOfficialAccount,
        widget: zhWidget,
        cloudDrive: zhCloudDrive,
        mcp: zhMcp,
        search: zhSearch,
        errors: zhErrors,
      },
      en: {
        common: enCommon,
        kb: enKb,
        knowledgebase: enKb,
        editor: enEditor,
        shell: enShell,
        applet: enApplet,
        officialAccount: enOfficialAccount,
        widget: enWidget,
        cloudDrive: enCloudDrive,
        mcp: enMcp,
        search: enSearch,
        errors: enErrors,
      },
    },
    lng: initialLanguage,
    fallbackLng: 'en',
    supportedLngs: ['zh', 'en'],
    nonExplicitSupportedLngs: true,
    ns: [
      'common',
      'kb',
      'knowledgebase',
      'editor',
      'shell',
      'applet',
      'officialAccount',
      'widget',
      'cloudDrive',
      'mcp',
      'search',
      'errors',
    ],
    defaultNS: 'common',
    interpolation: {
      escapeValue: false,
    },
  });

i18n.on('languageChanged', (language) => {
  persistAppLanguage(normalizeAppLanguage(language));
});

export default i18n;
