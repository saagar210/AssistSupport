import type { ButtonHTMLAttributes, ReactNode } from 'react';
import './asButton.css';

export type AsButtonTone = 'default' | 'primary' | 'ghost' | 'danger';
export type AsButtonSize = 'default' | 'small';

export interface AsButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  tone?: AsButtonTone;
  size?: AsButtonSize;
  icon?: ReactNode;
  iconOnly?: boolean;
}

export function AsButton({
  tone = 'default',
  size = 'default',
  icon,
  iconOnly = false,
  className,
  children,
  ...props
}: AsButtonProps) {
  const classes = [
    'as-btn',
    tone !== 'default' ? `as-btn--${tone}` : '',
    size === 'small' ? 'as-btn--small' : '',
    iconOnly ? 'as-btn--iconOnly' : '',
    className ?? '',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <button type="button" className={classes} {...props}>
      {icon}
      {!iconOnly && children}
    </button>
  );
}

