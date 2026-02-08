import { Network, MessageSquare, FileText, Search, Settings } from "lucide-react";
import { useAppStore } from "../stores/appStore";
import type { ViewType } from "../types";

interface NavItem {
  view: ViewType;
  icon: typeof Network;
  label: string;
}

const NAV_ITEMS: NavItem[] = [
  { view: "graph", icon: Network, label: "Knowledge Graph" },
  { view: "chat", icon: MessageSquare, label: "Chat" },
  { view: "documents", icon: FileText, label: "Documents" },
  { view: "search", icon: Search, label: "Search" },
];

export function Sidebar() {
  const activeView = useAppStore((state) => state.activeView);
  const setActiveView = useAppStore((state) => state.setActiveView);

  return (
    <div className="flex w-14 flex-col items-center border-r border-border bg-sidebar py-3">
      <div className="flex flex-1 flex-col gap-1">
        {NAV_ITEMS.map(({ view, icon: Icon, label }) => {
          const isActive = activeView === view;
          return (
            <button
              key={view}
              title={label}
              onClick={() => setActiveView(view)}
              className={`flex h-10 w-10 items-center justify-center rounded-lg transition-colors ${
                isActive
                  ? "bg-sidebar-active/10 text-sidebar-active"
                  : "text-sidebar-foreground hover:bg-muted hover:text-foreground"
              }`}
            >
              <Icon size={20} />
            </button>
          );
        })}
      </div>

      <button
        title="Settings"
        onClick={() => setActiveView("settings")}
        className={`flex h-10 w-10 items-center justify-center rounded-lg transition-colors ${
          activeView === "settings"
            ? "bg-sidebar-active/10 text-sidebar-active"
            : "text-sidebar-foreground hover:bg-muted hover:text-foreground"
        }`}
      >
        <Settings size={20} />
      </button>
    </div>
  );
}
