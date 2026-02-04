import { expect, test, type Page } from '@playwright/test';

async function clickSidebarTab(page: Page, label: string) {
  const tab = page.locator('aside.sidebar button', { hasText: label }).first();
  await tab.click();
}

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem('onboarding-completed', 'true');
    localStorage.setItem('sidebar-collapsed', 'false');
  });
});

test('@smoke exercises ops workflows', async ({ page }) => {
  await page.goto('/');

  await clickSidebarTab(page, 'Ops');
  await expect(page.getByRole('heading', { name: 'Deployment Rollback & Signed Pack Verification' })).toBeVisible();

  await page.getByRole('button', { name: 'Run Preflight' }).click();
  await expect(page.getByText('Database integrity: pass')).toBeVisible();

  await page.getByRole('button', { name: 'Verify' }).first().click();
  await expect(page.getByText('Verification: verified')).toBeVisible();

  await page.getByRole('button', { name: 'Roll Back Last Run' }).click();
  await expect(page.getByRole('dialog', { name: 'Confirm rollback' })).toBeVisible();
  await page.getByRole('button', { name: 'Confirm Rollback' }).click();
  await expect(page.getByText('Last status: rolled_back')).toBeVisible();

  await page.getByRole('button', { name: 'Eval Harness' }).click();
  await page.getByRole('button', { name: 'Run Eval' }).click();
  await expect(page.getByText(/Run eval-/)).toBeVisible();

  await page.getByRole('button', { name: 'Triage' }).click();
  await page.getByRole('button', { name: 'Cluster Tickets' }).click();
  await expect(page.locator('.ops-pre')).toContainText('cluster_key');

  await page.getByRole('button', { name: 'Runbook' }).click();
  await page.getByRole('button', { name: 'Start Runbook' }).click();
  await expect(page.getByText('security-incident').first()).toBeVisible();

  await page.getByRole('button', { name: 'Integrations' }).click();
  await expect(page.locator('.ops-card-title', { hasText: 'servicenow' }).first()).toBeVisible();
  await expect(page.locator('.ops-card-title', { hasText: 'slack' }).first()).toBeVisible();
  await expect(page.locator('.ops-card-title', { hasText: 'teams' }).first()).toBeVisible();
});
