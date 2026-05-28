import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const mode = process.argv.includes("--full") ? "full" : "quick";
const listOnly = process.argv.includes("--list");

const quickCommands = [
  command("Browser workflow environment", "node", ["scripts/check-e2e-environment.mjs"]),
  command("Frontend typecheck", "pnpm", ["run", "check"]),
  command("Frontend unit tests", "pnpm", ["run", "test:unit"]),
  command("Project structure guard", "pnpm", ["run", "check:structure"]),
  command("Accessibility guard", "pnpm", ["run", "check:a11y"]),
  command("Dependency/license admission", "pnpm", ["run", "check:deps"]),
  command("External transform documentation contract", "pnpm", ["run", "check:external-transform-docs"]),
  command("AI-first roadmap contract", "pnpm", ["run", "check:ai-roadmap"]),
  command("AI provider evidence contract", "pnpm", ["run", "check:ai-provider"]),
  command("AI runtime evidence contract", "pnpm", ["run", "check:ai-runtime"]),
  command("Security review evidence contract", "pnpm", ["run", "check:security-review"]),
  command("Homebrew cask packaging contract", "pnpm", ["run", "check:homebrew"]),
  command("Release evidence workflow guard", "pnpm", ["run", "check:release-ci"]),
  command("Release candidate script syntax", "node", ["--check", "scripts/create-release-candidate.mjs"]),
  command("Markdown links", "pnpm", ["run", "check:docs"]),
  command("Spec completion matrix contract", "pnpm", ["run", "check:spec-completion"]),
  command("Table editor manual review contract", "pnpm", ["run", "check:tables:manual"]),
  command("Rust formatting", "cargo", ["fmt", "--check"], "src-tauri"),
  command("Rust dev check", "cargo", ["check", "--locked"], "src-tauri"),
  command("Whitespace check", "git", ["diff", "--check"]),
];

const fullCommands = [
  ...quickCommands,
  command("Frontend production build", "pnpm", ["run", "build"]),
  command("Browser workflow suite", "node", ["scripts/run-e2e.mjs"]),
  command("Accessibility runtime audit", "pnpm", ["run", "check:a11y:runtime"]),
  command("Accessibility manual review contract", "pnpm", ["run", "check:a11y:manual"]),
  command("Optional engine probe", "pnpm", ["run", "check:engines"]),
  command(
    "Rust native-watch check",
    "cargo",
    ["check", "--locked", "--features", "native-watch"],
    "src-tauri",
  ),
  command(
    "Rust clippy",
    "cargo",
    ["clippy", "--locked", "--all-targets", "--", "-D", "warnings"],
    "src-tauri",
  ),
  command("Rust tests", "cargo", ["test", "--locked"], "src-tauri"),
  command("Rendered export audit", "pnpm", ["run", "test:rendered-exports"]),
  command("Google Docs import evidence contract", "pnpm", ["run", "check:google-docs-import"]),
  command("AI provider live endpoint evidence contract", "pnpm", ["run", "check:ai-provider"]),
  command("Release device performance profile contract", "pnpm", ["run", "check:performance-profile"]),
  command("Platform package configuration", "pnpm", ["run", "check:platform-packaging"]),
  command("External platform evidence contract", "pnpm", ["run", "check:platform-evidence"]),
  command("Release signing evidence contract", "pnpm", ["run", "check:release-signing"]),
  command("Release evidence kit generation", "pnpm", ["run", "collect:evidence-kit"]),
  command("Release evidence kit contract", "pnpm", ["run", "check:evidence-kit"]),
  command(
    "Desktop release compile",
    "./node_modules/.bin/tauri",
    ["build", "--no-bundle"],
  ),
  ...platformBundleCommands(),
  command("Desktop artifact smoke", "pnpm", ["run", "test:desktop-smoke"]),
  ...desktopLaunchSmokeCommands(),
  command("Desktop WebDriver smoke", "pnpm", ["run", "test:tauri-webdriver"]),
  command("Release readiness aggregation", "pnpm", ["run", "check:release-readiness"]),
];

const commands = mode === "full" ? fullCommands : quickCommands;

if (listOnly) {
  console.log(`NEditor local verification (${mode}) will run:`);
  for (const item of commands) {
    console.log(`- ${item.label}: ${formatCommand(item)}`);
  }
  process.exit(0);
}

console.log(`Running NEditor local verification (${mode}) with ${commands.length} steps.`);
for (const item of commands) {
  console.log(`\n==> ${item.label}`);
  console.log(`$ ${formatCommand(item)}`);
  const result = spawnSync(item.cmd, item.args, {
    cwd: item.cwd,
    env: { ...process.env, ...item.env },
    shell: process.platform === "win32",
    stdio: "inherit",
  });
  if (result.status !== 0) {
    const code = result.status ?? 1;
    console.error(`\nLocal verification failed at "${item.label}" with exit code ${code}.`);
    process.exit(code);
  }
}

console.log(`\nNEditor local verification (${mode}) passed.`);

function command(label, cmd, args, cwd = ".", env = {}) {
  return {
    label,
    cmd,
    args,
    cwd: join(root, cwd),
    env,
  };
}

function platformBundleCommands() {
  if (process.platform !== "darwin") return [];
  return [
    command("Desktop macOS app bundle", "./node_modules/.bin/tauri", ["build", "--bundles", "app"]),
    command("Desktop bundle smoke", "pnpm", ["run", "test:desktop-bundle"]),
    command("Desktop DMG classification", "pnpm", ["run", "test:desktop-dmg"]),
  ];
}

function desktopLaunchSmokeCommands() {
  if (process.platform !== "darwin") return [];
  return [
    command("Desktop macOS GUI launch smoke", "pnpm", ["run", "test:desktop-smoke"], ".", {
      NEDITOR_DESKTOP_SMOKE_LAUNCH: "1",
    }),
  ];
}

function formatCommand(item) {
  const relativeCwd = item.cwd === root ? "." : item.cwd.slice(root.length + 1);
  const envPrefix = Object.entries(item.env || {})
    .map(([key, value]) => `${key}=${value}`)
    .join(" ");
  const rendered = [envPrefix, item.cmd, ...item.args].filter(Boolean).join(" ");
  return relativeCwd === "." ? rendered : `(cd ${relativeCwd} && ${rendered})`;
}
