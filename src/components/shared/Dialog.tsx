import { type ReactNode, useEffect, useId, useRef } from 'react';
import './Dialog.css';

const FOCUSABLE_SELECTOR = [
  'button:not([disabled])',
  '[href]',
  'input:not([disabled])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])',
].join(',');

interface DialogProps {
  open: boolean;
  children: ReactNode;
  labelledBy?: string;
  describedBy?: string;
  ariaLabel?: string;
  onClose?: () => void;
  closeOnOverlayClick?: boolean;
  initialFocusRef?: React.RefObject<HTMLElement | null>;
  overlayClassName?: string;
  panelClassName?: string;
}

function getFocusableElements(panel: HTMLElement | null): HTMLElement[] {
  if (!panel) {
    return [];
  }

  return Array.from(panel.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTOR)).filter(
    (element) => !element.hasAttribute('disabled') && element.getAttribute('aria-hidden') !== 'true',
  );
}

export function Dialog({
  open,
  children,
  labelledBy,
  describedBy,
  ariaLabel,
  onClose,
  closeOnOverlayClick = true,
  initialFocusRef,
  overlayClassName = '',
  panelClassName = '',
}: DialogProps) {
  const panelRef = useRef<HTMLDivElement | null>(null);
  const restoreFocusRef = useRef<HTMLElement | null>(null);
  const fallbackLabelId = useId();

  useEffect(() => {
    if (!open) {
      return undefined;
    }

    restoreFocusRef.current = document.activeElement instanceof HTMLElement
      ? document.activeElement
      : null;

    const panel = panelRef.current;
    const focusTarget = initialFocusRef?.current ?? getFocusableElements(panel)[0] ?? panel;
    focusTarget?.focus();

    const handleKeyDown = (event: KeyboardEvent) => {
      if (!panelRef.current) {
        return;
      }

      if (event.key === 'Escape' && onClose) {
        event.preventDefault();
        onClose();
        return;
      }

      if (event.key !== 'Tab') {
        return;
      }

      const focusable = getFocusableElements(panelRef.current);
      if (focusable.length === 0) {
        event.preventDefault();
        panelRef.current.focus();
        return;
      }

      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      const activeElement = document.activeElement;

      if (!event.shiftKey && activeElement === last) {
        event.preventDefault();
        first.focus();
      } else if (event.shiftKey && activeElement === first) {
        event.preventDefault();
        last.focus();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      restoreFocusRef.current?.focus();
    };
  }, [initialFocusRef, onClose, open]);

  if (!open) {
    return null;
  }

  return (
    <div
      className={`dialog-overlay ${overlayClassName}`.trim()}
      role="presentation"
      onMouseDown={(event) => {
        if (closeOnOverlayClick && event.target === event.currentTarget && onClose) {
          onClose();
        }
      }}
    >
      <div
        ref={panelRef}
        className={`dialog-panel ${panelClassName}`.trim()}
        role="dialog"
        aria-modal="true"
        aria-label={ariaLabel}
        aria-labelledby={ariaLabel ? undefined : (labelledBy ?? fallbackLabelId)}
        aria-describedby={describedBy}
        tabIndex={-1}
      >
        {!ariaLabel && !labelledBy && <span id={fallbackLabelId} className="dialog-visually-hidden">Dialog</span>}
        {children}
      </div>
    </div>
  );
}
