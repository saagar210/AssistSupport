import { useAppStore } from "../stores/appStore";

export function LoadingBar() {
  const loading = useAppStore((s) => s.globalLoadingCount > 0);

  if (!loading) return null;

  return (
    <div className="fixed left-0 right-0 top-0 z-50 h-0.5 overflow-hidden bg-accent/20">
      <div
        className="h-full w-1/3 bg-accent"
        style={{
          animation: "loading-bar 1.5s ease-in-out infinite",
        }}
      />
    </div>
  );
}
