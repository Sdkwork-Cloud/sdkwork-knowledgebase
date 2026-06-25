import React from 'react';
import { CHAT_LAYOUT_MAX } from '../constants';

interface SearchComposerDockProps {
  children: React.ReactNode;
}

/** Bottom dock — aligns composer with the chat thread column */
export function SearchComposerDock({ children }: SearchComposerDockProps) {
  return (
    <div className="shrink-0 w-full search-composer-dock bg-[var(--color-kb-editor)]">
      <div className={`${CHAT_LAYOUT_MAX} mx-auto w-full px-4 md:px-6 pb-3 pt-2`}>{children}</div>
    </div>
  );
}
