import { useState } from "react";
import { ChevronDown, ChevronUp } from "lucide-react";
import { FILE_TYPE_COLORS } from "../utils/fileTypeColors";

export function GraphLegend() {
  const [collapsed, setCollapsed] = useState(false);

  return (
    <div className="absolute bottom-4 left-4 rounded-md border border-border bg-background/90 shadow-sm backdrop-blur">
      <button
        onClick={() => setCollapsed(!collapsed)}
        className="flex w-full items-center justify-between px-3 py-1.5 text-xs font-medium text-foreground"
      >
        <span>Legend</span>
        {collapsed ? <ChevronUp size={12} /> : <ChevronDown size={12} />}
      </button>
      {!collapsed && (
        <div className="border-t border-border px-3 py-2 space-y-2">
          <div>
            <p className="mb-1 text-[10px] font-medium uppercase text-muted-foreground">
              File Types
            </p>
            <div className="grid grid-cols-2 gap-x-3 gap-y-0.5">
              {Object.entries(FILE_TYPE_COLORS).map(([type, color]) => (
                <div key={type} className="flex items-center gap-1.5">
                  <span
                    className="inline-block h-2.5 w-5 rounded-sm"
                    style={{ backgroundColor: color }}
                  />
                  <span className="text-[10px] text-foreground">
                    {type.toUpperCase()}
                  </span>
                </div>
              ))}
            </div>
          </div>
          <div>
            <p className="mb-1 text-[10px] font-medium uppercase text-muted-foreground">
              Edges
            </p>
            <div className="space-y-0.5">
              <div className="flex items-center gap-1.5">
                <span className="inline-block h-0.5 w-5 bg-slate-400" />
                <span className="text-[10px] text-foreground">Similar content</span>
              </div>
              <div className="flex items-center gap-1.5">
                <span className="inline-block h-0.5 w-5 border-t-2 border-dashed border-slate-400" />
                <span className="text-[10px] text-foreground">Same document</span>
              </div>
            </div>
          </div>
          <div>
            <p className="text-[10px] text-muted-foreground">
              Node size = chunk count
            </p>
          </div>
        </div>
      )}
    </div>
  );
}
