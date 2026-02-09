import { useState, useRef, useEffect, useCallback } from "react";
import { HelpCircle, X } from "lucide-react";
import { helpContent } from "../utils/helpContent";

interface ContextualHelpProps {
  topic: string;
  placement?: "top" | "bottom" | "left" | "right";
}

const PLACEMENT_CLASSES: Record<string, string> = {
  top: "bottom-full left-1/2 -translate-x-1/2 mb-2",
  bottom: "top-full left-1/2 -translate-x-1/2 mt-2",
  left: "right-full top-1/2 -translate-y-1/2 mr-2",
  right: "left-full top-1/2 -translate-y-1/2 ml-2",
};

const ARROW_CLASSES: Record<string, string> = {
  top: "top-full left-1/2 -translate-x-1/2 border-l-transparent border-r-transparent border-b-transparent border-t-border",
  bottom:
    "bottom-full left-1/2 -translate-x-1/2 border-l-transparent border-r-transparent border-t-transparent border-b-border",
  left: "left-full top-1/2 -translate-y-1/2 border-t-transparent border-b-transparent border-r-transparent border-l-border",
  right:
    "right-full top-1/2 -translate-y-1/2 border-t-transparent border-b-transparent border-l-transparent border-r-border",
};

export function ContextualHelp({ topic, placement = "bottom" }: ContextualHelpProps) {
  const [open, setOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const content = helpContent[topic];

  const handleClickOutside = useCallback(
    (e: MouseEvent) => {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setOpen(false);
      }
    },
    [],
  );

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setOpen(false);
      }
    },
    [],
  );

  useEffect(() => {
    if (!open) return;

    document.addEventListener("mousedown", handleClickOutside);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [open, handleClickOutside, handleKeyDown]);

  if (!content) return null;

  return (
    <div className="relative inline-block" ref={containerRef}>
      <button
        onClick={() => setOpen(!open)}
        className="flex h-5 w-5 items-center justify-center rounded-full text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
        aria-label={`Help: ${content.title}`}
        data-testid={`help-trigger-${topic}`}
      >
        <HelpCircle size={14} />
      </button>

      {open && (
        <div
          className={`absolute z-50 w-64 ${PLACEMENT_CLASSES[placement]}`}
          data-testid={`help-popover-${topic}`}
        >
          <div className="rounded-lg border border-border bg-background p-3 shadow-lg">
            <div className="mb-1.5 flex items-center justify-between">
              <h4 className="text-xs font-semibold text-foreground">{content.title}</h4>
              <button
                onClick={() => setOpen(false)}
                className="flex h-4 w-4 items-center justify-center rounded text-muted-foreground hover:text-foreground"
                aria-label="Close help"
              >
                <X size={12} />
              </button>
            </div>
            <p className="text-xs leading-relaxed text-muted-foreground">
              {content.description}
            </p>
          </div>
          <div
            className={`absolute h-0 w-0 border-4 ${ARROW_CLASSES[placement]}`}
          />
        </div>
      )}
    </div>
  );
}
