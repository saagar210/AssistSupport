import type { ReactNode } from 'react';
import './emptyState.css';

export interface EmptyStateProps {
  title: string;
  description?: string;
  icon?: ReactNode;
  action?: ReactNode;
}

export function EmptyState({ title, description, icon, action }: EmptyStateProps) {
  return (
    <div className="as-empty">
      {icon && <div className="as-empty__icon">{icon}</div>}
      <div className="as-empty__body">
        <h3 className="as-empty__title">{title}</h3>
        {description && <p className="as-empty__desc">{description}</p>}
        {action && <div className="as-empty__action">{action}</div>}
      </div>
    </div>
  );
}

