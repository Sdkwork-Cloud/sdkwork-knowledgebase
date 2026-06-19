import { Node, mergeAttributes } from '@tiptap/core';

export const WechatMiniprogram = Node.create({
  name: 'wechatMiniprogram',
  group: 'inline',
  inline: true,
  atom: true,

  addAttributes() {
    return {
      'data-miniprogram-appid': { default: '' },
      'data-miniprogram-path': { default: '' },
      'data-miniprogram-title': { default: '' },
      'data-miniprogram-imageurl': { default: '' },
      'data-miniprogram-type': { default: 'text' },
      'data-miniprogram-nickname': { default: '小程序' }
    }
  },

  parseHTML() {
    return [
      {
        tag: 'mp-miniprogram',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['mp-miniprogram', mergeAttributes(HTMLAttributes)]
  },

  addNodeView() {
    return ({ node }) => {
      const type = node.attrs['data-miniprogram-type'];
      const title = node.attrs['data-miniprogram-title'];
      const imageUrl = node.attrs['data-miniprogram-imageurl'];
      const nickname = node.attrs['data-miniprogram-nickname'];

      const dom = document.createElement('span');
      dom.className = 'wechat-miniprogram-wrapper';
      dom.style.display = type === 'card' ? 'block' : 'inline-block';
      dom.contentEditable = 'false';

      if (type === 'card') {
        dom.innerHTML = `
          <div style="max-width: 340px; border: 1px solid var(--color-kb-panel-border, #e3e3e3); border-radius: 6px; padding: 12px; background: var(--color-kb-panel, #fff); font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif; cursor: pointer; user-select: none;">
            <div style="display: flex; align-items: center; margin-bottom: 8px;">
              <span style="width: 14px; height: 14px; border-radius: 50%; background: #eee; display: inline-block; margin-right: 6px;"></span>
              <span style="font-size: 13px; color: var(--color-kb-text-muted, #888);">${nickname || '小程序'}</span>
            </div>
            <div style="font-size: 15px; color: var(--color-kb-text-heading, #333); margin-bottom: 10px; font-weight: 400; line-height: 1.4; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; text-overflow: ellipsis; max-height: 42px;">${title || ''}</div>
            ${imageUrl ? `<div style="width: 100%; aspect-ratio: 5 / 4; background-image: url('${imageUrl}'); background-size: cover; background-position: center; border-radius: 4px;"></div>` : '<div style="width: 100%; aspect-ratio: 5 / 4; background: #f7f7f7; border-radius: 4px; display: flex; align-items: center; justify-content: center; color: #ccc; font-size: 12px;">无图片</div>'}
            <div style="font-size: 11px; color: var(--color-kb-text-muted, #999); margin-top: 10px; border-top: 1px solid var(--color-kb-panel-border, #eee); padding-top: 10px; display: flex; align-items: center;">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="margin-right: 4px;"><path d="m18 16 4-4-4-4"/><path d="m6 8-4 4 4 4"/><path d="m14.5 4-5 16"/></svg>
              小程序
            </div>
          </div>
        `;
      } else if (type === 'image') {
        dom.innerHTML = `
          <img src="${imageUrl}" style="max-width: 100%; height: auto; border-radius: 4px; border: 1px solid var(--color-kb-panel-border);" alt="${title}" />
        `;
      } else {
        dom.innerHTML = `<span style="color: #576b95; text-decoration: none; cursor: pointer;">${title || '小程序'}</span>`;
      }

      return {
        dom,
      }
    }
  }
});
