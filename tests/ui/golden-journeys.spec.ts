import { expect, test } from "@playwright/test";

test("@journey draft generation can be handed off to the follow-up queue", async ({ page }) => {
  const incidentSummary = "VPN access denied after MFA reset";

  await page.goto("/");
  await expect(page.locator(".app")).toBeVisible({ timeout: 20_000 });

  await page.getByPlaceholder("Paste ticket content or describe the issue...").fill(incidentSummary);
  await page.getByRole("button", { name: "Generate Full Response" }).click();

  const responseTextarea = page.getByRole("textbox", {
    name: "Response will appear here...",
    exact: true,
  });
  await expect(responseTextarea).toHaveValue(/approved VPN/i);

  await page.getByRole("button", { name: "Save Draft" }).first().click();
  await expect(page.getByText("Draft saved")).toBeVisible();

  await page.getByRole("button", { name: "Follow-ups" }).click();
  await expect(page.getByRole("heading", { name: "Queue Command Center" })).toBeVisible();
  await expect(page.locator("[data-testid='queue-items-list'] li")).toHaveCount(1);

  await page.getByRole("button", { name: "Open In Draft" }).click();
  await expect(page.getByPlaceholder("Paste ticket content or describe the issue...")).toHaveValue(
    incidentSummary,
  );
});

test("@journey settings preflight keeps policy and operational feedback visible", async ({
  page,
}) => {
  await page.goto("/");
  await expect(page.locator(".app")).toBeVisible({ timeout: 20_000 });

  await page.getByRole("button", { name: "Settings" }).click();
  await expect(page.getByRole("heading", { name: "Operator console" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "MemoryKernel" })).toBeVisible();
  await expect(page.getByText("expected service.v2")).toBeVisible();

  await page.getByRole("button", { name: "Run Deployment Preflight" }).click();
  await expect(page.getByText("Database integrity: pass")).toBeVisible();
  await expect(page.getByText("Model status: loaded")).toBeVisible();
});
