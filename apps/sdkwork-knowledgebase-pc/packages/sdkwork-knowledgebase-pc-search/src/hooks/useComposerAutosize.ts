import { useEffect } from 'react';
import type { RefObject } from 'react';

export function useComposerAutosize(
  textareaRef: RefObject<HTMLTextAreaElement | null>,
  inputValue: string,
  maxHeight: number
) {
  useEffect(() => {
    const textarea = textareaRef.current;
    if (!textarea) return;
    textarea.style.height = 'auto';
    textarea.style.height = `${Math.min(textarea.scrollHeight, maxHeight)}px`;
  }, [inputValue, maxHeight, textareaRef]);
}
