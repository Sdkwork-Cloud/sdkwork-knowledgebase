import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';

import zhCommon from './zh-CN/intelligence/knowledge/common.json';
import zhKb from './zh-CN/intelligence/knowledge/kb.json';
import zhEditor from './zh-CN/intelligence/knowledge/editor.json';
import zhShell from './zh-CN/intelligence/knowledge/shell.json';
import zhApplet from './zh-CN/intelligence/knowledge/applet.json';
import zhOfficialAccount from './zh-CN/intelligence/knowledge/officialAccount.json';
import zhWidget from './zh-CN/intelligence/knowledge/widget.json';
import zhCloudDrive from './zh-CN/intelligence/knowledge/cloudDrive.json';
import zhMcp from './zh-CN/intelligence/knowledge/mcp.json';
import zhSearch from './zh-CN/intelligence/knowledge/search.json';
import zhErrors from './zh-CN/intelligence/knowledge/errors.json';

import enCommon from './en-US/intelligence/knowledge/common.json';
import enKb from './en-US/intelligence/knowledge/kb.json';
import enEditor from './en-US/intelligence/knowledge/editor.json';
import enShell from './en-US/intelligence/knowledge/shell.json';
import enApplet from './en-US/intelligence/knowledge/applet.json';
import enOfficialAccount from './en-US/intelligence/knowledge/officialAccount.json';
import enWidget from './en-US/intelligence/knowledge/widget.json';
import enCloudDrive from './en-US/intelligence/knowledge/cloudDrive.json';
import enMcp from './en-US/intelligence/knowledge/mcp.json';
import enSearch from './en-US/intelligence/knowledge/search.json';
import enErrors from './en-US/intelligence/knowledge/errors.json';

import {
  normalizeAppLanguage,
  persistAppLanguage,
  resolveInitialAppLanguage,
} from '../runtime/locale';

const initialLanguage = resolveInitialAppLanguage();

void i18n
  .use(initReactI18next)
  .init({
    resources: {
      'zh-CN': {
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
      'en-US': {
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
    fallbackLng: 'en-US',
    supportedLngs: ['zh-CN', 'en-US'],
    nonExplicitSupportedLngs: false,
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
