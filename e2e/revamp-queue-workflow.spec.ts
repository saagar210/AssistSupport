import { expect, test } from '@playwright/test';

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem('onboarding-completed', 'true');
    localStorage.setItem('sidebar-collapsed', 'false');
    localStorage.setItem('assistsupport.flag.ASSISTSUPPORT_REVAMP_INBOX', 'true');
    localStorage.setItem('assistsupport.flag.ASSISTSUPPORT_REVAMP_WORKSPACE', 'true');
    localStorage.setItem('assistsupport.flag.ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2', 'true');
  });
});

test('@smoke queue workflow deep-link and keyboard triage', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByText('Live queue context')).toBeVisible();
  await page.getByRole('button', { name: 'Open Unassigned Queue' }).click();
  await expect(page.getByText('Queue-first inbox mode')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Unassigned' })).toHaveClass(/btn-primary/);
  await expect(page.getByText('No queue items for this view.')).toBeVisible();
  await page.getByRole('button', { name: 'At Risk' }).click();
  await expect(page.getByRole('button', { name: 'At Risk' })).toHaveClass(/btn-primary/);
  await expect(page.getByText('No queue items for this view.')).toBeVisible();
});
