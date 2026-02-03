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

test('@smoke loads draft workspace', async ({ page }) => {
  await page.goto('/');

  await expect(page.getByPlaceholder('Paste ticket content or describe the issue...')).toBeVisible();
  await expect(page.getByText('Model ready')).toBeVisible();
});

test('@smoke navigates to settings and hybrid search', async ({ page }) => {
  await page.goto('/');

  await clickSidebarTab(page, 'Settings');
  await expect(page.getByRole('heading', { name: 'Appearance' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Language Model' })).toBeVisible();

  await clickSidebarTab(page, 'Search');
  await expect(page.getByRole('heading', { name: 'Hybrid Search' }).first()).toBeVisible();

  await page.getByPlaceholder('Ask about policies, procedures, or search for information...').fill('Can I use a flash drive?');
  await page.locator('.hybrid-search-form').getByRole('button', { name: 'Search', exact: true }).click();
  await expect(page.getByText('Removable Media Policy')).toBeVisible();
});

test('@smoke generates a draft response in mock mode', async ({ page }) => {
  await page.goto('/');

  const input = page.getByPlaceholder('Paste ticket content or describe the issue...');
  await input.fill('Customer needs remote VPN access while traveling.');

  await page.getByRole('button', { name: 'Generate' }).first().click();

  await expect(
    page.getByText('Per Remote Work Policy, use the approved VPN and complete MFA before accessing internal systems.')
  ).toBeVisible();
});
