import type { ReactNode } from 'react';
import './panel.css';

export interface PanelProps {
  title?: string;
  subtitle?: string;
  children: ReactNode;
  className?: string;
  actions?: ReactNode;
}

export function Panel({ title, subtitle, children, className, actions }: PanelProps) {
  return (
    <section className={['as-panel', className].filter(Boolean).join(' ')}>
      {(title || subtitle || actions) && (
        <header className="as-panel__header">
          <div className="as-panel__titles">
            {title && <h2 className="as-panel__title">{title}</h2>}
            {subtitle && <p className="as-panel__subtitle">{subtitle}</p>}
          </div>
          {actions && <div className="as-panel__actions">{actions}</div>}
        </header>
      )}
      <div className="as-panel__body">{children}</div>
    </section>
  );
}

