import { useState, useMemo } from "react";
import {
  X,
  Route,
  BarChart3,
  Users,
  FileText,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import type { Community, GraphStats, GraphNode } from "../types";

interface ProcessedNode extends GraphNode {
  x?: number;
  y?: number;
  color: string;
}

/** Palette used for community coloring (cycles for >10 communities). */
export const COMMUNITY_COLORS = [
  "#3b82f6", // blue
  "#ef4444", // red
  "#22c55e", // green
  "#f59e0b", // amber
  "#8b5cf6", // violet
  "#ec4899", // pink
  "#06b6d4", // cyan
  "#f97316", // orange
  "#14b8a6", // teal
  "#a855f7", // purple
] as const;

export function communityColor(communityId: number): string {
  return COMMUNITY_COLORS[communityId % COMMUNITY_COLORS.length];
}

interface GraphSidebarProps {
  selectedNode: ProcessedNode | null;
  communities: Community[];
  graphStats: GraphStats;
  connectionCount: number;
  highlightedPath: string[];
  onFindPath: (fromId: string, toId: string) => void;
  onClose: () => void;
  onViewDocument: (nodeId: string) => void;
  allNodes: ProcessedNode[];
}

export function GraphSidebar({
  selectedNode,
  communities,
  graphStats,
  connectionCount,
  highlightedPath,
  onFindPath,
  onClose,
  onViewDocument,
  allNodes,
}: GraphSidebarProps) {
  const [communitiesOpen, setCommunitiesOpen] = useState(true);
  const [pathTargetOpen, setPathTargetOpen] = useState(false);
  const [pathSearch, setPathSearch] = useState("");

  const filteredTargets = useMemo(() => {
    if (!pathSearch.trim()) return allNodes.slice(0, 20);
    const lower = pathSearch.toLowerCase();
    return allNodes
      .filter(
        (n) =>
          n.label.toLowerCase().includes(lower) &&
          n.id !== selectedNode?.id,
      )
      .slice(0, 20);
  }, [allNodes, pathSearch, selectedNode?.id]);

  return (
    <div className="flex h-full w-72 flex-col border-l border-border bg-background/95 backdrop-blur">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border px-4 py-3">
        <h2 className="text-sm font-semibold text-foreground">Graph Details</h2>
        <button
          onClick={onClose}
          className="text-muted-foreground transition-colors hover:text-foreground"
        >
          <X size={14} />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto">
        {/* Graph Stats */}
        <div className="border-b border-border px-4 py-3">
          <div className="mb-2 flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
            <BarChart3 size={12} />
            <span>Statistics</span>
          </div>
          <div className="grid grid-cols-3 gap-2">
            <div className="rounded-md bg-muted/50 p-2 text-center">
              <p className="text-sm font-semibold text-foreground">
                {graphStats.nodeCount}
              </p>
              <p className="text-[10px] text-muted-foreground">Nodes</p>
            </div>
            <div className="rounded-md bg-muted/50 p-2 text-center">
              <p className="text-sm font-semibold text-foreground">
                {graphStats.edgeCount}
              </p>
              <p className="text-[10px] text-muted-foreground">Edges</p>
            </div>
            <div className="rounded-md bg-muted/50 p-2 text-center">
              <p className="text-sm font-semibold text-foreground">
                {graphStats.density.toFixed(2)}
              </p>
              <p className="text-[10px] text-muted-foreground">Density</p>
            </div>
          </div>
        </div>

        {/* Selected Node Details */}
        {selectedNode && (
          <div className="border-b border-border px-4 py-3">
            <div className="mb-2 flex items-center gap-1.5 text-xs font-medium text-muted-foreground">
              <FileText size={12} />
              <span>Selected Node</span>
            </div>
            <h3 className="mb-2 truncate text-sm font-medium text-foreground">
              {selectedNode.label}
            </h3>
            <div className="space-y-1.5 text-xs">
              <div className="flex justify-between text-muted-foreground">
                <span>Type</span>
                <span
                  className="rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase text-white"
                  style={{ backgroundColor: selectedNode.color }}
                >
                  {selectedNode.file_type}
                </span>
              </div>
              <div className="flex justify-between text-muted-foreground">
                <span>Connections</span>
                <span className="text-foreground">{connectionCount}</span>
              </div>
              <div className="flex justify-between text-muted-foreground">
                <span>Chunks</span>
                <span className="text-foreground">
                  {selectedNode.chunk_count}
                </span>
              </div>
              <div className="flex justify-between text-muted-foreground">
                <span>Words</span>
                <span className="text-foreground">
                  {selectedNode.word_count.toLocaleString()}
                </span>
              </div>
            </div>

            <div className="mt-3 flex gap-2">
              <button
                onClick={() => onViewDocument(selectedNode.id)}
                className="flex-1 rounded-md bg-accent px-3 py-1.5 text-xs font-medium text-accent-foreground transition-colors hover:bg-accent/90"
              >
                View Document
              </button>
              <button
                onClick={() => setPathTargetOpen(!pathTargetOpen)}
                className="flex items-center gap-1 rounded-md border border-border px-3 py-1.5 text-xs font-medium text-foreground transition-colors hover:bg-muted"
              >
                <Route size={12} />
                Find Path
              </button>
            </div>

            {/* Path target picker */}
            {pathTargetOpen && (
              <div className="mt-2 rounded-md border border-border bg-muted/30 p-2">
                <input
                  type="text"
                  value={pathSearch}
                  onChange={(e) => setPathSearch(e.target.value)}
                  placeholder="Search target node..."
                  className="mb-2 w-full rounded border border-border bg-background px-2 py-1 text-xs text-foreground outline-none focus:ring-1 focus:ring-accent"
                />
                <div className="max-h-32 space-y-0.5 overflow-y-auto">
                  {filteredTargets.map((node) => (
                    <button
                      key={node.id}
                      onClick={() => {
                        onFindPath(selectedNode.id, node.id);
                        setPathTargetOpen(false);
                        setPathSearch("");
                      }}
                      className="flex w-full items-center gap-2 rounded px-2 py-1 text-left text-xs text-foreground hover:bg-muted"
                    >
                      <span
                        className="inline-block h-2 w-2 flex-shrink-0 rounded-full"
                        style={{ backgroundColor: node.color }}
                      />
                      <span className="truncate">{node.label}</span>
                    </button>
                  ))}
                  {filteredTargets.length === 0 && (
                    <p className="px-2 py-1 text-[10px] text-muted-foreground">
                      No matching nodes
                    </p>
                  )}
                </div>
              </div>
            )}

            {/* Path display */}
            {highlightedPath.length > 0 && (
              <div className="mt-2 rounded-md border border-accent/30 bg-accent/5 p-2">
                <p className="mb-1 text-[10px] font-medium text-accent">
                  Path ({highlightedPath.length} nodes)
                </p>
                <div className="space-y-0.5">
                  {highlightedPath.map((nodeId, idx) => {
                    const node = allNodes.find((n) => n.id === nodeId);
                    return (
                      <div
                        key={nodeId}
                        className="flex items-center gap-1 text-[10px] text-muted-foreground"
                      >
                        <span className="text-accent">
                          {idx + 1}.
                        </span>
                        <span className="truncate">
                          {node?.label ?? nodeId}
                        </span>
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
          </div>
        )}

        {/* Communities */}
        <div className="px-4 py-3">
          <button
            onClick={() => setCommunitiesOpen(!communitiesOpen)}
            className="mb-2 flex w-full items-center gap-1.5 text-xs font-medium text-muted-foreground"
          >
            <Users size={12} />
            <span>
              Communities ({communities.length})
            </span>
            {communitiesOpen ? (
              <ChevronDown size={12} className="ml-auto" />
            ) : (
              <ChevronRight size={12} className="ml-auto" />
            )}
          </button>
          {communitiesOpen && (
            <div className="space-y-1.5">
              {communities.length === 0 && (
                <p className="text-[10px] text-muted-foreground">
                  No communities detected. Build the graph first.
                </p>
              )}
              {communities.map((community) => (
                <div
                  key={community.id}
                  className="flex items-center gap-2 rounded-md bg-muted/30 px-2 py-1.5"
                >
                  <span
                    className="inline-block h-3 w-3 flex-shrink-0 rounded"
                    style={{
                      backgroundColor: communityColor(community.id),
                    }}
                  />
                  <div className="min-w-0 flex-1">
                    <p className="text-xs font-medium text-foreground">
                      Cluster {community.id + 1}
                    </p>
                    <p className="text-[10px] text-muted-foreground">
                      {community.size} member{community.size !== 1 ? "s" : ""}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
