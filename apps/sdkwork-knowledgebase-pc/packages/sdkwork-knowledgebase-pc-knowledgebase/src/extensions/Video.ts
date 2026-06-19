import { Node, mergeAttributes } from '@tiptap/core';

export const Video = Node.create({
  name: 'video',
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
      class: {
        default: 'w-full rounded-xl my-4 border border-[var(--color-kb-panel-border)] shadow-sm'
      }
    };
  },

  parseHTML() {
    return [
      {
        tag: 'video',
      },
    ];
  },

  renderHTML({ HTMLAttributes }) {
    return ['video', mergeAttributes(HTMLAttributes)];
  },

  addNodeView() {
    return ({ node, HTMLAttributes }) => {
      const dom = document.createElement('video');
      Object.entries(HTMLAttributes).forEach(([key, value]) => {
        if (value !== null) {
          dom.setAttribute(key, value);
        }
      });
      dom.className = 'w-full rounded-xl my-4 border border-[var(--color-kb-panel-border)] shadow-sm';
      dom.style.maxWidth = '100%';
      return {
        dom,
      };
    };
  },
});
