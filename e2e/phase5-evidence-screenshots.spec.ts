import fs from 'node:fs';
import path from 'node:path';
import { expect, test } from '@playwright/test';

function ensureDir(dir: string) {
  fs.mkdirSync(dir, { recursive: true });
}

async function captureCoreScreenshots(page, outDir: string) {
  ensureDir(outDir);

  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto('/');

  // Draft is the core workbench; keep a deterministic "generated" state for evidence.
  await page
    .getByPlaceholder('Paste ticket content or describe the issue...')
    .fill('Evidence capture: VPN access issue [phase5-evidence]');
  await page.getByRole('button', { name: 'Generate Full Response' }).click();
  await expect(
    page.getByText(/Per Remote Work Policy, use the approved VPN and complete MFA/i),
  ).toBeVisible();

  await page.screenshot({ path: path.join(outDir, 'draft-generated.png') });

  // Sources panel (trust surface).
  const sourcesToggle = page.getByRole('button', { name: /Hide Sources|Sources \\(\\d+\\)/i });
  await expect(sourcesToggle).toBeVisible();
  const sourcesToggleText = (await sourcesToggle.textContent()) ?? '';
  // The app may auto-open sources for freshly generated answers. Only click when we need to open.
  if (/Sources \\(\\d+\\)/i.test(sourcesToggleText)) {
    await sourcesToggle.click();
  }
  await expect(page.getByRole('heading', { name: /Knowledge Base Sources/i })).toBeVisible();
  await page.screenshot({ path: path.join(outDir, 'draft-sources-open.png') });

  // Settings (AI + policy posture visibility).
  await page.getByRole('button', { name: 'Settings' }).click();
  await expect(page.getByText(/Deployment & Integrations/i)).toBeVisible();
  await page.screenshot({ path: path.join(outDir, 'settings.png') });
}

test.describe('Phase 5 evidence screenshots', () => {
  test('baseline (revamp shell off)', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('onboarding-completed', 'true');
      localStorage.setItem('sidebar-collapsed', 'false');
      localStorage.removeItem('assistsupport.flag.ASSISTSUPPORT_REVAMP_APP_SHELL');
      localStorage.removeItem('assistsupport.flag.ASSISTSUPPORT_REVAMP_INBOX');
      localStorage.removeItem('assistsupport.flag.ASSISTSUPPORT_REVAMP_WORKSPACE');
      localStorage.removeItem('assistsupport.flag.ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2');
    });

    const outDir = path.join(
      process.cwd(),
      'docs',
      'revamp',
      'evidence',
      'phase5',
      'baseline',
      'screenshots',
    );
    await captureCoreScreenshots(page, outDir);
  });

  test('ux-6 (revamp shell on)', async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem('onboarding-completed', 'true');
      localStorage.setItem('sidebar-collapsed', 'false');
      localStorage.setItem('assistsupport.flag.ASSISTSUPPORT_REVAMP_APP_SHELL', 'true');
      localStorage.setItem('assistsupport.flag.ASSISTSUPPORT_REVAMP_INBOX', 'true');
      localStorage.setItem('assistsupport.flag.ASSISTSUPPORT_REVAMP_WORKSPACE', 'true');
      localStorage.setItem('assistsupport.flag.ASSISTSUPPORT_REVAMP_COMMAND_PALETTE_V2', 'true');
    });

    const outDir = path.join(
      process.cwd(),
      'docs',
      'revamp',
      'evidence',
      'phase5',
      'ux-6',
      'screenshots',
    );
    await captureCoreScreenshots(page, outDir);
  });
});
