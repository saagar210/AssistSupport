import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";

const defaultBlockingImpacts = ["critical", "serious"];
const blockingImpacts = (process.env.UI_A11Y_BLOCKING_IMPACTS ?? defaultBlockingImpacts.join(","))
  .split(",")
  .map((impact) => impact.trim())
  .filter(Boolean);

function formatBlockingViolations(
  violations: Array<{ id: string; impact?: string | null; nodes: Array<{ target: string[] }> }>,
): string {
  if (violations.length === 0) return "No blocking axe violations.";
  return violations
    .map((violation) => {
      const firstTarget = violation.nodes[0]?.target?.join(" > ") ?? "unknown target";
      return `${violation.id} (${violation.impact ?? "unknown"}) at ${firstTarget}`;
    })
    .join("\n");
}

test("@a11y home page", async ({ page }) => {
  await page.goto("/");
  await page.addStyleTag({
    content: `
      *,*::before,*::after{
        animation:none !important;
        transition:none !important;
      }
      .app-shell-revamp .draft-tab--revamp{
        opacity:1 !important;
        transform:none !important;
      }
    `,
  });
  await page.waitForTimeout(50);
  const results = await new AxeBuilder({ page }).analyze();
  const blockingViolations = results.violations.filter((violation) =>
    blockingImpacts.includes(violation.impact ?? ""),
  );
  expect(blockingViolations, formatBlockingViolations(blockingViolations)).toEqual([]);
});
