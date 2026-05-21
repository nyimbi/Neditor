import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { chromium } from "@playwright/test";

const systemChromiumCandidates = {
  darwin: [
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
  ],
  linux: [
    "/usr/bin/google-chrome",
    "/usr/bin/google-chrome-stable",
    "/usr/bin/chromium",
    "/usr/bin/chromium-browser",
    "/usr/bin/microsoft-edge",
  ],
  win32: [
    "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
    "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
    "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
    "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
  ],
};

export function resolvePlaywrightBrowserEnv(baseEnv = process.env) {
  const env = {
    ...baseEnv,
    PLAYWRIGHT_BROWSERS_PATH: baseEnv.PLAYWRIGHT_BROWSERS_PATH ?? "0",
  };
  const explicitPath = env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH?.trim();
  if (explicitPath) {
    if (!existsSync(explicitPath)) {
      return {
        ok: false,
        env,
        source: "missing-explicit",
        executablePath: explicitPath,
        expectedBundledPath: expectedBundledChromiumPath(),
        message: `PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH points to a missing executable: ${explicitPath}`,
      };
    }
    return {
      ok: true,
      env,
      source: "explicit",
      executablePath: explicitPath,
      expectedBundledPath: expectedBundledChromiumPath(),
      message: `Using explicit Chromium executable at ${explicitPath}.`,
    };
  }

  const expectedBundledPath = expectedBundledChromiumPath(env);
  if (expectedBundledPath && existsSync(expectedBundledPath)) {
    return {
      ok: true,
      env,
      source: "playwright-bundled",
      executablePath: expectedBundledPath,
      expectedBundledPath,
      message: `Using Playwright bundled Chromium at ${expectedBundledPath}.`,
    };
  }

  const fallbackPath = findSystemChromium();
  if (fallbackPath) {
    env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH = fallbackPath;
    env.NEDITOR_E2E_BROWSER_SOURCE = "system-chromium";
    return {
      ok: true,
      env,
      source: "system-chromium",
      executablePath: fallbackPath,
      expectedBundledPath,
      message: `Playwright bundled Chromium is missing; using system Chromium-compatible browser at ${fallbackPath}.`,
    };
  }

  return {
    ok: false,
    env,
    source: "missing",
    executablePath: "",
    expectedBundledPath,
    message: [
      "No Chromium executable is available for browser workflows.",
      expectedBundledPath ? `Expected Playwright bundled Chromium at ${expectedBundledPath}.` : "",
      "Run PLAYWRIGHT_BROWSERS_PATH=0 pnpm exec playwright install chromium,",
      "or install Google Chrome/Chromium,",
      "or set PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH to a compatible browser executable.",
    ]
      .filter(Boolean)
      .join(" "),
  };
}

export function writePlaywrightBrowserReport(reportPath, resolution, status, extra = {}) {
  const report = {
    generatedAt: new Date().toISOString(),
    status,
    platform: process.platform,
    browser: {
      source: resolution.source,
      executablePath: resolution.executablePath,
      expectedBundledPath: resolution.expectedBundledPath,
      message: resolution.message,
    },
    ...extra,
  };
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(reportPath, `${JSON.stringify(report, null, 2)}\n`);
}

function expectedBundledChromiumPath(env = process.env) {
  const original = process.env.PLAYWRIGHT_BROWSERS_PATH;
  if (env.PLAYWRIGHT_BROWSERS_PATH !== undefined) {
    process.env.PLAYWRIGHT_BROWSERS_PATH = env.PLAYWRIGHT_BROWSERS_PATH;
  }
  try {
    return chromium.executablePath();
  } finally {
    if (original === undefined) {
      delete process.env.PLAYWRIGHT_BROWSERS_PATH;
    } else {
      process.env.PLAYWRIGHT_BROWSERS_PATH = original;
    }
  }
}

function findSystemChromium() {
  const candidates = systemChromiumCandidates[process.platform] || [];
  return candidates.map((candidate) => resolve(candidate)).find((candidate) => existsSync(candidate)) || "";
}
