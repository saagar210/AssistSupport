/**
 * KeyboardShortcuts - Help panel showing available keyboard shortcuts
 */

import { useState, useCallback, useEffect } from 'react';
import { Icon } from './Icon';
import './KeyboardShortcuts.css';

interface ShortcutGroup {
  title: string;
  shortcuts: Array<{
    keys: string[];
    description: string;
  }>;
}

const shortcutGroups: ShortcutGroup[] = [
  {
    title: 'Navigation',
    shortcuts: [
      { keys: ['⌘', 'K'], description: 'Open command palette' },
      { keys: ['⌘', '1'], description: 'Go to Draft' },
      { keys: ['⌘', '2'], description: 'Go to Follow-ups' },
      { keys: ['⌘', '3'], description: 'Go to Sources' },
      { keys: ['⌘', '4'], description: 'Go to Ingest' },
      { keys: ['⌘', '5'], description: 'Go to Knowledge' },
      { keys: ['⌘', '6'], description: 'Go to Analytics' },
      { keys: ['⌘', '7'], description: 'Go to Settings' },
    ],
  },
  {
    title: 'Draft Actions',
    shortcuts: [
      { keys: ['⌘', '↵'], description: 'Generate response' },
      { keys: ['⌘', 'S'], description: 'Save draft' },
      { keys: ['⌘', '⇧', 'C'], description: 'Copy response' },
      { keys: ['⌘', 'E'], description: 'Export response' },
      { keys: ['⌘', '.'], description: 'Cancel generation' },
      { keys: ['Esc'], description: 'Cancel / Close' },
    ],
  },
  {
    title: 'Features',
    shortcuts: [
      { keys: ['⌘', 'K'], description: 'Templates (via command palette)' },
      { keys: ['⌘', 'K'], description: 'Batch processing (via command palette)' },
      { keys: ['⌘', 'K'], description: 'Voice input (via command palette)' },
    ],
  },
  {
    title: 'General',
    shortcuts: [
      { keys: ['⌘', '?'], description: 'Show keyboard shortcuts' },
      { keys: ['↑', '↓'], description: 'Navigate lists' },
      { keys: ['↵'], description: 'Select item' },
    ],
  },
];

interface KeyboardShortcutsProps {
  isOpen: boolean;
  onClose: () => void;
}

export function KeyboardShortcuts({ isOpen, onClose }: KeyboardShortcutsProps) {
  // Close on Escape
  useEffect(() => {
    if (!isOpen) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        e.preventDefault();
        onClose();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [isOpen, onClose]);

  // Handle click outside
  const handleBackdropClick = useCallback((e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }, [onClose]);

  if (!isOpen) return null;

  return (
    <div className="shortcuts-overlay" onClick={handleBackdropClick}>
      <div className="shortcuts-panel" role="dialog" aria-label="Keyboard Shortcuts">
        <div className="shortcuts-header">
          <h2>Keyboard Shortcuts</h2>
          <button className="shortcuts-close" onClick={onClose} aria-label="Close">
            <Icon name="x" size={18} />
          </button>
        </div>

        <div className="shortcuts-body">
          {shortcutGroups.map(group => (
            <div key={group.title} className="shortcuts-group">
              <h3>{group.title}</h3>
              <div className="shortcuts-list">
                {group.shortcuts.map((shortcut, idx) => (
                  <div key={idx} className="shortcut-item">
                    <span className="shortcut-description">{shortcut.description}</span>
                    <div className="shortcut-keys">
                      {shortcut.keys.map((key, keyIdx) => (
                        <kbd key={keyIdx}>{key}</kbd>
                      ))}
                    </div>
                  </div>
                ))}
              </div>
            </div>
          ))}
        </div>

        <div className="shortcuts-footer">
          <p>Press <kbd>⌘</kbd><kbd>K</kbd> to open the command palette for more actions</p>
        </div>
      </div>
    </div>
  );
}

// Hook for managing keyboard shortcuts help
export function useKeyboardShortcutsHelp() {
  const [isOpen, setIsOpen] = useState(false);

  const open = useCallback(() => setIsOpen(true), []);
  const close = useCallback(() => setIsOpen(false), []);
  const toggle = useCallback(() => setIsOpen(prev => !prev), []);

  // Global Cmd+? listener
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.shiftKey && e.key === '/') {
        e.preventDefault();
        toggle();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [toggle]);

  return { isOpen, open, close, toggle };
}
