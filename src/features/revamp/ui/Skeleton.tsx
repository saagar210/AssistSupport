import './skeleton.css';

export interface SkeletonProps {
  lines?: number;
}

export function Skeleton({ lines = 3 }: SkeletonProps) {
  return (
    <div className="as-skeleton" aria-hidden="true">
      {Array.from({ length: lines }).map((_, idx) => (
        <div key={idx} className="as-skeleton__line" style={{ width: `${92 - idx * 9}%` }} />
      ))}
    </div>
  );
}

