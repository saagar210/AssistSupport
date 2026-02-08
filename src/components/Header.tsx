import { useState } from "react";
import { Settings, Plus } from "lucide-react";
import { useAppStore } from "../stores/appStore";
import { useCollectionStore } from "../stores/collectionStore";
import { Modal, FormField, TextInput } from "./ui";

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

  const [modalOpen, setModalOpen] = useState(false);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [nameError, setNameError] = useState("");

  function openModal() {
    setName("");
    setDescription("");
    setNameError("");
    setModalOpen(true);
  }

  function handleSubmit() {
    if (!name.trim()) {
      setNameError("Collection name is required");
      return;
    }
    createCollection(name.trim(), description.trim());
    setModalOpen(false);
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
          onClick={openModal}
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

      <Modal
        isOpen={modalOpen}
        onClose={() => setModalOpen(false)}
        title="New Collection"
        size="sm"
      >
        <form
          onSubmit={(e) => {
            e.preventDefault();
            handleSubmit();
          }}
          className="flex flex-col gap-4"
        >
          <FormField label="Name" required error={nameError}>
            <TextInput
              placeholder="Collection name"
              value={name}
              onChange={(e) => {
                setName(e.target.value);
                if (nameError) setNameError("");
              }}
              error={!!nameError}
              autoFocus
            />
          </FormField>
          <FormField label="Description" helpText="Optional short description">
            <TextInput
              placeholder="Description (optional)"
              value={description}
              onChange={(e) => setDescription(e.target.value)}
            />
          </FormField>
          <div className="flex justify-end gap-3 pt-2">
            <button
              type="button"
              onClick={() => setModalOpen(false)}
              className="rounded-md px-4 py-2 text-sm font-medium text-muted-foreground hover:bg-muted"
            >
              Cancel
            </button>
            <button
              type="submit"
              className="rounded-md bg-primary px-4 py-2 text-sm font-medium text-white hover:bg-primary/90"
            >
              Create
            </button>
          </div>
        </form>
      </Modal>
    </div>
  );
}
