import i18n from 'i18next';
import { escapeHtmlText } from '@sdkwork/sdkwork-knowledgebase-pc-commons/htmlSanitizer';

const t = (key: string, options?: any) => i18n.t(key, { ns: 'widget', ...options });

function safeExternalUrl(url: string): string {
  const trimmed = url.trim();
  if (!trimmed) {
    return '#';
  }
  try {
    const parsed = new URL(trimmed, window.location.origin);
    if (parsed.protocol === 'http:' || parsed.protocol === 'https:') {
      return escapeHtmlText(parsed.href);
    }
  } catch {
    return '#';
  }
  return '#';
}

export const WechatWidgetTemplates = {
  link: (title: string, url: string) => `
    <p><a href="${safeExternalUrl(url)}" target="_blank" rel="noopener noreferrer" style="color: var(--color-kb-accent); text-decoration: none; border-bottom: 2px solid var(--color-kb-accent); font-weight: bold; font-size: 14.5px;">🔗 ${escapeHtmlText(title)}</a></p>
  `,
  coupons: (quota: string, merchant: string, condition: string) => `
    <div style="border: 1px solid #ff4d4f; border-radius: 12px; background: #fff1f0; padding: 16px; display: flex; align-items: center; margin: 20px 0; justify-content: space-between; position: relative; overflow: hidden;">
      <div>
        <div style="color: #ff4d4f; font-weight: 800; font-size: 18px;">${quota}</div>
        <div style="color: #555; font-size: 12px; margin-top: 4px; font-weight: bold;">${merchant} · ${condition}</div>
      </div>
      <button style="background: #ff4d4f; color: white; border: none; padding: 6px 14px; border-radius: 20px; font-size: 12px; font-weight: bold; cursor: pointer; box-shadow: 0 4px 10px rgba(255,77,79,0.25);">${t('receiveCoupon')}</button>
    </div>
  `,
  templates: () => `
    <div style="margin: 24px 0; border-left: 4px solid var(--color-kb-accent); padding-left: 16px; background-color: var(--color-kb-panel-hover); padding-top: 12px; padding-bottom: 12px; border-top-right-radius: 8px; border-bottom-right-radius: 8px;">
        <h3 style="font-size: 16px; font-weight: bold; color: var(--color-kb-text-heading); margin-bottom: 6px;">${t('themeCardTitle')}</h3>
        <p style="font-size: 13.5px; color: var(--color-kb-text-muted); line-height: 1.6; margin: 0;">${t('themeCardDesc')}</p>
    </div>
  `,
  vote: (title: string, options: string[]) => `
    <div style="border: 1px solid var(--color-kb-panel-border); border-radius: 12px; padding: 16px; background: var(--color-kb-panel-hover); margin: 24px 0;">
      <div style="font-weight: bold; font-size: 14.5px; margin-bottom: 12px; color: var(--color-kb-text-heading); display: flex; align-items: center; gap: 8px;">
          <span>${t('voteTitlePrefix')}</span> ${title}
      </div>
      ${options.map((opt) => `
        <div style="padding: 10px 14px; border-radius: 8px; background: var(--color-kb-panel); border: 1px solid var(--color-kb-panel-border); margin-bottom: 8px; font-size: 13px; color: var(--color-kb-text); display: flex; justify-content: space-between; align-items: center;">
          <span>${opt}</span>
          <span style="font-size: 11px; color: var(--color-kb-accent); font-weight: 500;">${t('voteBtn')}</span>
        </div>
      `).join('')}
      <div style="font-size: 10.5px; color: var(--color-kb-text-muted); text-align: center; margin-top: 8px;">${t('voteFooter')}</div>
    </div>
  `,
  search: (title: string) => `
    <div style="padding: 14px; border: 1px solid var(--color-kb-panel-border); border-radius: 12px; background: var(--color-kb-panel); margin: 20px 0; display: flex; align-items: center; gap: 10px;">
      <div style="flex: 1; display: flex; align-items: center; gap: 8px; border-radius: 20px; background: var(--color-kb-panel-hover); padding: 8px 16px; font-size: 13px; color: var(--color-kb-text-muted);">
        <span>${t('searchPrefix')}</span><strong style="color: var(--color-kb-text-heading); font-weight: bold;">${title}</strong>
      </div>
      <span style="color: var(--color-kb-text-muted); font-size: 11px;">${t('directSearch')}</span>
    </div>
  `,
  location: (title: string, subtitle: string) => `
    <div style="display: flex; align-items: center; padding: 14px; border: 1px solid var(--color-kb-panel-border); border-radius: 12px; background: var(--color-kb-panel-hover); margin: 20px 0;">
      <div style="font-size: 20px; padding: 6px; background: rgba(59,130,246,0.1); border-radius: 8px;">📍</div>
      <div style="margin-left: 12px; flex: 1;">
        <div style="font-weight: bold; font-size: 14px; color: var(--color-kb-text-heading);">${title}</div>
        <div style="font-size: 11.5px; color: var(--color-kb-text-muted); margin-top: 2px;">${subtitle}</div>
      </div>
    </div>
  `,
  channel: (title: string) => `
    <div style="border: 1px solid var(--color-kb-panel-border); border-radius: 12px; overflow: hidden; background: var(--color-kb-panel); margin: 24px 0;">
      <div style="display: flex; align-items: center; gap: 10px; padding: 12px; border-bottom: 1px solid var(--color-kb-panel-border);">
        <div style="font-size: 14px;">🎬</div>
        <span style="font-weight: bold; font-size: 13px; color: var(--color-kb-text-heading); flex: 1;">${t('channelPrefix')}${title}</span>
        <span style="font-size: 11px; color: var(--color-kb-text-muted);">${t('watchOriginal')}</span>
      </div>
      <div style="position: relative; height: 160px; background: linear-gradient(135deg, #0f172a 0%, #334155 100%); display: flex; align-items: center; justify-content: center;">
        <div style="position: relative; width: 48px; height: 48px; border-radius: 50%; background: #fff; display: flex; align-items: center; justify-content: center;">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="#111">
            <path d="M8 5v14l11-7z"/>
          </svg>
        </div>
      </div>
    </div>
  `,
  qa: (question: string, answer: string) => `
    <div style="border: 1px solid rgba(59,130,246,0.15); border-radius: 10px; padding: 14px; background: rgba(59,130,246,0.02); margin: 20px 0;">
      <div style="font-weight: bold; color: var(--color-kb-accent); margin-bottom: 8px; font-size: 14.5px;">❓ Q：${question}</div>
      <div style="color: var(--color-kb-text-heading); font-size: 13.5px; line-height: 1.5; font-weight: 500;">📝 A：${answer}</div>
    </div>
  `,
  ad: () => `
    <div style="border: 1px solid var(--color-kb-panel-border); border-radius: 12px; overflow: hidden; background: var(--color-kb-panel-hover); margin: 24px 0;">
      <div style="width: 100%; height: 150px; background: linear-gradient(135deg, #1e293b 0%, #475569 100%); display: flex; align-items: center; justify-content: center; color: #cbd5e1; font-size: 12px; letter-spacing: 0.08em;">${t('adTitle')}</div>
      <div style="padding: 12px; display: flex; justify-content: space-between; align-items: center;">
        <div>
          <div style="font-size: 13.5px; font-weight: bold; color: var(--color-kb-text-heading);">${t('adTitle')}</div>
          <div style="font-size: 11px; color: var(--color-kb-text-muted); margin-top: 2px;">${t('adSubtitle')}</div>
        </div>
        <span style="border: 1px solid var(--color-kb-panel-border); font-size: 10px; color: var(--color-kb-text-muted); padding: 2px 5px; border-radius: 3px;">${t('adLabel')}</span>
      </div>
    </div>
  `,
  card: (title: string, subtitle: string) => `
    <div style="border: 1px solid var(--color-kb-panel-border); border-radius: 12px; padding: 16px; display: flex; align-items: center; gap: 12px; margin: 24px 0; background: var(--color-kb-panel);">
      <div style="width: 44px; height: 44px; border-radius: 50%; background: var(--color-kb-accent); display: flex; align-items: center; justify-content: center; font-weight: bold; color: white;">SW</div>
      <div style="flex: 1;">
        <div style="font-weight: bold; font-size: 14px; color: var(--color-kb-text-heading);">${title}</div>
        <div style="font-size: 11.5px; color: var(--color-kb-text-muted); margin-top: 2px; line-height: 1.4;">${subtitle}</div>
      </div>
      <button style="border: none; background: #07c160; color: white; padding: 6px 14px; border-radius: 16px; font-size: 12px; font-weight: bold; cursor: pointer;">${t('followBtn')}</button>
    </div>
  `,
  gifts: () => `
    <div style="text-align: center; margin: 24px 0; padding: 20px; border-radius: 12px; background: rgba(245,158,11,0.02); border: 1px dashed rgba(245,158,11,0.2);">
      <div style="font-size: 13.5px; color: #8a6d3b; margin-bottom: 12px;">${t('giftTitle')}</div>
      <div style="display: flex; justify-content: center; gap: 10px;">
        <button style="border: none; background: #ea580c; color: white; padding: 8px 20px; border-radius: 20px; font-size: 12.5px; font-weight: bold; cursor: pointer;">${t('giftBtnSmall')}</button>
        <button style="border: 1px solid #ea580c; background: transparent; color: #ea580c; padding: 6px 18px; border-radius: 20px; font-size: 12.5px; font-weight: bold; cursor: pointer;">${t('giftBtnAny')}</button>
      </div>
    </div>
  `,
  appletCard: (title: string, icon: string, extensionBg: string, extensionColor: string, sizeText: string, timeText: string, actionBtnText: string) => `
    <div style="border: 1.5px solid var(--color-kb-panel-border); border-radius: 12px; padding: 16px; margin: 24px auto; background: var(--color-kb-panel); max-width: 580px; box-shadow: 0 4px 16px rgba(0,0,0,0.03); font-family: system-ui, -apple-system, sans-serif;">
      <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; border-bottom: 1px solid var(--color-kb-panel-border); padding-bottom: 8px;">
        <div style="display: flex; align-items: center; gap: 8px;">
          <span style="font-size: 16px;">🧩</span>
          <span style="font-weight: bold; font-size: 13px; color: var(--color-kb-text-heading); letter-spacing: 0.3px;">${t('appletCardTitle')}</span>
        </div>
        <span style="color: #07c160; background: rgba(7, 193, 96, 0.09); font-size: 10px; font-weight: bold; padding: 2px 8px; border-radius: 100px;">${t('appletAccelerate')}</span>
      </div>
      
      <div style="display: flex; align-items: flex-start; gap: 12px; padding: 14px; background: var(--color-kb-panel-hover); border-radius: 10px; border: 1.5px solid var(--color-kb-panel-border);">
        <div style="width: 44px; height: 44px; border-radius: 10px; background: ${extensionBg}; display: flex; align-items: center; justify-content: center; font-weight: 800; color: ${extensionColor}; font-size: 18px; flex-shrink: 0;">
          ${icon}
        </div>
        <div style="flex: 1; min-width: 0; text-align: left;">
          <div style="font-size: 14px; font-weight: bold; color: var(--color-kb-text-heading); margin-bottom: 4px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            ${title}
          </div>
          <div style="font-size: 11.5px; color: var(--color-kb-text-muted);">
            ${sizeText}${timeText}
          </div>
        </div>
      </div>
      
      <div style="display: flex; align-items: center; justify-content: space-between; margin-top: 12px; font-size: 11px; color: var(--color-kb-text-muted);">
        <span style="display: flex; align-items: center; gap: 4px;">
          <span style="display:inline-block; width:6px; height:6px; border-radius:50%; background-color: #07c160;"></span>
          ${t('appletDirectAccess')}
        </span>
        <span style="font-weight: 800; color: var(--color-kb-accent); cursor: pointer; text-transform: uppercase;">${actionBtnText} →</span>
      </div>
    </div>
    <p></p>
  `,
  fallback: (toolName: string) => `
    <div style="border: 1px dashed var(--color-kb-panel-border); padding: 12px; margin: 16px 0; border-radius: 8px; text-align: center; color: var(--color-kb-text-muted); font-size: 13px;">
      ${t('fallbackWidget', { toolName })}
    </div>
  `
};
