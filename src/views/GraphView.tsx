import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import ForceGraph2D, { type ForceGraphMethods } from "react-force-graph-2d";
import {
  Network,
  ZoomIn,
  ZoomOut,
  Maximize2,
  RefreshCw,
  Loader2,
  Filter,
  ChevronDown,
  ChevronRight,
  Search,
} from "lucide-react";
import { useCollectionStore } from "../stores/collectionStore";
import { useAppStore } from "../stores/appStore";
import { useToastStore } from "../stores/toastStore";
import { FILE_TYPE_COLORS, getFileTypeColor } from "../utils/fileTypeColors";
import { GraphSkeleton } from "../components/LoadingSkeleton";
import { EmptyState } from "../components/EmptyState";
import { GraphLegend } from "../components/GraphLegend";
import type { GraphData, GraphNode } from "../types";

interface ProcessedNode extends GraphNode {
  x?: number;
  y?: number;
  color: string;
}

interface ProcessedLink {
  source: string | ProcessedNode;
  target: string | ProcessedNode;
  weight: number;
  relationship_type: string;
}

interface ProcessedGraphData {
  nodes: ProcessedNode[];
  links: ProcessedLink[];
}

interface ContextMenu {
  x: number;
  y: number;
  node: ProcessedNode;
}

export function GraphView() {
  const activeCollectionId = useCollectionStore((s) => s.activeCollectionId);
  const setActiveView = useAppStore((s) => s.setActiveView);
  const setSelectedDocument = useAppStore((s) => s.setSelectedDocument);
  const addToast = useToastStore((s) => s.addToast);

  const [graphData, setGraphData] = useState<ProcessedGraphData>({ nodes: [], links: [] });
  const [loading, setLoading] = useState(false);
  const [building, setBuilding] = useState(false);
  const [selectedNode, setSelectedNode] = useState<ProcessedNode | null>(null);
  const [hoveredNode, setHoveredNode] = useState<ProcessedNode | null>(null);
  const [filterOpen, setFilterOpen] = useState(false);
  const [enabledTypes, setEnabledTypes] = useState<Set<string>>(new Set(Object.keys(FILE_TYPE_COLORS)));
  const [minWeight, setMinWeight] = useState(0);
  const [containerSize, setContainerSize] = useState({ width: 800, height: 600 });

  // Graph search
  const [searchQuery, setSearchQuery] = useState("");
  const [searchMatches, setSearchMatches] = useState<Set<string>>(new Set());

  // Context menu
  const [contextMenu, setContextMenu] = useState<ContextMenu | null>(null);

  // Hidden nodes
  const [hiddenNodes, setHiddenNodes] = useState<Set<string>>(new Set());

  const graphRef = useRef<ForceGraphMethods<ProcessedNode, ProcessedLink> | undefined>(undefined);
  const containerRef = useRef<HTMLDivElement>(null);

  // Resize observer
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        setContainerSize({
          width: entry.contentRect.width,
          height: entry.contentRect.height,
        });
      }
    });

    observer.observe(container);
    return () => observer.disconnect();
  }, []);

  // Close context menu on click elsewhere
  useEffect(() => {
    const handleClick = () => setContextMenu(null);
    window.addEventListener("click", handleClick);
    return () => window.removeEventListener("click", handleClick);
  }, []);

  const loadGraph = useCallback(async () => {
    if (!activeCollectionId) return;
    setLoading(true);
    try {
      const data = await invoke<GraphData>("get_graph", {
        collectionId: activeCollectionId,
      });

      const processedNodes: ProcessedNode[] = data.nodes.map((node) => ({
        ...node,
        color: getFileTypeColor(node.file_type),
      }));

      const processedLinks: ProcessedLink[] = data.links.map((link) => ({
        source: link.source,
        target: link.target,
        weight: link.weight,
        relationship_type: link.relationship_type,
      }));

      setGraphData({ nodes: processedNodes, links: processedLinks });
    } catch (error) {
      console.error("Failed to load graph:", error);
    } finally {
      setLoading(false);
    }
  }, [activeCollectionId]);

  useEffect(() => {
    loadGraph();
  }, [loadGraph]);

  const handleBuildGraph = useCallback(async () => {
    if (!activeCollectionId) return;
    setBuilding(true);
    try {
      await invoke("build_graph", { collectionId: activeCollectionId });
      addToast("success", "Graph built successfully");
      await loadGraph();
    } catch (error) {
      console.error("Failed to build graph:", error);
      addToast("error", `Failed to build graph: ${String(error)}`);
    } finally {
      setBuilding(false);
    }
  }, [activeCollectionId, loadGraph, addToast]);

  const handleZoomIn = useCallback(() => {
    if (graphRef.current) {
      const currentZoom = graphRef.current.zoom();
      graphRef.current.zoom(currentZoom * 1.3, 300);
    }
  }, []);

  const handleZoomOut = useCallback(() => {
    if (graphRef.current) {
      const currentZoom = graphRef.current.zoom();
      graphRef.current.zoom(currentZoom / 1.3, 300);
    }
  }, []);

  const handleFitToView = useCallback(() => {
    graphRef.current?.zoomToFit(400);
  }, []);

  const handleNodeClick = useCallback((node: ProcessedNode) => {
    setSelectedNode(node);
    setContextMenu(null);
  }, []);

  const handleNodeDoubleClick = useCallback(
    (node: ProcessedNode) => {
      setSelectedDocument(node.id);
      setActiveView("document-detail");
    },
    [setSelectedDocument, setActiveView],
  );

  const handleNodeRightClick = useCallback(
    (node: ProcessedNode, event: MouseEvent) => {
      event.preventDefault();
      setContextMenu({
        x: event.clientX,
        y: event.clientY,
        node,
      });
    },
    [],
  );

  const toggleFileType = useCallback((fileType: string) => {
    setEnabledTypes((prev) => {
      const next = new Set(prev);
      if (next.has(fileType)) {
        next.delete(fileType);
      } else {
        next.add(fileType);
      }
      return next;
    });
  }, []);

  // Graph search handler
  const handleGraphSearch = useCallback((q: string) => {
    setSearchQuery(q);
    if (!q.trim()) {
      setSearchMatches(new Set());
      return;
    }
    const lower = q.toLowerCase();
    const matches = new Set<string>();
    for (const node of graphData.nodes) {
      if (node.label.toLowerCase().includes(lower)) {
        matches.add(node.id);
      }
    }
    setSearchMatches(matches);

    // Zoom to first match
    if (matches.size > 0 && graphRef.current) {
      const firstId = matches.values().next().value;
      const firstNode = graphData.nodes.find((n) => n.id === firstId);
      if (firstNode?.x != null && firstNode?.y != null) {
        graphRef.current.centerAt(firstNode.x, firstNode.y, 1000);
        graphRef.current.zoom(3, 1000);
      }
    }
  }, [graphData.nodes]);

  // Filter graph data
  const filteredData = useMemo<ProcessedGraphData>(() => ({
    nodes: graphData.nodes.filter((n) =>
      enabledTypes.has(n.file_type.toLowerCase()) && !hiddenNodes.has(n.id),
    ),
    links: graphData.links.filter((l) => {
      if (l.weight < minWeight) return false;
      const sourceId = typeof l.source === "string" ? l.source : l.source.id;
      const targetId = typeof l.target === "string" ? l.target : l.target.id;
      if (hiddenNodes.has(sourceId) || hiddenNodes.has(targetId)) return false;
      const sourceNode = graphData.nodes.find((n) => n.id === sourceId);
      const targetNode = graphData.nodes.find((n) => n.id === targetId);
      if (!sourceNode || !targetNode) return false;
      return (
        enabledTypes.has(sourceNode.file_type.toLowerCase()) &&
        enabledTypes.has(targetNode.file_type.toLowerCase())
      );
    }),
  }), [graphData, enabledTypes, minWeight, hiddenNodes]);

  const nodeCanvasObject = useCallback(
    (node: ProcessedNode, ctx: CanvasRenderingContext2D, globalScale: number) => {
      const baseSize = Math.max(20, Math.min(60, Math.sqrt(node.chunk_count) * 8));
      const halfW = baseSize / 2;
      const halfH = baseSize * 0.6 / 2;
      const x = node.x ?? 0;
      const y = node.y ?? 0;
      const isHovered = hoveredNode?.id === node.id;
      const isSelected = selectedNode?.id === node.id;
      const isSearchMatch = searchMatches.has(node.id);
      const isNeighbor =
        hoveredNode &&
        filteredData.links.some((l) => {
          const sid = typeof l.source === "string" ? l.source : l.source.id;
          const tid = typeof l.target === "string" ? l.target : l.target.id;
          return (
            (sid === hoveredNode.id && tid === node.id) ||
            (tid === hoveredNode.id && sid === node.id)
          );
        });

      const dimmed = (hoveredNode && !isHovered && !isNeighbor) ||
        (searchMatches.size > 0 && !isSearchMatch);

      // Rounded rectangle
      const radius = 4 / globalScale;
      ctx.beginPath();
      ctx.moveTo(x - halfW + radius, y - halfH);
      ctx.lineTo(x + halfW - radius, y - halfH);
      ctx.arcTo(x + halfW, y - halfH, x + halfW, y - halfH + radius, radius);
      ctx.lineTo(x + halfW, y + halfH - radius);
      ctx.arcTo(x + halfW, y + halfH, x + halfW - radius, y + halfH, radius);
      ctx.lineTo(x - halfW + radius, y + halfH);
      ctx.arcTo(x - halfW, y + halfH, x - halfW, y + halfH - radius, radius);
      ctx.lineTo(x - halfW, y - halfH + radius);
      ctx.arcTo(x - halfW, y - halfH, x - halfW + radius, y - halfH, radius);
      ctx.closePath();

      ctx.fillStyle = dimmed ? `${node.color}33` : node.color;
      ctx.fill();

      // Border ring for selected/search match
      if (isSelected || isSearchMatch) {
        ctx.strokeStyle = isSelected ? "#ffffff" : "#fbbf24";
        ctx.lineWidth = 2 / globalScale;
        ctx.stroke();
      } else if (isHovered) {
        ctx.strokeStyle = node.color;
        ctx.lineWidth = 1.5 / globalScale;
        ctx.stroke();
      }

      // File type abbreviation inside node
      const fontSize = Math.max(8, Math.min(12, baseSize * 0.35));
      ctx.font = `bold ${fontSize}px sans-serif`;
      ctx.textAlign = "center";
      ctx.textBaseline = "middle";
      ctx.fillStyle = dimmed ? "rgba(255,255,255,0.3)" : "rgba(255,255,255,0.95)";
      ctx.fillText(node.file_type.toUpperCase().slice(0, 4), x, y);

      // Label below node
      if (globalScale > 1.2 || isHovered || isSelected) {
        const labelFontSize = Math.max(10, 12 / globalScale);
        ctx.font = `${labelFontSize}px sans-serif`;
        ctx.textAlign = "center";
        ctx.textBaseline = "top";
        ctx.fillStyle = dimmed ? "#94a3b866" : "#94a3b8";
        const label = node.label.length > 20 ? node.label.slice(0, 18) + "..." : node.label;
        ctx.fillText(label, x, y + halfH + 3);
      }
    },
    [hoveredNode, selectedNode, filteredData.links, searchMatches],
  );

  const linkCanvasObject = useCallback(
    (link: ProcessedLink, ctx: CanvasRenderingContext2D) => {
      const source = typeof link.source === "string" ? null : link.source;
      const target = typeof link.target === "string" ? null : link.target;
      if (!source || !target) return;

      ctx.beginPath();
      ctx.moveTo(source.x ?? 0, source.y ?? 0);
      ctx.lineTo(target.x ?? 0, target.y ?? 0);
      ctx.strokeStyle = `rgba(148, 163, 184, ${Math.min(link.weight, 0.6)})`;
      ctx.lineWidth = Math.max(0.5, link.weight * 3);
      ctx.stroke();
    },
    [],
  );

  if (!activeCollectionId) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-4 text-muted-foreground">
        <Network size={48} strokeWidth={1.5} />
        <p className="text-sm">Select a collection to view the knowledge graph</p>
      </div>
    );
  }

  if (loading) {
    return <GraphSkeleton />;
  }

  if (graphData.nodes.length === 0) {
    return (
      <EmptyState
        icon={Network}
        title="Your knowledge graph"
        description="Import documents to see connections between your knowledge."
        action={
          <button
            onClick={handleBuildGraph}
            disabled={building}
            className="flex items-center gap-2 rounded-lg bg-accent px-4 py-2 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90 disabled:opacity-50"
          >
            {building ? (
              <Loader2 size={16} className="animate-spin" />
            ) : (
              <RefreshCw size={16} />
            )}
            Build Graph
          </button>
        }
      />
    );
  }

  return (
    <div className="relative flex flex-1 overflow-hidden" ref={containerRef}>
      {/* Graph Canvas */}
      <ForceGraph2D
        ref={graphRef as React.MutableRefObject<ForceGraphMethods<ProcessedNode, ProcessedLink> | undefined>}
        graphData={filteredData}
        width={containerSize.width - (selectedNode ? 280 : 0)}
        height={containerSize.height}
        nodeCanvasObject={nodeCanvasObject}
        linkCanvasObject={linkCanvasObject}
        onNodeClick={handleNodeClick}
        onNodeRightClick={(node, event) => handleNodeRightClick(node as ProcessedNode, event)}
        onNodeHover={(node) => setHoveredNode(node as ProcessedNode | null)}
        onNodeDragEnd={(node) => {
          const n = node as ProcessedNode;
          n.x = node.x;
          n.y = node.y;
        }}
        cooldownTicks={100}
        nodeId="id"
        linkSource="source"
        linkTarget="target"
        backgroundColor="transparent"
      />

      {/* Search Bar */}
      <div className="absolute left-1/2 top-4 -translate-x-1/2">
        <div className="relative">
          <Search
            size={14}
            className="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground"
          />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => handleGraphSearch(e.target.value)}
            placeholder="Search nodes..."
            className="h-8 w-56 rounded-md border border-border bg-background/90 pl-8 pr-3 text-xs text-foreground outline-none backdrop-blur focus:ring-1 focus:ring-accent"
          />
          {searchQuery && (
            <button
              onClick={() => handleGraphSearch("")}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground"
            >
              <span className="text-xs">×</span>
            </button>
          )}
          {searchMatches.size > 0 && (
            <span className="absolute -right-8 top-1/2 -translate-y-1/2 text-[10px] text-muted-foreground">
              {searchMatches.size}
            </span>
          )}
        </div>
      </div>

      {/* Floating Controls */}
      <div className="absolute right-4 top-4 flex flex-col gap-1">
        <button
          onClick={handleZoomIn}
          className="flex h-8 w-8 items-center justify-center rounded-md border border-border bg-background/90 text-foreground shadow-sm backdrop-blur transition-colors hover:bg-muted"
          title="Zoom in"
        >
          <ZoomIn size={14} />
        </button>
        <button
          onClick={handleZoomOut}
          className="flex h-8 w-8 items-center justify-center rounded-md border border-border bg-background/90 text-foreground shadow-sm backdrop-blur transition-colors hover:bg-muted"
          title="Zoom out"
        >
          <ZoomOut size={14} />
        </button>
        <button
          onClick={handleFitToView}
          className="flex h-8 w-8 items-center justify-center rounded-md border border-border bg-background/90 text-foreground shadow-sm backdrop-blur transition-colors hover:bg-muted"
          title="Fit to view"
        >
          <Maximize2 size={14} />
        </button>
        <button
          onClick={handleBuildGraph}
          disabled={building}
          className="flex h-8 w-8 items-center justify-center rounded-md border border-border bg-background/90 text-foreground shadow-sm backdrop-blur transition-colors hover:bg-muted disabled:opacity-50"
          title="Rebuild graph"
        >
          {building ? (
            <Loader2 size={14} className="animate-spin" />
          ) : (
            <RefreshCw size={14} />
          )}
        </button>
        {hiddenNodes.size > 0 && (
          <button
            onClick={() => setHiddenNodes(new Set())}
            className="flex h-8 items-center justify-center rounded-md border border-border bg-background/90 px-2 text-[10px] text-foreground shadow-sm backdrop-blur transition-colors hover:bg-muted"
            title="Show all hidden nodes"
          >
            Show {hiddenNodes.size}
          </button>
        )}
      </div>

      {/* Filter Panel */}
      <div className="absolute left-4 top-4">
        <div className="rounded-md border border-border bg-background/90 shadow-sm backdrop-blur">
          <button
            onClick={() => setFilterOpen(!filterOpen)}
            className="flex items-center gap-2 px-3 py-2 text-xs font-medium text-foreground"
          >
            <Filter size={12} />
            Filters
            {filterOpen ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
          </button>
          {filterOpen && (
            <div className="border-t border-border px-3 py-2">
              <p className="mb-2 text-[10px] font-medium uppercase text-muted-foreground">
                File Types
              </p>
              <div className="space-y-1">
                {Object.entries(FILE_TYPE_COLORS).map(([type, color]) => (
                  <label
                    key={type}
                    className="flex cursor-pointer items-center gap-2 text-xs text-foreground"
                  >
                    <input
                      type="checkbox"
                      checked={enabledTypes.has(type)}
                      onChange={() => toggleFileType(type)}
                      className="rounded border-border"
                    />
                    <span
                      className="inline-block h-2 w-2 rounded-full"
                      style={{ backgroundColor: color }}
                    />
                    {type.toUpperCase()}
                  </label>
                ))}
              </div>
              <p className="mb-1 mt-3 text-[10px] font-medium uppercase text-muted-foreground">
                Min Similarity
              </p>
              <input
                type="range"
                min={0}
                max={100}
                value={minWeight * 100}
                onChange={(e) => setMinWeight(Number(e.target.value) / 100)}
                className="w-full"
              />
              <p className="text-right text-[10px] text-muted-foreground">
                {Math.round(minWeight * 100)}%
              </p>
            </div>
          )}
        </div>
      </div>

      {/* Legend */}
      <GraphLegend />

      {/* Context Menu */}
      {contextMenu && (
        <div
          className="fixed z-50 min-w-[160px] rounded-md border border-border bg-background py-1 shadow-lg"
          style={{ left: contextMenu.x, top: contextMenu.y }}
          onClick={(e) => e.stopPropagation()}
        >
          <button
            onClick={() => {
              setSelectedDocument(contextMenu.node.id);
              setActiveView("document-detail");
              setContextMenu(null);
            }}
            className="flex w-full items-center px-3 py-1.5 text-xs text-foreground hover:bg-muted"
          >
            View Document
          </button>
          <button
            onClick={() => {
              setActiveView("search");
              setContextMenu(null);
            }}
            className="flex w-full items-center px-3 py-1.5 text-xs text-foreground hover:bg-muted"
          >
            Find Similar
          </button>
          <button
            onClick={() => {
              setActiveView("chat");
              setContextMenu(null);
            }}
            className="flex w-full items-center px-3 py-1.5 text-xs text-foreground hover:bg-muted"
          >
            Chat About This
          </button>
          <div className="my-1 border-t border-border" />
          <button
            onClick={() => {
              setHiddenNodes((prev) => new Set([...prev, contextMenu.node.id]));
              setContextMenu(null);
            }}
            className="flex w-full items-center px-3 py-1.5 text-xs text-muted-foreground hover:bg-muted hover:text-foreground"
          >
            Hide Node
          </button>
        </div>
      )}

      {/* Node Detail Panel */}
      {selectedNode && (
        <div className="absolute bottom-4 right-4 w-64 rounded-lg border border-border bg-background/95 p-4 shadow-lg backdrop-blur">
          <div className="mb-3 flex items-center justify-between">
            <h3 className="truncate text-sm font-medium text-foreground">
              {selectedNode.label}
            </h3>
            <button
              onClick={() => setSelectedNode(null)}
              className="text-xs text-muted-foreground hover:text-foreground"
            >
              Close
            </button>
          </div>
          <div className="space-y-2 text-xs text-muted-foreground">
            <div className="flex justify-between">
              <span>Type</span>
              <span
                className="rounded px-1.5 py-0.5 text-[10px] font-semibold uppercase text-white"
                style={{
                  backgroundColor: getFileTypeColor(selectedNode.file_type),
                }}
              >
                {selectedNode.file_type}
              </span>
            </div>
            <div className="flex justify-between">
              <span>Chunks</span>
              <span className="text-foreground">{selectedNode.chunk_count}</span>
            </div>
            <div className="flex justify-between">
              <span>Words</span>
              <span className="text-foreground">
                {selectedNode.word_count.toLocaleString()}
              </span>
            </div>
          </div>
          <button
            onClick={() => handleNodeDoubleClick(selectedNode)}
            className="mt-3 w-full rounded-md bg-accent px-3 py-1.5 text-xs font-medium text-accent-foreground transition-colors hover:bg-accent/90"
          >
            View Document
          </button>
        </div>
      )}
    </div>
  );
}
