import { useEffect } from "react";
import { useAppStore } from "./stores/appStore";
import { useCollectionStore } from "./stores/collectionStore";
import { useSettingsStore } from "./stores/settingsStore";
import { useTheme } from "./hooks/useTheme";
import { useKeyboardShortcuts } from "./hooks/useKeyboardShortcuts";
import { Sidebar } from "./components/Sidebar";
import { Header } from "./components/Header";
import { StatusBar } from "./components/StatusBar";
import { CommandPalette } from "./components/CommandPalette";
import { ToastContainer } from "./components/Toast";
import { ErrorBoundary } from "./components/ErrorBoundary";
import { GraphView } from "./views/GraphView";
import { ChatView } from "./views/ChatView";
import { DocumentsView } from "./views/DocumentsView";
import { DocumentDetailView } from "./views/DocumentDetailView";
import { SearchView } from "./views/SearchView";
import { SettingsView } from "./views/SettingsView";

function ActiveView() {
  const activeView = useAppStore((state) => state.activeView);

  switch (activeView) {
    case "graph":
      return <GraphView />;
    case "chat":
      return <ChatView />;
    case "documents":
      return <DocumentsView />;
    case "document-detail":
      return <DocumentDetailView />;
    case "search":
      return <SearchView />;
    case "settings":
      return <SettingsView />;
    default:
      return <DocumentsView />;
  }
}

function App() {
  useTheme();
  useKeyboardShortcuts();

  const fetchCollections = useCollectionStore(
    (state) => state.fetchCollections,
  );
  const fetchSettings = useSettingsStore((state) => state.fetchSettings);

  useEffect(() => {
    fetchCollections();
    fetchSettings();
  }, [fetchCollections, fetchSettings]);

  return (
    <div className="flex h-full bg-background text-foreground">
      <Sidebar />
      <div className="flex flex-1 flex-col overflow-hidden">
        <Header />
        <ErrorBoundary>
          <main className="flex flex-1 overflow-auto">
            <ActiveView />
          </main>
        </ErrorBoundary>
        <StatusBar />
      </div>
      <CommandPalette />
      <ToastContainer />
    </div>
  );
}

export default App;
