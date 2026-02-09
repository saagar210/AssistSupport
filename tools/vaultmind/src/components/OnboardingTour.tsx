import { useState, useEffect, useCallback } from "react";
import { createPortal } from "react-dom";
import {
  Library,
  FileText,
  Search,
  MessageSquare,
  Sparkles,
  ChevronRight,
  X,
} from "lucide-react";
import { useSettingsStore } from "../stores/settingsStore";

interface TourStep {
  id: string;
  title: string;
  description: string;
  icon: React.ElementType;
}

const TOUR_STEPS: TourStep[] = [
  {
    id: "welcome",
    title: "Welcome to VaultMind",
    description:
      "Your personal knowledge management system. Import documents, search by meaning, chat with your knowledge base, and visualize connections. Let's take a quick tour.",
    icon: Sparkles,
  },
  {
    id: "collection",
    title: "Create a Collection",
    description:
      "Start by creating a collection in the sidebar. Collections group related documents together — think of them as folders for a project or topic.",
    icon: Library,
  },
  {
    id: "documents",
    title: "Add Documents",
    description:
      "Import documents by dragging files into the Documents view or clicking Add Documents. VaultMind supports PDF, Markdown, HTML, TXT, DOCX, CSV, and EPUB files.",
    icon: FileText,
  },
  {
    id: "search",
    title: "Search Your Knowledge",
    description:
      "Use semantic search to find information by meaning, not just keywords. Try Hybrid mode for the best results. Press Cmd+Shift+F to jump to search anytime.",
    icon: Search,
  },
  {
    id: "chat",
    title: "Chat With Your Documents",
    description:
      "Ask questions in natural language and get AI-generated answers with citations from your documents. The AI runs locally through Ollama — your data stays private.",
    icon: MessageSquare,
  },
];

export function OnboardingTour() {
  const [currentStep, setCurrentStep] = useState(0);
  const [visible, setVisible] = useState(false);

  const settings = useSettingsStore((s) => s.settings);
  const updateSetting = useSettingsStore((s) => s.updateSetting);

  useEffect(() => {
    // Only show if settings are loaded and onboarding hasn't been completed
    if (Object.keys(settings).length > 0 && settings.onboarding_completed !== "true") {
      setVisible(true);
    }
  }, [settings]);

  const handleNext = useCallback(() => {
    if (currentStep < TOUR_STEPS.length - 1) {
      setCurrentStep((prev) => prev + 1);
    } else {
      // Tour complete
      updateSetting("onboarding_completed", "true");
      setVisible(false);
    }
  }, [currentStep, updateSetting]);

  const handleSkip = useCallback(() => {
    updateSetting("onboarding_completed", "true");
    setVisible(false);
  }, [updateSetting]);

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        handleSkip();
      } else if (e.key === "ArrowRight" || e.key === "Enter") {
        handleNext();
      }
    },
    [handleSkip, handleNext],
  );

  useEffect(() => {
    if (!visible) return;

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [visible, handleKeyDown]);

  if (!visible) return null;

  const step = TOUR_STEPS[currentStep];
  const StepIcon = step.icon;
  const isLastStep = currentStep === TOUR_STEPS.length - 1;

  return createPortal(
    <div
      className="fixed inset-0 z-[60] flex items-center justify-center bg-black/60"
      data-testid="onboarding-overlay"
    >
      <div
        className="relative w-full max-w-md rounded-xl border border-border bg-background p-6 shadow-2xl"
        role="dialog"
        aria-modal="true"
        aria-label={`Onboarding step ${currentStep + 1} of ${TOUR_STEPS.length}: ${step.title}`}
        data-testid="onboarding-dialog"
      >
        {/* Close / Skip */}
        <button
          onClick={handleSkip}
          className="absolute right-3 top-3 flex h-7 w-7 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
          aria-label="Skip tour"
          data-testid="onboarding-skip"
        >
          <X size={16} />
        </button>

        {/* Progress indicator */}
        <div className="mb-5 flex items-center gap-1.5" data-testid="onboarding-progress">
          {TOUR_STEPS.map((_, idx) => (
            <div
              key={idx}
              className={`h-1 flex-1 rounded-full transition-colors ${
                idx <= currentStep ? "bg-accent" : "bg-muted"
              }`}
            />
          ))}
        </div>

        {/* Icon */}
        <div className="mb-4 flex h-12 w-12 items-center justify-center rounded-xl bg-accent/10 text-accent">
          <StepIcon size={24} />
        </div>

        {/* Content */}
        <h2 className="mb-2 text-lg font-semibold text-foreground">{step.title}</h2>
        <p className="mb-6 text-sm leading-relaxed text-muted-foreground">
          {step.description}
        </p>

        {/* Actions */}
        <div className="flex items-center justify-between">
          <span className="text-xs text-muted-foreground">
            {currentStep + 1} of {TOUR_STEPS.length}
          </span>
          <div className="flex items-center gap-2">
            {!isLastStep && (
              <button
                onClick={handleSkip}
                className="rounded-md px-3 py-1.5 text-xs text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
                data-testid="onboarding-skip-btn"
              >
                Skip Tour
              </button>
            )}
            <button
              onClick={handleNext}
              className="flex items-center gap-1.5 rounded-md bg-accent px-4 py-1.5 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90"
              data-testid="onboarding-next"
            >
              {isLastStep ? "Get Started" : "Next"}
              {!isLastStep && <ChevronRight size={14} />}
            </button>
          </div>
        </div>
      </div>
    </div>,
    document.body,
  );
}
