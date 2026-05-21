import { existsSync } from "node:fs";

process.env.PLAYWRIGHT_BROWSERS_PATH ??= "0";

const installCommand = "PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright install chromium";
const { chromium } = await import("@playwright/test");
const executablePath = chromium.executablePath();

if (!existsSync(executablePath)) {
  fail(
    "Playwright Chromium is not installed in the project-local browser cache.",
    [
      `Expected executable: ${executablePath}`,
      `Install it with: ${installCommand}`,
      "Then rerun: pnpm run check:e2e-env",
    ],
  );
}

let browser;
try {
  browser = await chromium.launch({ headless: true });
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  if (
    message.includes("bootstrap_check_in") ||
    message.includes("MachPortRendezvous") ||
    message.includes("Permission denied (1100)")
  ) {
    fail(
      "Playwright Chromium is installed, but this macOS host blocks Chromium launch.",
      [
        `Executable: ${executablePath}`,
        "Observed macOS Mach bootstrap permission denial.",
        "Run browser workflows from a normal terminal/session that permits Chromium launch.",
      ],
    );
  }
  if (message.includes("Executable doesn't exist") || message.includes("Looks like Playwright")) {
    fail(
      "Playwright Chromium could not be launched because the browser executable is missing.",
      [
        `Expected executable: ${executablePath}`,
        `Install it with: ${installCommand}`,
      ],
    );
  }
  fail("Playwright Chromium launch failed.", [message]);
} finally {
  await browser?.close();
}

console.log(`Playwright Chromium launch preflight passed: ${executablePath}`);

function fail(summary, details) {
  console.error(`E2E environment check failed: ${summary}`);
  for (const detail of details) {
    console.error(`- ${detail}`);
  }
  process.exit(1);
}
