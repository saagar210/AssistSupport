interface SkeletonProps {
  className?: string;
  lines?: number;
  width?: string;
}

export function Skeleton({ className = "", lines = 1, width }: SkeletonProps) {
  const style = width ? { width } : undefined;

  if (lines === 1) {
    return (
      <div
        className={`animate-pulse rounded bg-muted ${className}`}
        style={style}
        data-testid="skeleton"
      />
    );
  }

  return (
    <div className="flex flex-col gap-2" data-testid="skeleton">
      {Array.from({ length: lines }).map((_, i) => (
        <div
          key={i}
          className={`animate-pulse rounded bg-muted ${className}`}
          style={i === lines - 1 ? { width: "75%" } : style}
          data-testid="skeleton-line"
        />
      ))}
    </div>
  );
}
