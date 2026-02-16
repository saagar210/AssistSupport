import { expect, test } from "@playwright/test";

test("@visual home page", async ({ page }) => {
  await page.goto("/");
  await page.addStyleTag({
    content:
      "*,*::before,*::after{animation:none!important;transition:none!important;caret-color:transparent!important;}",
  });
  const maxDiffPixelRatio = 0.035;
  await expect(page.locator("body")).toHaveScreenshot("home-body.png", {
    animations: "disabled",
    maxDiffPixelRatio,
  });
});
