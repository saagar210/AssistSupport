import { test, expect, type Page } from "@playwright/test";
import { freezeAppClock } from "./support/freezeAppClock";

async function expectNoHorizontalOverflow(page: Page) {
  const hasHorizontalOverflow = await page.evaluate(() => {
    const tolerance = 1;
    return document.documentElement.scrollWidth > (window.innerWidth + tolerance);
  });
  expect(hasHorizontalOverflow).toBe(false);
}

async function navigateToTab(page: Page, label: "Sources" | "Settings") {
  const mobileTabBar = page.locator(".tab-bar");
  if (await mobileTabBar.isVisible()) {
    await mobileTabBar.getByRole("button", { name: label, exact: true }).click();
    return "tab-bar" as const;
  }

  const revampNav = page.locator('.as-shell__nav[aria-label="Primary navigation"]');
  if (await revampNav.isVisible()) {
    await revampNav.getByRole("button", { name: label, exact: true }).click();
    return "revamp-nav" as const;
  }

  const legacySidebar = page.locator(".sidebar");
  if (await legacySidebar.isVisible()) {
    await legacySidebar.getByRole("button", { name: new RegExp(`^${label}`) }).click();
    return "sidebar" as const;
  }

  return null;
}

test("@smoke @responsive desktop shell keeps navigation and content in sync", async ({ page }) => {
  await freezeAppClock(page);
  await page.setViewportSize({ width: 1440, height: 960 });
  await page.goto("/");

  await expect(page.locator(".app")).toBeVisible({ timeout: 20_000 });
  await expect(page.getByText("Application Error")).toHaveCount(0);

  const desktopNav = await navigateToTab(page, "Sources");
  expect(desktopNav).not.toBeNull();
  await expect(page.getByRole("heading", { name: "Knowledge Base" })).toBeVisible();

  await navigateToTab(page, "Settings");
  await expect(page.getByRole("heading", { name: "Operator console" })).toBeVisible();

  await expectNoHorizontalOverflow(page);
});

test("@smoke @responsive mobile shell keeps tab-bar navigation usable across tabs", async ({ page }) => {
  await freezeAppClock(page);
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/");

  await expect(page.locator(".app")).toBeVisible({ timeout: 20_000 });
  await expect(page.getByText("Application Error")).toHaveCount(0);
  const mobileNav = await navigateToTab(page, "Sources");

  if (mobileNav) {
    await expect(page.getByRole("heading", { name: "Knowledge Base" })).toBeVisible();
    await expectNoHorizontalOverflow(page);

    await navigateToTab(page, "Settings");
    await expect(page.getByRole("heading", { name: "Operator console" })).toBeVisible();
  } else {
    await expect(page.getByText("Draft Workbench")).toBeVisible();
    await expect(page.locator(".as-shell__topbar, .header")).toBeVisible();
  }

  await expectNoHorizontalOverflow(page);
});
