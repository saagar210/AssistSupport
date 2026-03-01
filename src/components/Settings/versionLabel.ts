export function formatAppVersion(version: string | null | undefined): string {
  if (typeof version !== "string" || version.trim().length === 0) {
    return "Version Unknown";
  }
  return `Version ${version}`;
}
