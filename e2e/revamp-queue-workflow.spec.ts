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

  await page.getByPlaceholder('Paste ticket content or describe the issue...').fill('Sev2 onboarding access issue: cannot access VPN after MFA');
  await page.getByRole('button', { name: 'Save Draft' }).first().click();

  await expect(page.getByText('Live queue context')).toBeVisible();
  await page.getByRole('button', { name: 'Open Unassigned Queue' }).click();
  await expect(page.getByText('Queue-first inbox mode')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Unassigned' })).toHaveClass(/btn-primary/);
  const queueList = page.locator('[data-testid="queue-items-list"]');
  await expect(queueList.getByText(/Sev2 onboarding access issue/i).first()).toBeVisible();

  await page.getByRole('button', { name: 'All' }).click();
  await expect(page.getByRole('button', { name: 'All' })).toHaveClass(/btn-primary/);

  await expect(queueList).toBeVisible();
  await queueList.focus();

  await page.keyboard.press('c');
  await expect(page.getByText(/Owner:\s*current-operator/i).first()).toBeVisible();

  await page.keyboard.press('x');
  await expect(page.getByRole('button', { name: 'Reopen' }).first()).toBeVisible();

  await page.keyboard.press('o');
  await expect(page.getByRole('button', { name: 'Resolve' }).first()).toBeVisible();
});
