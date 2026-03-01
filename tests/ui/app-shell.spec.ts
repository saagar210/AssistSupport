import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

test("@smoke @visual app shell renders with mocked Tauri bridge", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(".app")).toBeVisible({ timeout: 20_000 });
  await expect(page.getByText("Application Error")).toHaveCount(0);
  await expect(page).toHaveScreenshot("app-shell.png", {
    fullPage: true,
    animations: "disabled",
    maxDiffPixels: 150,
  });
});

test("@smoke @a11y app shell has no critical accessibility violations", async ({ page }) => {
  await page.goto("/");
  await expect(page.locator(".app")).toBeVisible({ timeout: 20_000 });

  const report = await new AxeBuilder({ page }).analyze();
  const seriousViolations = report.violations.filter((violation) =>
    ["critical", "serious"].includes(violation.impact ?? ""),
  );

  expect(seriousViolations, JSON.stringify(seriousViolations, null, 2)).toEqual([]);
});
