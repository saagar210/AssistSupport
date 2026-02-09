import { useEffect, useState } from "react";
import { motion, AnimatePresence } from "framer-motion";
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
import { IngestionPanel } from "./components/IngestionPanel";
import { SetupWizard } from "./components/SetupWizard";
import { OllamaStatusBanner } from "./components/OllamaStatusBanner";
import { LoadingBar } from "./components/LoadingBar";
import { OnboardingTour } from "./components/OnboardingTour";
import { ShortcutCheatsheet } from "./components/ShortcutCheatsheet";
import { GraphView } from "./views/GraphView";
import { ChatView } from "./views/ChatView";
import { DocumentsView } from "./views/DocumentsView";
import { DocumentDetailView } from "./views/DocumentDetailView";
import { SearchView } from "./views/SearchView";
import { SettingsView } from "./views/SettingsView";

function getViewComponent(view: string) {
  switch (view) {
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

function ActiveView() {
  const activeView = useAppStore((state) => state.activeView);

  return (
    <AnimatePresence mode="wait">
      <motion.div
        key={activeView}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        transition={{ duration: 0.15 }}
        className="flex min-w-0 flex-1 overflow-auto"
      >
        {getViewComponent(activeView)}
      </motion.div>
    </AnimatePresence>
  );
}

function App() {
  useTheme();
  useKeyboardShortcuts();

  const [showSetup, setShowSetup] = useState(false);

  const fetchCollections = useCollectionStore(
    (state) => state.fetchCollections,
  );
  const fetchSettings = useSettingsStore((state) => state.fetchSettings);
  const fetchModels = useSettingsStore((state) => state.fetchModels);
  const settings = useSettingsStore((s) => s.settings);

  useEffect(() => {
    fetchCollections();
    fetchSettings();
    fetchModels();
  }, [fetchCollections, fetchSettings, fetchModels]);

  useEffect(() => {
    if (settings.setup_complete !== "true" && Object.keys(settings).length > 0) {
      setShowSetup(true);
    }
  }, [settings]);

  return (
    <>
      <LoadingBar />
      {showSetup && <SetupWizard onComplete={() => setShowSetup(false)} />}
      <div className="flex h-full min-w-0 bg-background text-foreground">
        <Sidebar />
        <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
          <Header />
          <OllamaStatusBanner />
          <ErrorBoundary>
            <main className="flex flex-1 overflow-hidden">
              <ActiveView />
            </main>
          </ErrorBoundary>
          <StatusBar />
        </div>
        <CommandPalette />
        <ToastContainer />
        <IngestionPanel />
        <ShortcutCheatsheet />
        <OnboardingTour />
      </div>
    </>
  );
}

export default App;
