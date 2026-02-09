export const FILE_TYPE_COLORS: Record<string, string> = {
  pdf: "#ef4444",
  md: "#22c55e",
  markdown: "#22c55e",
  html: "#a855f7",
  txt: "#6b7280",
  docx: "#f97316",
  csv: "#eab308",
  epub: "#ec4899",
};

export function getFileTypeColor(fileType: string): string {
  return FILE_TYPE_COLORS[fileType.toLowerCase()] ?? "#6b7280";
}

export const FILE_TYPE_BADGE_COLORS: Record<string, string> = {
  pdf: "bg-red-500",
  md: "bg-green-500",
  markdown: "bg-green-500",
  html: "bg-purple-500",
  txt: "bg-gray-500",
  docx: "bg-orange-500",
  csv: "bg-yellow-500",
  epub: "bg-pink-500",
};

export function getFileTypeBadgeColor(fileType: string): string {
  return FILE_TYPE_BADGE_COLORS[fileType.toLowerCase()] ?? "bg-gray-500";
}
