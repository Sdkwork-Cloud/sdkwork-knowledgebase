import i18n from '../i18n';
import { resolveKnowledgebaseAuthLocaleFromAppLanguage } from '../i18n/locale';

export interface SdkworkAuthAppearanceConfig {
  asidePanelClassName?: string;
  bodyClassName?: string;
  contentContainerClassName?: string;
  pageClassName?: string;
  qrFrameClassName?: string;
  shellClassName?: string;
  slotProps?: {
    background?: { className?: string };
    page?: { className?: string };
    shell?: { className?: string };
  };
  theme?: Record<string, string>;
}

export interface SdkworkAuthRuntimeConfig {
  leftRailMode?: string;
  loginMethods?: string[];
  oauthLoginEnabled?: boolean;
  oauthProviders?: string[];
  qrLoginEnabled?: boolean;
  recoveryMethods?: string[];
  registerMethods?: string[];
  verificationPolicy?: Record<string, boolean>;
  developmentPrefill?: Record<string, unknown>;
}

const KNOWLEDGEBASE_VERIFICATION_POLICY = {
  emailCodeLoginEnabled: false,
  emailRegistrationVerificationRequired: false,
  phoneCodeLoginEnabled: false,
  phoneRegistrationVerificationRequired: false,
};

export function resolveKnowledgebaseAuthRuntimeConfig(): SdkworkAuthRuntimeConfig {
  const config: SdkworkAuthRuntimeConfig = {
    leftRailMode: 'qr-only',
    loginMethods: ['password'],
    oauthLoginEnabled: false,
    oauthProviders: [],
    qrLoginEnabled: true,
    recoveryMethods: [],
    registerMethods: ['email', 'phone'],
    verificationPolicy: KNOWLEDGEBASE_VERIFICATION_POLICY,
  };

  if (import.meta.env.DEV) {
    const email = import.meta.env.VITE_SDKWORK_KNOWLEDGEBASE_AUTH_DEV_EMAIL;
    const password = import.meta.env.VITE_SDKWORK_KNOWLEDGEBASE_AUTH_DEV_PASSWORD;
    if (email && password) {
      config.developmentPrefill = {
        email,
        password,
      };
    }
  }

  return config;
}

export function resolveKnowledgebaseAuthAppearance(): SdkworkAuthAppearanceConfig {
  return {
    asidePanelClassName: 'sdkwork-knowledgebase-auth-aside-panel',
    bodyClassName: 'sdkwork-knowledgebase-auth-body',
    contentContainerClassName: 'sdkwork-knowledgebase-auth-content',
    pageClassName: 'sdkwork-knowledgebase-auth-page',
    qrFrameClassName: 'sdkwork-knowledgebase-auth-qr-frame',
    shellClassName: 'sdkwork-knowledgebase-auth-card-shell',
    slotProps: {
      background: {
        className: 'sdkwork-knowledgebase-auth-background',
      },
      page: {
        className: 'sdkwork-knowledgebase-auth-page',
      },
      shell: {
        className: 'sdkwork-knowledgebase-auth-card-shell',
      },
    },
    theme: {
      asideCardBackgroundColor: 'var(--sdkwork-knowledgebase-auth-aside-card-bg)',
      asideCardBorderColor: 'var(--sdkwork-knowledgebase-auth-aside-card-border)',
      asidePanelBackgroundColor: 'var(--sdkwork-knowledgebase-auth-aside-bg)',
      asidePanelBorderColor: 'var(--sdkwork-knowledgebase-auth-aside-border)',
      asidePanelColor: 'var(--sdkwork-knowledgebase-auth-aside-text)',
      badgeBackgroundColor: 'var(--sdkwork-knowledgebase-auth-aside-badge-bg)',
      badgeTextColor: 'var(--sdkwork-knowledgebase-auth-aside-badge-text)',
      contentBackgroundColor: 'var(--sdkwork-knowledgebase-auth-content-bg)',
      contentBorderColor: 'transparent',
      contentTextColor: 'var(--sdkwork-knowledgebase-auth-content-text)',
      descriptionColor: 'var(--sdkwork-knowledgebase-auth-muted-text)',
      dividerColor: 'var(--sdkwork-knowledgebase-auth-divider)',
      fieldBackgroundColor: 'var(--sdkwork-knowledgebase-auth-field-bg)',
      fieldBorderColor: 'transparent',
      fieldPlaceholderColor: '#9ca3af',
      fieldTextColor: 'var(--sdkwork-knowledgebase-auth-content-text)',
      formMutedTextColor: 'var(--sdkwork-knowledgebase-auth-muted-text)',
      iconMutedColor: 'var(--sdkwork-knowledgebase-auth-muted-text)',
      labelColor: 'var(--sdkwork-knowledgebase-auth-content-text)',
      pageBackgroundColor: 'var(--sdkwork-knowledgebase-auth-bg)',
      qrFrameBackgroundColor: 'var(--sdkwork-knowledgebase-auth-qr-bg)',
      qrFrameBorderColor: 'transparent',
      shellBackgroundColor: 'var(--sdkwork-knowledgebase-auth-content-bg)',
      shellBorderColor: 'transparent',
      tabActiveBackgroundColor: 'transparent',
      tabActiveTextColor: 'var(--sdkwork-knowledgebase-auth-content-text)',
      tabBackgroundColor: 'transparent',
      tabInactiveTextColor: 'var(--sdkwork-knowledgebase-auth-muted-text)',
      titleColor: 'var(--sdkwork-knowledgebase-auth-content-text)',
    },
  };
}

export function resolveKnowledgebaseAuthLocale(): string | null {
  if (typeof window === 'undefined') {
    return null;
  }

  return resolveKnowledgebaseAuthLocaleFromAppLanguage(i18n.language);
}
