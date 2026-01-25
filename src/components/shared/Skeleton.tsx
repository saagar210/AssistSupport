import './Skeleton.css';

interface SkeletonProps {
  width?: string;
  height?: string;
  variant?: 'text' | 'rectangular' | 'circular';
  className?: string;
}

export function Skeleton({
  width = '100%',
  height = '1em',
  variant = 'text',
  className = ''
}: SkeletonProps) {
  return (
    <div
      className={`skeleton skeleton-${variant} ${className}`}
      style={{ width, height }}
    />
  );
}

interface SkeletonTextProps {
  lines?: number;
}

export function SkeletonText({ lines = 3 }: SkeletonTextProps) {
  return (
    <div className="skeleton-text">
      {Array.from({ length: lines }).map((_, i) => (
        <Skeleton
          key={i}
          width={i === lines - 1 ? '60%' : '100%'}
          height="1em"
        />
      ))}
    </div>
  );
}
