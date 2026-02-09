import { expect, test } from '@playwright/test';

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    localStorage.setItem('onboarding-completed', 'true');
    localStorage.setItem('sidebar-collapsed', 'false');
    localStorage.setItem('assistsupport.flag.ASSISTSUPPORT_REVAMP_APP_SHELL', 'true');
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
  await expect(page.getByText('Queue Command Center')).toBeVisible();
  const queueList = page.locator('[data-testid="queue-items-list"]');
  await expect(queueList.getByText(/Sev2 onboarding access issue/i).first()).toBeVisible();

  await page.getByRole('button', { name: 'All' }).click();

  await expect(queueList).toBeVisible();
  await queueList.focus();

  await page.keyboard.press('c');
  await expect(page.getByText(/Owner:\s*current-operator/i).first()).toBeVisible();

  await page.keyboard.press('x');
  await expect(page.getByRole('button', { name: 'Reopen' }).first()).toBeVisible();

  await page.keyboard.press('o');
  await expect(page.getByRole('button', { name: 'Resolve' }).first()).toBeVisible();
});

test('@smoke at-risk queue opens draft and generates response', async ({ page }) => {
  await page.goto('/');

  await page.getByPlaceholder('Paste ticket content or describe the issue...').fill(
    'Sev1 SSO outage affecting all onboarding users [e2e-at-risk]',
  );
  await page.getByRole('button', { name: 'Save Draft' }).first().click();

  await expect(page.getByText('Live queue context')).toBeVisible();
  await page.getByRole('button', { name: 'Open At-Risk Queue' }).click();
  await expect(page.getByText('Queue Command Center')).toBeVisible();

  const queueList = page.locator('[data-testid="queue-items-list"]');
  await expect(queueList).toBeVisible();
  await expect(queueList.getByText(/Sev1 SSO outage affecting all onboarding users/i).first()).toBeVisible();

  await page.getByRole('button', { name: 'Open Draft' }).first().click();
  await expect(page.getByPlaceholder('Paste ticket content or describe the issue...')).toHaveValue(
    /Sev1 SSO outage affecting all onboarding users/i,
  );

  await page.getByRole('button', { name: 'Generate Full Response' }).click();
  await expect(
    page.getByText(/Per Remote Work Policy, use the approved VPN and complete MFA/i),
  ).toBeVisible();
  await expect(page.getByRole('button', { name: /Hide Sources|Sources \(1\)/i })).toBeVisible();
});

test('@smoke draft handoff enforces copy override when citations missing', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);

  await page.goto('/');

  const input = page.getByPlaceholder('Paste ticket content or describe the issue...');
  await input.fill('Test response without citations [e2e-no-citations]');

  await page.getByRole('button', { name: 'Generate Full Response' }).click();
  await expect(page.getByText(/cannot provide a confident response without citations/i)).toBeVisible();

  // "Copy" can appear in multiple places (response actions, templates, etc). Target the response actions strip.
  await page.locator('.response-actions').getByRole('button', { name: 'Copy' }).click();
  await expect(page.getByRole('dialog', { name: 'Copy override' })).toBeVisible();

  await page.getByPlaceholder('Explain why copying without citations is acceptable here.').fill(
    'E2E validation: confirming override gating works in revamp shell.',
  );

  await page.getByRole('button', { name: 'Copy with override' }).click();
  await expect(page.getByText('Response copied (override logged)')).toBeVisible();
});

test('@smoke draft handoff allows copy when citations are present', async ({ page, context }) => {
  await context.grantPermissions(['clipboard-read', 'clipboard-write']);

  await page.goto('/');

  const input = page.getByPlaceholder('Paste ticket content or describe the issue...');
  await input.fill('Test response with citations [e2e-citations-ok]');

  await page.getByRole('button', { name: 'Generate Full Response' }).click();
  await expect(
    page.getByText(/Per Remote Work Policy, use the approved VPN and complete MFA/i),
  ).toBeVisible();
  await expect(page.getByRole('button', { name: /Hide Sources|Sources \\(1\\)/i })).toBeVisible();

  const responseActions = page.locator('.response-actions');
  const copyButton = responseActions.getByRole('button', { name: 'Copy' });
  await copyButton.click();

  // No override dialog should appear when citations exist and confidence is "answer".
  await expect(page.getByRole('dialog', { name: 'Copy override' })).toHaveCount(0);

  // Successful copy updates the button label briefly.
  await expect(responseActions.getByRole('button', { name: 'Copied!' })).toBeVisible();
});
