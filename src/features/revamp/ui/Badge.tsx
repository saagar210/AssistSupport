import type { ReactNode } from 'react';
import './badge.css';

export type BadgeTone = 'neutral' | 'good' | 'warn' | 'bad' | 'info';

export interface BadgeProps {
  tone?: BadgeTone;
  children: ReactNode;
  className?: string;
}

export function Badge({ tone = 'neutral', children, className }: BadgeProps) {
  return <span className={['as-badge', `as-badge--${tone}`, className].filter(Boolean).join(' ')}>{children}</span>;
}

