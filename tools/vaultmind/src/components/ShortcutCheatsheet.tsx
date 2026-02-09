import { useAppStore } from "../stores/appStore";
import { Modal } from "./ui/Modal";

interface ShortcutEntry {
  keys: string;
  description: string;
}

const SHORTCUT_SECTIONS: { title: string; shortcuts: ShortcutEntry[] }[] = [
  {
    title: "Navigation",
    shortcuts: [
      { keys: "Cmd+1", description: "Knowledge Graph view" },
      { keys: "Cmd+2", description: "Chat view" },
      { keys: "Cmd+3", description: "Documents view" },
      { keys: "Cmd+4", description: "Search view" },
      { keys: "Cmd+,", description: "Settings" },
      { keys: "Cmd+Shift+F", description: "Jump to search" },
    ],
  },
  {
    title: "Actions",
    shortcuts: [
      { keys: "Cmd+K", description: "Command palette" },
      { keys: "Cmd+O", description: "Import files" },
      { keys: "Cmd+N", description: "New conversation (in chat)" },
      { keys: "Cmd+Enter", description: "Send message (in chat)" },
    ],
  },
  {
    title: "General",
    shortcuts: [
      { keys: "?", description: "Show this cheatsheet" },
      { keys: "Esc", description: "Close dialog / cancel" },
    ],
  },
];

export function ShortcutCheatsheet() {
  const open = useAppStore((s) => s.shortcutCheatsheetOpen);
  const toggle = useAppStore((s) => s.toggleShortcutCheatsheet);

  return (
    <Modal isOpen={open} onClose={toggle} title="Keyboard Shortcuts" size="md">
      <div className="space-y-4" data-testid="shortcut-cheatsheet">
        {SHORTCUT_SECTIONS.map((section) => (
          <div key={section.title}>
            <h4 className="mb-2 text-xs font-semibold uppercase text-muted-foreground">
              {section.title}
            </h4>
            <div className="space-y-1">
              {section.shortcuts.map((shortcut) => (
                <div
                  key={shortcut.keys}
                  className="flex items-center justify-between rounded px-2 py-1 text-sm hover:bg-muted/50"
                >
                  <span className="text-foreground">{shortcut.description}</span>
                  <kbd className="rounded border border-border bg-muted px-1.5 py-0.5 text-xs font-mono text-muted-foreground">
                    {shortcut.keys}
                  </kbd>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </Modal>
  );
}
