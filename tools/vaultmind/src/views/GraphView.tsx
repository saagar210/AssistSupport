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
  AlertCircle,
  PanelRightOpen,
  PanelRightClose,
} from "lucide-react";
import { useCollectionStore } from "../stores/collectionStore";
import { useAppStore } from "../stores/appStore";
import { useToastStore } from "../stores/toastStore";
import { FILE_TYPE_COLORS, getFileTypeColor } from "../utils/fileTypeColors";
import { GraphSkeleton } from "../components/LoadingSkeleton";
import { EmptyState } from "../components/EmptyState";
import { GraphLegend } from "../components/GraphLegend";
import { ContextualHelp } from "../components/ContextualHelp";
import { GraphSidebar, communityColor } from "../components/GraphSidebar";
import type { GraphData, GraphNode, Community, GraphStats } from "../types";

interface ProcessedNode extends GraphNode {
  x?: number;
  y?: number;
  color: string;
  communityId?: number;
  degree: number;
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
  const [error, setError] = useState<string | null>(null);
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

  // Communities
  const [communities, setCommunities] = useState<Community[]>([]);

  // Path highlighting
  const [highlightedPath, setHighlightedPath] = useState<string[]>([]);
  const [pathNodeSet, setPathNodeSet] = useState<Set<string>>(new Set());

  // Sidebar visibility
  const [sidebarOpen, setSidebarOpen] = useState(false);

  // Path-finding mode: stores source node id when user clicks "Find Path To..."
  const [pathFindingFrom, setPathFindingFrom] = useState<string | null>(null);

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

  // Load communities and map chunk_ids to document_ids for coloring
  const loadCommunities = useCallback(
    async (collectionId: string, nodes: ProcessedNode[]): Promise<ProcessedNode[]> => {
      try {
        const result = await invoke<Community[]>("detect_graph_communities", {
          collectionId,
        });

        // Community members are chunk_ids. Map them to document_ids via traverse.
        const chunkToDoc = new Map<string, string>();
        const allChunkIds = result.flatMap((c) => c.members);
        const uniqueChunkIds = [...new Set(allChunkIds)];

        // Batch traverse each chunk at depth 0 to get its document_id
        const traversalPromises = uniqueChunkIds.map(async (chunkId) => {
          try {
            const traversalResult = await invoke<Array<{ chunk_id: string; document_id: string }>>(
              "traverse_graph_cmd",
              {
                collectionId,
                startChunkId: chunkId,
                maxDepth: 0,
                minWeight: 0.0,
              },
            );
            if (traversalResult.length > 0) {
              chunkToDoc.set(chunkId, traversalResult[0].document_id);
            }
          } catch {
            // Chunk may not exist or have no edges; skip
          }
        });

        await Promise.all(traversalPromises);

        // Assign each community to documents
        const docToCommunity = new Map<string, number>();
        for (const community of result) {
          for (const chunkId of community.members) {
            const docId = chunkToDoc.get(chunkId);
            if (docId && !docToCommunity.has(docId)) {
              docToCommunity.set(docId, community.id);
            }
          }
        }

        // Update node colors based on community
        const updatedNodes = nodes.map((node) => {
          const commId = docToCommunity.get(node.id);
          if (commId !== undefined) {
            return {
              ...node,
              communityId: commId,
              color: communityColor(commId),
            };
          }
          return node;
        });

        setCommunities(result);
        return updatedNodes;
      } catch (err) {
        console.error("Failed to detect communities:", err);
        return nodes;
      }
    },
    [],
  );

