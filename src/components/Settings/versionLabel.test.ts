import { describe, expect, it } from "vitest";
import { formatAppVersion } from "./versionLabel";

describe("formatAppVersion", () => {
  it("formats a normal semantic version for display", () => {
    expect(formatAppVersion("1.1.0")).toBe("Version 1.1.0");
  });

  it("returns fallback text when version is empty", () => {
    expect(formatAppVersion("")).toBe("Version Unknown");
  });

  it("returns fallback text when version is missing or whitespace", () => {
    expect(formatAppVersion(undefined)).toBe("Version Unknown");
    expect(formatAppVersion("   ")).toBe("Version Unknown");
  });
});
