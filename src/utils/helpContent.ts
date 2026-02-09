export interface HelpTopic {
  title: string;
  description: string;
}

export const helpContent: Record<string, HelpTopic> = {
  collections: {
    title: "Collections",
    description:
      "Collections are containers for organizing your documents. Create a collection for each project, topic, or area of study. All documents, searches, and chats happen within a collection context.",
  },
  documents: {
    title: "Documents",
    description:
      "Import documents by dragging files or clicking Add Documents. Supported formats include PDF, Markdown, HTML, TXT, DOCX, CSV, and EPUB. Documents are automatically chunked and embedded for semantic search.",
  },
  search: {
    title: "Search",
    description:
      "Search across your documents using three modes: Hybrid (best overall), Semantic (meaning-based), and Keyword (exact matches). Use filters to narrow results by file type. Click 'More like this' to find similar content.",
  },
  chat: {
    title: "Chat",
    description:
      "Ask questions about your documents in natural language. The AI retrieves relevant passages and generates answers with citations. Use Cmd+Enter to send. Click assistant messages to view source citations in the sidebar.",
  },
  graph: {
    title: "Knowledge Graph",
    description:
      "Visualize connections between your documents. Nodes represent documents and edges show semantic similarity. Use filters to focus on specific file types. Right-click nodes for actions like finding paths between documents.",
  },
  settings: {
    title: "Settings",
    description:
      "Configure your Ollama model preferences, embedding settings, and application behavior. Ensure Ollama is running locally for chat and embedding features to work.",
  },
  embeddings: {
    title: "Embeddings",
    description:
      "Embeddings convert document text into numerical vectors that capture meaning. VaultMind uses these to power semantic search and find connections between documents. They are generated automatically during ingestion.",
  },
  models: {
    title: "Models",
    description:
      "VaultMind uses Ollama to run AI models locally. Chat models generate responses, while embedding models create document vectors. Install models through Ollama CLI (e.g., 'ollama pull nomic-embed-text').",
  },
};
