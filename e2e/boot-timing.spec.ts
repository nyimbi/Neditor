/**
 * Boot timing regression gate.
 *
 * Asserts that the editor is painted within the cold-start budget:
 *   macOS / Linux : editorReady - bootStart < 500 ms
 *   Windows       : editorReady - bootStart < 900 ms
 *
 * Skip this spec by setting NEDITOR_SKIP_BOOT_TIMING=1 in the environment
 * (useful for slow CI runners where timing is unreliable).
 *
 * The test navigates to the Vite dev server (configured in playwright.config.ts
 * as baseURL), waits for `window.__neditor_boot.editorReady` to appear, then
 * reads the full timing object via `window.__neditor_boot`.
 */

import { expect, test } from "@playwright/test";

const SKIP = process.env.NEDITOR_SKIP_BOOT_TIMING === "1";

const BUDGET_MS =
  process.platform === "win32"
    ? 900
    : 500;

test.describe("boot timing", () => {
  test.skip(SKIP, "NEDITOR_SKIP_BOOT_TIMING=1 — skipped");

  test(`editor ready within ${BUDGET_MS} ms on ${process.platform}`, async ({ page }) => {
    await page.goto("/");

    // Wait up to 10 s for the editor to signal readiness.
    await page.waitForFunction(
      () =>
        typeof window.__neditor_boot?.editorReady === "number",
      { timeout: 10_000 },
    );

    const timing = await page.evaluate(() => window.__neditor_boot);

    const elapsed = timing.editorReady! - timing.bootStart;

    console.log(
      `[boot-timing] bootStart→editorReady: ${elapsed.toFixed(1)} ms` +
      ` (budget: ${BUDGET_MS} ms, platform: ${process.platform})`,
    );

    if (timing.mountStart !== undefined) {
      const mountLag = timing.mountStart - timing.bootStart;
      console.log(`[boot-timing] bootStart→mountStart (JS parse + Vue init): ${mountLag.toFixed(1)} ms`);
    }

    expect(elapsed, `boot elapsed ${elapsed.toFixed(0)} ms must be ≤ ${BUDGET_MS} ms`).toBeLessThanOrEqual(BUDGET_MS);
  });

  test("window.__neditor_boot has required keys after boot", async ({ page }) => {
    await page.goto("/");

    await page.waitForFunction(
      () => typeof window.__neditor_boot?.editorReady === "number",
      { timeout: 10_000 },
    );

    const keys = await page.evaluate(() => Object.keys(window.__neditor_boot));
    expect(keys).toContain("bootStart");
    expect(keys).toContain("editorReady");
  });
});
