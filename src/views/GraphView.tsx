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
} from "lucide-react";
import { useCollectionStore } from "../stores/collectionStore";
import { useAppStore } from "../stores/appStore";
import { useToastStore } from "../stores/toastStore";
import { FILE_TYPE_COLORS, getFileTypeColor } from "../utils/fileTypeColors";
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
  }, []);

  const handleNodeDoubleClick = useCallback(
    (node: ProcessedNode) => {
      setSelectedDocument(node.id);
      setActiveView("document-detail");
    },
    [setSelectedDocument, setActiveView],
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

  // Filter graph data
  const filteredData = useMemo<ProcessedGraphData>(() => ({
    nodes: graphData.nodes.filter((n) =>
      enabledTypes.has(n.file_type.toLowerCase()),
    ),
    links: graphData.links.filter((l) => {
      if (l.weight < minWeight) return false;
      const sourceId = typeof l.source === "string" ? l.source : l.source.id;
      const targetId = typeof l.target === "string" ? l.target : l.target.id;
      const sourceNode = graphData.nodes.find((n) => n.id === sourceId);
      const targetNode = graphData.nodes.find((n) => n.id === targetId);
      if (!sourceNode || !targetNode) return false;
      return (
        enabledTypes.has(sourceNode.file_type.toLowerCase()) &&
        enabledTypes.has(targetNode.file_type.toLowerCase())
      );
    }),
  }), [graphData, enabledTypes, minWeight]);

  const nodeCanvasObject = useCallback(
    (node: ProcessedNode, ctx: CanvasRenderingContext2D, globalScale: number) => {
      const size = Math.max(4, Math.sqrt(node.chunk_count) * 3);
      const isHovered = hoveredNode?.id === node.id;
      const isSelected = selectedNode?.id === node.id;
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

      const dimmed = hoveredNode && !isHovered && !isNeighbor;

      ctx.beginPath();
      ctx.arc(node.x ?? 0, node.y ?? 0, size, 0, 2 * Math.PI);
      ctx.fillStyle = dimmed ? `${node.color}33` : node.color;
      ctx.fill();

      if (isSelected || isHovered) {
        ctx.strokeStyle = isSelected ? "#ffffff" : node.color;
        ctx.lineWidth = 2 / globalScale;
        ctx.stroke();
      }

      if (globalScale > 1.5 || isHovered) {
        ctx.font = `${Math.max(10, 12 / globalScale)}px sans-serif`;
        ctx.textAlign = "center";
        ctx.textBaseline = "top";
        ctx.fillStyle = dimmed ? "#94a3b866" : "#94a3b8";
        ctx.fillText(node.label, node.x ?? 0, (node.y ?? 0) + size + 2);
      }
    },
    [hoveredNode, selectedNode, filteredData.links],
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
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-4 text-muted-foreground">
        <Loader2 size={32} className="animate-spin" />
        <p className="text-sm">Loading graph...</p>
      </div>
    );
  }

  if (graphData.nodes.length === 0) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-4 text-muted-foreground">
        <Network size={48} strokeWidth={1.5} />
        <h2 className="text-lg font-semibold text-foreground">Knowledge Graph</h2>
        <p className="text-sm">Import documents to build your knowledge graph</p>
        <button
          onClick={handleBuildGraph}
          disabled={building}
          className="mt-2 flex items-center gap-2 rounded-lg bg-accent px-4 py-2 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90 disabled:opacity-50"
        >
          {building ? (
            <Loader2 size={16} className="animate-spin" />
          ) : (
            <RefreshCw size={16} />
          )}
          Build Graph
        </button>
      </div>
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
