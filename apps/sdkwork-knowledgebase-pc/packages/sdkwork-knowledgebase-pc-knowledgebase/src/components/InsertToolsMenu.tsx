import React, { useState, useEffect, useRef, useCallback } from 'react';
import { createPortal } from 'react-dom';

export interface InsertToolItem {
  name: string;
  hasDropdown: boolean;
  onClick: () => void;
  dropdownContent?: React.ReactNode;
}

export interface InsertToolsMenuProps {
  tools: InsertToolItem[];
}

function computeDropdownStyle(button: HTMLButtonElement): React.CSSProperties {
  const rect = button.getBoundingClientRect();
  return {
    position: 'fixed',
    top: rect.bottom + 6,
    left: rect.left + rect.width / 2,
    transform: 'translateX(-50%)',
  };
}

export function InsertToolsMenu({ tools }: InsertToolsMenuProps) {
  const [openMenuName, setOpenMenuName] = useState<string | null>(null);
  const [menuStyle, setMenuStyle] = useState<React.CSSProperties | null>(null);
  const buttonRefs = useRef<Record<string, HTMLButtonElement | null>>({});

  const closeMenu = useCallback(() => {
    setOpenMenuName(null);
    setMenuStyle(null);
  }, []);

  const repositionMenu = useCallback(() => {
    if (!openMenuName) {
      return;
    }
    const button = buttonRefs.current[openMenuName];
    if (!button) {
      closeMenu();
      return;
    }
    setMenuStyle(computeDropdownStyle(button));
  }, [closeMenu, openMenuName]);

  useEffect(() => {
    const handleOutsideClick = (event: MouseEvent) => {
      const target = event.target as HTMLElement;
      if (
        !target.closest('.insert-tool-item-container') &&
        !target.closest('[data-insert-tool-dropdown]')
      ) {
        closeMenu();
      }
    };
    document.addEventListener('mousedown', handleOutsideClick);
    return () => document.removeEventListener('mousedown', handleOutsideClick);
  }, [closeMenu]);

  useEffect(() => {
    if (!openMenuName) {
      return undefined;
    }

    repositionMenu();
    window.addEventListener('resize', repositionMenu);
    window.addEventListener('scroll', repositionMenu, true);
    return () => {
      window.removeEventListener('resize', repositionMenu);
      window.removeEventListener('scroll', repositionMenu, true);
    };
  }, [openMenuName, repositionMenu]);

  const openTool = tools.find((tool) => tool.name === openMenuName);

  return (
    <>
      <div className="flex items-center gap-1.5 md:gap-1.5 ml-0 md:ml-1 shrink-0">
        {tools.map((tool, idx) => {
          const isOpen = openMenuName === tool.name;

          return (
            <div className="insert-tool-item-container relative overflow-visible shrink-0" key={idx}>
              <button
                type="button"
                ref={(node) => {
                  buttonRefs.current[tool.name] = node;
                }}
                onClick={(event) => {
                  event.stopPropagation();
                  if (tool.hasDropdown) {
                    if (isOpen) {
                      closeMenu();
                      return;
                    }
                    const button = buttonRefs.current[tool.name];
                    if (button) {
                      setMenuStyle(computeDropdownStyle(button));
                    }
                    setOpenMenuName(tool.name);
                  } else {
                    closeMenu();
                    tool.onClick();
                  }
                }}
                className={`cursor-pointer transition-all font-semibold text-[10.5px] md:text-[11.5px] px-1 md:px-2 py-1 rounded-md flex items-center gap-0.5 select-none shrink-0 ${
                  isOpen
                    ? 'text-[#07c160] dark:text-[#07c160] bg-[var(--color-kb-panel-hover)]'
                    : 'text-[var(--color-kb-text-muted)] hover:text-[#07c160] dark:hover:text-[#07c160] hover:bg-[var(--color-kb-panel-hover)]'
                }`}
              >
                <span className="whitespace-nowrap">{tool.name}</span>
                {tool.hasDropdown && (
                  <span className="text-[9px] opacity-75 font-normal ml-[0.5px] shrink-0">▾</span>
                )}
              </button>
            </div>
          );
        })}
      </div>

      {openTool?.hasDropdown && openTool.dropdownContent && menuStyle
        ? createPortal(
            <div
              data-insert-tool-dropdown
              style={menuStyle}
              className="bg-[var(--color-kb-editor)] border border-[var(--color-kb-panel-border)] shadow-[0_6px_24px_rgba(0,0,0,0.12)] rounded-xl py-1.5 min-w-[145px] z-[500] animate-in fade-in slide-in-from-top-2 duration-150 before:content-[''] before:absolute before:-top-1.5 before:left-[calc(50%-5px)] before:w-2.5 before:h-2.5 before:bg-[var(--color-kb-editor)] before:rotate-45 before:border-l before:border-t before:border-[var(--color-kb-panel-border)]"
              onMouseDown={(event) => event.stopPropagation()}
              onClick={closeMenu}
            >
              {openTool.dropdownContent}
            </div>,
            document.body,
          )
        : null}
    </>
  );
}

export function DropdownItem({
  onClick,
  icon,
  text,
  disabled,
  highlighted,
}: {
  onClick: () => void;
  icon?: React.ReactNode;
  text: string;
  disabled?: boolean;
  highlighted?: boolean;
}) {
  return (
    <div
      onClick={(event) => {
        if (disabled) {
          event.stopPropagation();
          return;
        }
        onClick();
      }}
      className={`px-4 py-2 text-xs flex items-center gap-2 transition-colors w-full text-left relative z-10 ${
        disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer hover:bg-[var(--color-kb-panel-hover)]'
      } ${
        highlighted ? 'text-[var(--color-kb-accent)] font-bold' : 'text-[var(--color-kb-text)] font-semibold'
      }`}
    >
      {icon}
      <span>{text}</span>
    </div>
  );
}

export function DropdownDivider() {
  return <div className="h-[1px] bg-[var(--color-kb-panel-border)] my-1 relative z-10" />;
}
