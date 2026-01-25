/**
 * CommandPalette - Quick action command palette (Cmd+K)
 * Provides keyboard-first navigation and search for app actions
 */

import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { Icon, IconName } from './Icon';
import './CommandPalette.css';

export interface Command {
  id: string;
  label: string;
  description?: string;
  icon: IconName;
  shortcut?: string;
  category: 'navigation' | 'action' | 'draft' | 'settings';
  action: () => void;
  disabled?: boolean;
}

interface CommandPaletteProps {
  isOpen: boolean;
  onClose: () => void;
  commands: Command[];
}

export function CommandPalette({ isOpen, onClose, commands }: CommandPaletteProps) {
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  // Filter commands based on search query
  const filteredCommands = useMemo(() => {
    if (!query.trim()) return commands;
    const lowerQuery = query.toLowerCase();
    return commands.filter(cmd =>
      cmd.label.toLowerCase().includes(lowerQuery) ||
      cmd.description?.toLowerCase().includes(lowerQuery) ||
      cmd.category.toLowerCase().includes(lowerQuery)
    );
  }, [commands, query]);

  // Group commands by category
  const groupedCommands = useMemo(() => {
    const groups: Record<string, Command[]> = {};
    for (const cmd of filteredCommands) {
      if (!groups[cmd.category]) {
        groups[cmd.category] = [];
      }
      groups[cmd.category].push(cmd);
    }
    return groups;
  }, [filteredCommands]);

  // Reset state when opening
  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIndex(0);
      // Focus input after a brief delay for animation
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  // Scroll selected item into view
  useEffect(() => {
    const selected = listRef.current?.querySelector('[data-selected="true"]');
    selected?.scrollIntoView({ block: 'nearest' });
  }, [selectedIndex]);

  // Handle keyboard navigation
  const handleKeyDown = useCallback((e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedIndex(prev =>
          prev < filteredCommands.length - 1 ? prev + 1 : 0
        );
        break;
      case 'ArrowUp':
        e.preventDefault();
        setSelectedIndex(prev =>
          prev > 0 ? prev - 1 : filteredCommands.length - 1
        );
        break;
      case 'Enter':
        e.preventDefault();
        if (filteredCommands[selectedIndex] && !filteredCommands[selectedIndex].disabled) {
          filteredCommands[selectedIndex].action();
          onClose();
        }
        break;
      case 'Escape':
        e.preventDefault();
        onClose();
        break;
    }
  }, [filteredCommands, selectedIndex, onClose]);

  // Execute command
  const executeCommand = useCallback((cmd: Command) => {
    if (cmd.disabled) return;
    cmd.action();
    onClose();
  }, [onClose]);

  // Handle click outside
  const handleBackdropClick = useCallback((e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      onClose();
    }
  }, [onClose]);

  if (!isOpen) return null;

  const categoryLabels: Record<string, string> = {
    navigation: 'Navigation',
    action: 'Actions',
    draft: 'Draft',
    settings: 'Settings',
  };

  let flatIndex = 0;

  return (
    <div className="command-palette-overlay" onClick={handleBackdropClick}>
      <div className="command-palette" role="dialog" aria-label="Command Palette">
        <div className="command-palette-header">
          <Icon name="search" size={18} className="command-search-icon" />
          <input
            ref={inputRef}
            type="text"
            className="command-input"
            placeholder="Type a command or search..."
            value={query}
            onChange={e => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleKeyDown}
            aria-label="Search commands"
          />
          <div className="command-shortcut-hint">
            <kbd>Esc</kbd>
            <span>to close</span>
          </div>
        </div>

        <div className="command-palette-body" ref={listRef}>
          {filteredCommands.length === 0 ? (
            <div className="command-empty">
              <p>No commands found for "{query}"</p>
            </div>
          ) : (
            Object.entries(groupedCommands).map(([category, cmds]) => (
              <div key={category} className="command-group">
                <div className="command-group-label">{categoryLabels[category]}</div>
                {cmds.map(cmd => {
                  const isSelected = flatIndex === selectedIndex;
                  const currentIndex = flatIndex++;
                  return (
                    <button
                      key={cmd.id}
                      className={`command-item ${isSelected ? 'selected' : ''} ${cmd.disabled ? 'disabled' : ''}`}
                      onClick={() => executeCommand(cmd)}
                      onMouseEnter={() => setSelectedIndex(currentIndex)}
                      data-selected={isSelected}
                      disabled={cmd.disabled}
                    >
                      <Icon name={cmd.icon} size={18} className="command-item-icon" />
                      <div className="command-item-content">
                        <span className="command-item-label">{cmd.label}</span>
                        {cmd.description && (
                          <span className="command-item-description">{cmd.description}</span>
                        )}
                      </div>
                      {cmd.shortcut && (
                        <div className="command-item-shortcut">
                          {cmd.shortcut.split('+').map((key, i) => (
                            <kbd key={i}>{key === 'Cmd' ? '\u2318' : key === 'Shift' ? '\u21E7' : key}</kbd>
                          ))}
                        </div>
                      )}
                    </button>
                  );
                })}
              </div>
            ))
          )}
        </div>

        <div className="command-palette-footer">
          <div className="command-hint">
            <kbd>\u2191</kbd><kbd>\u2193</kbd>
            <span>to navigate</span>
          </div>
          <div className="command-hint">
            <kbd>\u21B5</kbd>
            <span>to select</span>
          </div>
        </div>
      </div>
    </div>
  );
}

// Hook for managing command palette state and commands
export function useCommandPalette() {
  const [isOpen, setIsOpen] = useState(false);

  const open = useCallback(() => setIsOpen(true), []);
  const close = useCallback(() => setIsOpen(false), []);
  const toggle = useCallback(() => setIsOpen(prev => !prev), []);

  // Global Cmd+K listener
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault();
        toggle();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [toggle]);

  return { isOpen, open, close, toggle };
}
