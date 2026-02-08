import { Settings, Plus } from "lucide-react";
import { useAppStore } from "../stores/appStore";
import { useCollectionStore } from "../stores/collectionStore";

export function Header() {
  const setActiveView = useAppStore((state) => state.setActiveView);
  const collections = useCollectionStore((state) => state.collections);
  const activeCollectionId = useCollectionStore(
    (state) => state.activeCollectionId,
  );
  const setActiveCollection = useCollectionStore(
    (state) => state.setActiveCollection,
  );
  const createCollection = useCollectionStore(
    (state) => state.createCollection,
  );

  function handleNewCollection() {
    const name = window.prompt("Collection name:");
    if (!name?.trim()) return;
    const description =
      window.prompt("Description (optional):") ?? "";
    createCollection(name.trim(), description.trim());
  }

  return (
    <div className="flex h-12 items-center border-b border-border bg-background px-4">
      <div className="flex items-center gap-2">
        <select
          value={activeCollectionId ?? ""}
          onChange={(e) => setActiveCollection(e.target.value)}
          className="h-8 rounded-md border border-border bg-background px-2 text-sm text-foreground outline-none focus:ring-1 focus:ring-accent"
        >
          {collections.length === 0 && (
            <option value="">No collections</option>
          )}
          {collections.map((c) => (
            <option key={c.id} value={c.id}>
              {c.name}
            </option>
          ))}
        </select>

        <button
          title="New Collection"
          onClick={handleNewCollection}
          className="flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
        >
          <Plus size={16} />
        </button>
      </div>

      <div className="flex-1 text-center">
        <span className="text-sm font-semibold text-foreground">VaultMind</span>
      </div>

      <button
        title="Settings"
        onClick={() => setActiveView("settings")}
        className="flex h-8 w-8 items-center justify-center rounded-md text-muted-foreground transition-colors hover:bg-muted hover:text-foreground"
      >
        <Settings size={16} />
      </button>
    </div>
  );
}
