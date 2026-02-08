interface SkeletonBlockProps {
  className?: string;
}

function SkeletonBlock({ className = "" }: SkeletonBlockProps) {
  return (
    <div
      className={`animate-pulse rounded bg-muted ${className}`}
      data-testid="skeleton-block"
    />
  );
}

export function ChatSkeleton() {
  return (
    <div className="flex flex-1 flex-col gap-4 p-6" data-testid="chat-skeleton">
      {/* Assistant message */}
      <div className="flex gap-3 self-start max-w-[70%]">
        <SkeletonBlock className="h-8 w-8 shrink-0 rounded-full" />
        <div className="flex flex-col gap-2">
          <SkeletonBlock className="h-4 w-64" />
          <SkeletonBlock className="h-4 w-48" />
          <SkeletonBlock className="h-4 w-56" />
        </div>
      </div>
      {/* User message */}
      <div className="flex gap-3 self-end max-w-[70%]">
        <div className="flex flex-col gap-2 items-end">
          <SkeletonBlock className="h-4 w-40" />
          <SkeletonBlock className="h-4 w-32" />
        </div>
      </div>
      {/* Assistant message */}
      <div className="flex gap-3 self-start max-w-[70%]">
        <SkeletonBlock className="h-8 w-8 shrink-0 rounded-full" />
        <div className="flex flex-col gap-2">
          <SkeletonBlock className="h-4 w-72" />
          <SkeletonBlock className="h-4 w-52" />
          <SkeletonBlock className="h-4 w-60" />
          <SkeletonBlock className="h-4 w-44" />
        </div>
      </div>
    </div>
  );
}

export function DocumentGridSkeleton() {
  return (
    <div
      className="grid grid-cols-1 gap-4 p-6 sm:grid-cols-2 lg:grid-cols-3"
      data-testid="document-grid-skeleton"
    >
      {Array.from({ length: 6 }).map((_, i) => (
        <div key={i} className="flex flex-col gap-3 rounded-lg border border-border p-4">
          <SkeletonBlock className="h-5 w-3/4" />
          <SkeletonBlock className="h-4 w-1/2" />
          <SkeletonBlock className="h-3 w-1/3" />
        </div>
      ))}
    </div>
  );
}

export function DocumentListSkeleton() {
  return (
    <div className="flex flex-col gap-2 p-6" data-testid="document-list-skeleton">
      {Array.from({ length: 8 }).map((_, i) => (
        <div key={i} className="flex items-center gap-4 rounded-lg border border-border p-3">
          <SkeletonBlock className="h-8 w-8 rounded" />
          <div className="flex flex-1 flex-col gap-1">
            <SkeletonBlock className="h-4 w-2/3" />
            <SkeletonBlock className="h-3 w-1/3" />
          </div>
        </div>
      ))}
    </div>
  );
}

export function SearchSkeleton() {
  return (
    <div className="flex flex-col gap-3 p-6" data-testid="search-skeleton">
      {Array.from({ length: 5 }).map((_, i) => (
        <div key={i} className="flex flex-col gap-2 rounded-lg border border-border p-4">
          <SkeletonBlock className="h-5 w-1/2" />
          <SkeletonBlock className="h-4 w-full" />
          <SkeletonBlock className="h-4 w-3/4" />
          <SkeletonBlock className="h-3 w-1/4" />
        </div>
      ))}
    </div>
  );
}

export function GraphSkeleton() {
  return (
    <div
      className="flex flex-1 items-center justify-center"
      data-testid="graph-skeleton"
    >
      <div className="flex flex-col items-center gap-3">
        <div className="h-10 w-10 animate-spin rounded-full border-2 border-muted border-t-primary" />
        <p className="text-sm text-muted-foreground">Building graph...</p>
      </div>
    </div>
  );
}
