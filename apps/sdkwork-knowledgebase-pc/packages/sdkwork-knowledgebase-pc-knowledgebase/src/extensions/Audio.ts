import { Node, mergeAttributes } from '@tiptap/core';

export const Audio = Node.create({
  name: 'audio',
  group: 'block',
  selectable: true,
  draggable: true,
  atom: true,

  addAttributes() {
    return {
      src: {
        default: null,
      },
      controls: {
        default: true,
      },
      preload: {
        default: 'metadata'
      }
    };
  },

  parseHTML() {
    return [
      {
        tag: 'audio',
      },
    ];
  },

  renderHTML({ HTMLAttributes }) {
    return ['audio', mergeAttributes(HTMLAttributes, { style: 'width: 100%; outline: none; margin: 20px 0;' })];
  },

  addNodeView() {
    return ({ node, HTMLAttributes }) => {
      const dom = document.createElement('audio');
      Object.entries(HTMLAttributes).forEach(([key, value]) => {
        if (value !== null) {
          dom.setAttribute(key, value);
        }
      });
      dom.style.width = '100%';
      dom.style.outline = 'none';
      dom.style.margin = '20px 0';
      return {
        dom,
      };
    };
  },
});
