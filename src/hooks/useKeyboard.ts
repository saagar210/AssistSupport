import { useEffect, useCallback } from 'react';

interface ShortcutHandlers {
  onGenerate?: () => void;
  onFocusSearch?: () => void;
  onSwitchTab?: (tab: number) => void;
  onSaveDraft?: () => void;
  onCopyResponse?: () => void;
  onCancelGeneration?: () => void;
  onExport?: () => void;
}

export function useKeyboardShortcuts(handlers: ShortcutHandlers) {
  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    // Ignore if in input/textarea (unless specific shortcut)
    const target = e.target as HTMLElement;
    const isInput = target.tagName === 'INPUT' || target.tagName === 'TEXTAREA';

    // Cmd/Ctrl + Enter - Generate (works even in textarea)
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
      e.preventDefault();
      handlers.onGenerate?.();
      return;
    }

    // Cmd/Ctrl + S - Save draft (works even in textarea)
    if ((e.metaKey || e.ctrlKey) && e.key === 's') {
      e.preventDefault();
      handlers.onSaveDraft?.();
      return;
    }

    // Cmd/Ctrl + . or Escape - Cancel generation
    if ((e.metaKey || e.ctrlKey) && e.key === '.') {
      e.preventDefault();
      handlers.onCancelGeneration?.();
      return;
    }

    if (e.key === 'Escape') {
      handlers.onCancelGeneration?.();
      return;
    }

    // Skip other shortcuts if in input
    if (isInput) return;

    // Cmd/Ctrl + K - Focus search
    if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
      e.preventDefault();
      handlers.onFocusSearch?.();
      return;
    }

    // Cmd/Ctrl + Shift + C - Copy response
    if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === 'C') {
      e.preventDefault();
      handlers.onCopyResponse?.();
      return;
    }

    // Cmd/Ctrl + E - Export
    if ((e.metaKey || e.ctrlKey) && e.key === 'e') {
      e.preventDefault();
      handlers.onExport?.();
      return;
    }

    // Cmd/Ctrl + 1-6 - Switch tabs
    if ((e.metaKey || e.ctrlKey) && e.key >= '1' && e.key <= '6') {
      e.preventDefault();
      handlers.onSwitchTab?.(parseInt(e.key));
      return;
    }
  }, [handlers]);

  useEffect(() => {
    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);
}
