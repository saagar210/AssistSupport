import { expect, test } from "@playwright/test";

test("@visual home page", async ({ page }, testInfo) => {
  await page.goto("/");
  await page.addStyleTag({
    content:
      "*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important;}",
  });
  const maxDiffPixelRatio = testInfo.project.name === "mobile" ? 0.12 : 0.05;
  await expect(page.locator("body")).toHaveScreenshot("home-body.png", {
    animations: "disabled",
    maxDiffPixelRatio,
  });
});
