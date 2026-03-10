import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { freezeAppClock } from "./support/freezeAppClock";

test("@smoke @visual app shell renders with mocked Tauri bridge", async ({ page }) => {
  await freezeAppClock(page);
  await page.goto("/");
  const appShell = page.locator(".app");
  await expect(appShell).toBeVisible({ timeout: 20_000 });
  await expect(page.getByText("Application Error")).toHaveCount(0);
  await expect(appShell).toHaveScreenshot("app-shell.png", {
    animations: "disabled",
  });
});

test("@smoke @a11y app shell has no critical accessibility violations", async ({ page }) => {
  await freezeAppClock(page);
  await page.goto("/");
  await expect(page.locator(".app")).toBeVisible({ timeout: 20_000 });

  const report = await new AxeBuilder({ page }).analyze();
  const seriousViolations = report.violations.filter((violation) =>
    ["critical", "serious"].includes(violation.impact ?? ""),
  );

  expect(seriousViolations, JSON.stringify(seriousViolations, null, 2)).toEqual([]);
});

test("@smoke @responsive app shell stays usable on a narrow viewport", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await freezeAppClock(page);
  await page.goto("/");

  const appShell = page.locator(".app");
  await expect(appShell).toBeVisible({ timeout: 20_000 });
  await expect(page.getByText("Application Error")).toHaveCount(0);
  await expect(page.getByText("Draft Workbench")).toBeVisible();

  const hasHorizontalOverflow = await page.evaluate(() => {
    const tolerance = 1;
    return document.documentElement.scrollWidth > (window.innerWidth + tolerance);
  });
  expect(hasHorizontalOverflow).toBe(false);
});