  const loadGraph = useCallback(async () => {
    if (!activeCollectionId) return;
    setLoading(true);
    setError(null);
    try {
      const data = await invoke<GraphData>("get_graph", {
        collectionId: activeCollectionId,
      });

      // Calculate degree (connection count) per node
      const degreeMap = new Map<string, number>();
      for (const link of data.links) {
        degreeMap.set(link.source, (degreeMap.get(link.source) ?? 0) + 1);
        degreeMap.set(link.target, (degreeMap.get(link.target) ?? 0) + 1);
      }

      const processedNodes: ProcessedNode[] = data.nodes.map((node) => ({
        ...node,
        color: getFileTypeColor(node.file_type),
        degree: degreeMap.get(node.id) ?? 0,
      }));

      const processedLinks: ProcessedLink[] = data.links.map((link) => ({
        source: link.source,
        target: link.target,
        weight: link.weight,
        relationship_type: link.relationship_type,
      }));

      // Load communities and update node colors
      const coloredNodes = await loadCommunities(activeCollectionId, processedNodes);

      setGraphData({ nodes: coloredNodes, links: processedLinks });
    } catch (err) {
      console.error("Failed to load graph:", err);
      setError("Failed to load graph: " + String(err));
    } finally {
      setLoading(false);
    }
  }, [activeCollectionId, loadCommunities]);

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
    } catch (err) {
      console.error("Failed to build graph:", err);
      addToast("error", `Failed to build graph: ${String(err)}`);
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

  // Find path between two documents
  const handleFindPath = useCallback(
    async (fromId: string, toId: string) => {
      if (!activeCollectionId) return;
      try {
        // Get a chunk for each document via traverse
        const [fromTraversal, toTraversal] = await Promise.all([
          invoke<Array<{ chunk_id: string; document_id: string }>>(
            "traverse_graph_cmd",
            {
              collectionId: activeCollectionId,
              startChunkId: fromId,
              maxDepth: 0,
              minWeight: 0.0,
            },
          ).catch(() => [] as Array<{ chunk_id: string; document_id: string }>),
          invoke<Array<{ chunk_id: string; document_id: string }>>(
            "traverse_graph_cmd",
            {
              collectionId: activeCollectionId,
              startChunkId: toId,
              maxDepth: 0,
              minWeight: 0.0,
            },
          ).catch(() => [] as Array<{ chunk_id: string; document_id: string }>),
        ]);

        const fromChunkId = fromTraversal.length > 0 ? fromTraversal[0].chunk_id : fromId;
        const toChunkId = toTraversal.length > 0 ? toTraversal[0].chunk_id : toId;

        const pathChunkIds = await invoke<string[]>("find_graph_path", {
          collectionId: activeCollectionId,
          fromChunkId,
          toChunkId,
        });

        if (pathChunkIds.length === 0) {
          addToast("info", "No path found between these nodes");
          setHighlightedPath([]);
          setPathNodeSet(new Set());
          return;
        }

        // Map chunk IDs back to document IDs for highlighting
        const docIds: string[] = [];
        const seen = new Set<string>();
        for (const chunkId of pathChunkIds) {
          try {
            const traversal = await invoke<Array<{ chunk_id: string; document_id: string }>>(
              "traverse_graph_cmd",
              {
                collectionId: activeCollectionId,
                startChunkId: chunkId,
                maxDepth: 0,
                minWeight: 0.0,
              },
            );
            if (traversal.length > 0 && !seen.has(traversal[0].document_id)) {
              seen.add(traversal[0].document_id);
              docIds.push(traversal[0].document_id);
            }
          } catch {
            if (!seen.has(chunkId)) {
              seen.add(chunkId);
              docIds.push(chunkId);
            }
          }
        }

        setHighlightedPath(docIds);
        setPathNodeSet(new Set(docIds));
        addToast("success", `Path found: ${docIds.length} nodes`);
      } catch (err) {
        console.error("Failed to find path:", err);
        addToast("error", `Path finding failed: ${String(err)}`);
      }
    },
    [activeCollectionId, addToast],
  );

  const handleNodeClick = useCallback(
    (node: ProcessedNode) => {
      setSelectedNode(node);
      setContextMenu(null);
      setSidebarOpen(true);

      // If in path-finding mode, find path to this node
      if (pathFindingFrom && pathFindingFrom !== node.id) {
        handleFindPath(pathFindingFrom, node.id);
        setPathFindingFrom(null);
      }
    },
    [pathFindingFrom, handleFindPath],
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

  // Graph stats
  const graphStats = useMemo<GraphStats>(() => {
    const nodeCount = graphData.nodes.length;
    const edgeCount = graphData.links.length;
    const maxEdges = nodeCount > 1 ? (nodeCount * (nodeCount - 1)) / 2 : 1;
    const density = edgeCount / maxEdges;
    return { nodeCount, edgeCount, density };
  }, [graphData]);

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

  // Connection count for selected node
  const connectionCount = useMemo(() => {
    if (!selectedNode) return 0;
    return filteredData.links.filter((l) => {
      const sid = typeof l.source === "string" ? l.source : l.source.id;
      const tid = typeof l.target === "string" ? l.target : l.target.id;
      return sid === selectedNode.id || tid === selectedNode.id;
    }).length;
  }, [selectedNode, filteredData.links]);

  // Max weight for edge thickness scaling
  const maxWeight = useMemo(() => {
    if (graphData.links.length === 0) return 1;
    return Math.max(...graphData.links.map((l) => l.weight));
  }, [graphData.links]);

  // Max degree for node sizing
  const maxDegree = useMemo(() => {
    if (graphData.nodes.length === 0) return 1;
    return Math.max(...graphData.nodes.map((n) => n.degree), 1);
  }, [graphData.nodes]);

  const nodeCanvasObject = useCallback(
    (node: ProcessedNode, ctx: CanvasRenderingContext2D, globalScale: number) => {
      // Size proportional to connection count (degree)
      const degreeScale = Math.max(0.6, Math.min(2.0, 0.6 + (node.degree / maxDegree) * 1.4));
      const baseSize = Math.max(18, Math.min(60, 22 * degreeScale));
      const halfW = baseSize / 2;
      const halfH = (baseSize * 0.6) / 2;
      const x = node.x ?? 0;
      const y = node.y ?? 0;
      const isHovered = hoveredNode?.id === node.id;
      const isSelected = selectedNode?.id === node.id;
      const isSearchMatch = searchMatches.has(node.id);
      const isOnPath = pathNodeSet.has(node.id);
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

      const dimmed =
        (hoveredNode && !isHovered && !isNeighbor) ||
        (searchMatches.size > 0 && !isSearchMatch) ||
        (pathNodeSet.size > 0 && !isOnPath);

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

      // Border for selected/search/path
      if (isOnPath) {
        ctx.strokeStyle = "#fbbf24";
        ctx.lineWidth = 2.5 / globalScale;
        ctx.stroke();
      } else if (isSelected) {
        ctx.strokeStyle = "#ffffff";
        ctx.lineWidth = 2 / globalScale;
        ctx.stroke();
      } else if (isSearchMatch) {
        ctx.strokeStyle = "#fbbf24";
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

      // Label below node: document title, truncated
      if (globalScale > 1.2 || isHovered || isSelected || isOnPath) {
        const labelFontSize = Math.max(10, 12 / globalScale);
        ctx.font = `${labelFontSize}px sans-serif`;
        ctx.textAlign = "center";
        ctx.textBaseline = "top";
        ctx.fillStyle = dimmed ? "#94a3b866" : "#94a3b8";
        const label =
          node.label.length > 24 ? node.label.slice(0, 22) + "..." : node.label;
        ctx.fillText(label, x, y + halfH + 3);
      }
    },
    [hoveredNode, selectedNode, filteredData.links, searchMatches, pathNodeSet, maxDegree],
  );

  const linkCanvasObject = useCallback(
    (link: ProcessedLink, ctx: CanvasRenderingContext2D) => {
      const source = typeof link.source === "string" ? null : link.source;
      const target = typeof link.target === "string" ? null : link.target;
      if (!source || !target) return;

      const sourceId = source.id;
      const targetId = target.id;
      const isOnPath =
        pathNodeSet.size > 0 &&
        pathNodeSet.has(sourceId) &&
        pathNodeSet.has(targetId);

      ctx.beginPath();
      ctx.moveTo(source.x ?? 0, source.y ?? 0);
      ctx.lineTo(target.x ?? 0, target.y ?? 0);

      if (isOnPath) {
        ctx.strokeStyle = "rgba(251, 191, 36, 0.8)";
        ctx.lineWidth = Math.max(2, (link.weight / maxWeight) * 5);
      } else {
        const alpha = pathNodeSet.size > 0
          ? Math.min(link.weight * 0.3, 0.2)
          : Math.min(link.weight, 0.6);
        ctx.strokeStyle = `rgba(148, 163, 184, ${alpha})`;
        // Edge thickness proportional to weight
        ctx.lineWidth = Math.max(0.5, (link.weight / maxWeight) * 4);
      }
      ctx.stroke();
    },
    [pathNodeSet, maxWeight],
  );

  const sidebarWidth = sidebarOpen ? 288 : 0;

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

  if (error && graphData.nodes.length === 0) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-4 text-muted-foreground">
        <AlertCircle size={48} strokeWidth={1.5} className="text-destructive" />
        <p className="text-sm text-destructive">{error}</p>
        <button
          onClick={loadGraph}
          className="flex items-center gap-2 rounded-lg bg-accent px-4 py-2 text-sm font-medium text-accent-foreground transition-colors hover:bg-accent/90"
        >
          <RefreshCw size={16} />
          Retry
        </button>
        <div aria-live="polite" className="sr-only">
          {error}
        </div>
      </div>
    );
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
      <div className="flex-1">
        <ForceGraph2D
          ref={graphRef as React.MutableRefObject<ForceGraphMethods<ProcessedNode, ProcessedLink> | undefined>}
          graphData={filteredData}
          width={containerSize.width - sidebarWidth}
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
      </div>

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
              <span className="text-xs">x</span>
            </button>
          )}
          {searchMatches.size > 0 && (
            <span className="absolute -right-8 top-1/2 -translate-y-1/2 text-[10px] text-muted-foreground">
              {searchMatches.size}
            </span>
          )}
        </div>
      </div>

      {/* Path-finding mode indicator */}
      {pathFindingFrom && (
        <div className="absolute left-1/2 top-14 -translate-x-1/2 rounded-md border border-accent/50 bg-accent/10 px-3 py-1.5 text-xs text-accent backdrop-blur">
          Click a target node to find path...
          <button
            onClick={() => setPathFindingFrom(null)}
            className="ml-2 text-muted-foreground hover:text-foreground"
          >
            Cancel
          </button>
        </div>
      )}

      {/* Floating Controls */}
      <div
        className="absolute top-4 flex flex-col gap-1"
        style={{ right: sidebarOpen ? sidebarWidth + 16 : 16 }}
      >
        <button
          onClick={handleZoomIn}
          className="flex h-8 w-8 items-center justify-center rounded-md border border-border bg-background/90 text-foreground shadow-sm backdrop-blur transition-colors hover:bg-muted"
          title="Zoom in"
          aria-label="Zoom in"
        >
          <ZoomIn size={14} />
        </button>
        <button
          onClick={handleZoomOut}
          className="flex h-8 w-8 items-center justify-center rounded-md border border-border bg-background/90 text-foreground shadow-sm backdrop-blur transition-colors hover:bg-muted"
          title="Zoom out"
          aria-label="Zoom out"
        >
          <ZoomOut size={14} />
        </button>
        <button
          onClick={handleFitToView}
          className="flex h-8 w-8 items-center justify-center rounded-md border border-border bg-background/90 text-foreground shadow-sm backdrop-blur transition-colors hover:bg-muted"
          title="Fit to view"
          aria-label="Fit to view"
        >
          <Maximize2 size={14} />
        </button>
        <button
          onClick={handleBuildGraph}
          disabled={building}
          className="flex h-8 w-8 items-center justify-center rounded-md border border-border bg-background/90 text-foreground shadow-sm backdrop-blur transition-colors hover:bg-muted disabled:opacity-50"
          title="Rebuild graph"
          aria-label="Rebuild graph"
        >
          {building ? (
            <Loader2 size={14} className="animate-spin" />
          ) : (
            <RefreshCw size={14} />
          )}
        </button>
        <button
          onClick={() => setSidebarOpen(!sidebarOpen)}
          className="flex h-8 w-8 items-center justify-center rounded-md border border-border bg-background/90 text-foreground shadow-sm backdrop-blur transition-colors hover:bg-muted"
          title={sidebarOpen ? "Close sidebar" : "Open sidebar"}
          aria-label={sidebarOpen ? "Close sidebar" : "Open sidebar"}
        >
          {sidebarOpen ? <PanelRightClose size={14} /> : <PanelRightOpen size={14} />}
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
        {highlightedPath.length > 0 && (
          <button
            onClick={() => {
              setHighlightedPath([]);
              setPathNodeSet(new Set());
            }}
            className="flex h-8 items-center justify-center rounded-md border border-accent/50 bg-accent/10 px-2 text-[10px] text-accent shadow-sm backdrop-blur transition-colors hover:bg-accent/20"
            title="Clear path"
          >
            Clear Path
          </button>
        )}
      </div>

      {/* Filter Panel */}
      <div className="absolute left-4 top-4">
        <div className="rounded-md border border-border bg-background/90 shadow-sm backdrop-blur">
          <div className="flex items-center gap-1 px-3 py-2">
            <ContextualHelp topic="graph" placement="right" />
            <button
              onClick={() => setFilterOpen(!filterOpen)}
              className="flex items-center gap-2 text-xs font-medium text-foreground"
            >
            <Filter size={12} />
            Filters
              {filterOpen ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
            </button>
          </div>
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
              setSelectedNode(contextMenu.node);
              setPathFindingFrom(contextMenu.node.id);
              setSidebarOpen(true);
              setContextMenu(null);
              addToast("info", "Click another node to find the path");
            }}
            className="flex w-full items-center px-3 py-1.5 text-xs text-foreground hover:bg-muted"
          >
            Find Path To...
          </button>
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

      {/* Sidebar */}
      {sidebarOpen && (
        <GraphSidebar
          selectedNode={selectedNode}
          communities={communities}
          graphStats={graphStats}
          connectionCount={connectionCount}
          highlightedPath={highlightedPath}
          onFindPath={handleFindPath}
          onClose={() => setSidebarOpen(false)}
          onViewDocument={(nodeId) => {
            setSelectedDocument(nodeId);
            setActiveView("document-detail");
          }}
          allNodes={filteredData.nodes}
        />
      )}
    </div>
  );
}
