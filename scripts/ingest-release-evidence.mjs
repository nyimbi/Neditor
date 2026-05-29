import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, statSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const args = parseArgs(process.argv.slice(2));
const sourceDir = resolve(args.source || process.env.NEDITOR_RELEASE_EVIDENCE_RETURN_DIR || join(root, ".tmp", "release-evidence-return"));
const reportPath = resolve(args.report || join(root, ".tmp", "release-evidence-ingest", "report.json"));
const dryRun = Boolean(args["dry-run"]);
const listOnly = Boolean(args.list);
const validate = args.validate !== false && args["no-validate"] !== true && !dryRun;

const evidenceItems = [
  item("platform-win32-package", ".tmp/platform-evidence/external/win32/package-artifacts.json", "platform", [
    "platform-evidence/external/win32/package-artifacts.json",
    "platform/win32/package-artifacts.json",
    "platform/win32-package-artifacts.json",
    "win32/package-artifacts.json",
    "win32-package-artifacts.json",
  ]),
  item("platform-win32-webdriver", ".tmp/platform-evidence/external/win32/tauri-webdriver-report.json", "platform", [
    "platform-evidence/external/win32/tauri-webdriver-report.json",
    "platform/win32/tauri-webdriver-report.json",
    "platform/win32-tauri-webdriver-report.json",
    "win32/tauri-webdriver-report.json",
    "win32-tauri-webdriver-report.json",
  ]),
  item("platform-linux-package", ".tmp/platform-evidence/external/linux/package-artifacts.json", "platform", [
    "platform-evidence/external/linux/package-artifacts.json",
    "platform/linux/package-artifacts.json",
    "platform/linux-package-artifacts.json",
    "linux/package-artifacts.json",
    "linux-package-artifacts.json",
  ]),
  item("platform-linux-webdriver", ".tmp/platform-evidence/external/linux/tauri-webdriver-report.json", "platform", [
    "platform-evidence/external/linux/tauri-webdriver-report.json",
    "platform/linux/tauri-webdriver-report.json",
    "platform/linux-tauri-webdriver-report.json",
    "linux/tauri-webdriver-report.json",
    "linux-tauri-webdriver-report.json",
  ]),
  item("signing-darwin", ".tmp/release-signing/external/darwin/signing-evidence.json", "signing", [
    "release-signing/external/darwin/signing-evidence.json",
    "signing/darwin/signing-evidence.json",
    "signing/darwin-signing-evidence.json",
    "darwin/signing-evidence.json",
    "darwin-signing-evidence.json",
  ]),
  item("signing-win32", ".tmp/release-signing/external/win32/signing-evidence.json", "signing", [
    "release-signing/external/win32/signing-evidence.json",
    "signing/win32/signing-evidence.json",
    "signing/win32-signing-evidence.json",
    "win32/signing-evidence.json",
    "win32-signing-evidence.json",
  ]),
  item("signing-linux", ".tmp/release-signing/external/linux/signing-evidence.json", "signing", [
    "release-signing/external/linux/signing-evidence.json",
    "signing/linux/signing-evidence.json",
    "signing/linux-signing-evidence.json",
    "linux/signing-evidence.json",
    "linux-signing-evidence.json",
  ]),
  item("homebrew-cask", ".tmp/homebrew/external/neditor.rb", "homebrew", [
    "homebrew/neditor.rb",
    "homebrew/Casks/neditor.rb",
    "Casks/neditor.rb",
    "neditor.rb",
  ]),
  item("homebrew-release-artifact", ".tmp/homebrew/external/neditor-release-artifact", "homebrew", [
    `homebrew/NEditor-${packageJson.version}-macos.zip`,
    `homebrew/NEditor-${packageJson.version}-macos.dmg`,
    "homebrew/NEditor-macos.zip",
    "homebrew/NEditor-macos.dmg",
    "homebrew/neditor-release-artifact",
    `NEditor-${packageJson.version}-macos.zip`,
    `NEditor-${packageJson.version}-macos.dmg`,
    "NEditor-macos.zip",
    "NEditor-macos.dmg",
    "neditor-release-artifact",
  ]),
  item("homebrew-materialization-report", ".tmp/homebrew/external/materialize-cask-report.json", "homebrew", [
    "homebrew/materialize-cask-report.json",
    "homebrew/materialization-report.json",
    ".tmp/homebrew/materialize-cask-report.json",
    "materialize-cask-report.json",
    "materialization-report.json",
  ]),
  item("google-docs-import", ".tmp/google-docs-import/external/import-evidence.json", "google-docs", [
    "google-docs-import/external/import-evidence.json",
    "google-docs/import-evidence.json",
    "import-evidence.json",
  ]),
  item("ai-provider-endpoint", ".tmp/ai-provider-evidence/external/provider-evidence.json", "ai-provider", [
    "ai-provider-evidence/external/provider-evidence.json",
    "ai-provider/provider-evidence.json",
    "ai-provider-endpoint/provider-evidence.json",
    "provider-evidence.json",
  ]),
  item("ai-runtime-device", ".tmp/ai-runtime-evidence/external/runtime-evidence.json", "ai-runtime", [
    "ai-runtime-evidence/external/runtime-evidence.json",
    "ai-runtime/runtime-evidence.json",
    "ai-runtime-device/runtime-evidence.json",
    "runtime-evidence.json",
  ]),
  item("security-review-signoff", ".tmp/security-review/external/security-review.json", "security-review", [
    "security-review/external/security-review.json",
    "security/security-review.json",
    "independent-security-review/security-review.json",
    "security-review.json",
  ]),
  item("performance-native-profile", ".tmp/performance-profile/external/native-profile.json", "performance-profile", [
    "performance-profile/external/native-profile.json",
    "performance/native-profile.json",
    "release-device-performance-profile/native-profile.json",
    "native-profile.json",
  ]),
  item("rendered-export-signoff", ".tmp/rendered-export-audit/external/visual-review-signoff.json", "rendered-signoff", [
    "rendered-export/visual-review-signoff.json",
    "rendered-export-human-review/visual-review-signoff.json",
    "visual-review-signoff.json",
  ]),
  item("table-editor-signoff", ".tmp/table-editor/external/manual-review-signoff.json", "table-editor-signoff", [
    "table-editor/manual-review-signoff.json",
    "table-editor-human-review/manual-review-signoff.json",
    "table-editor-signoff.json",
  ]),
  item("accessibility-signoff", ".tmp/accessibility/external/manual-review-signoff.json", "accessibility-signoff", [
    "accessibility/manual-review-signoff.json",
    "accessibility-human-review/manual-review-signoff.json",
    "manual-review-signoff.json",
  ]),
  item("external-engine-pikchr", ".tmp/external-engines/external/pikchr.json", "external-engine", [
    "external-engines/external/pikchr.json",
    "external-engines/pikchr.json",
    "optional-engines/pikchr.json",
    "pikchr.json",
  ]),
  item("external-engine-sqlite", ".tmp/external-engines/external/sqlite.json", "external-engine", [
    "external-engines/external/sqlite.json",
    "external-engines/sqlite.json",
    "optional-engines/sqlite.json",
    "sqlite.json",
  ]),
];

if (listOnly) {
  console.log("NEditor release evidence ingest recognizes:");
  for (const entry of evidenceItems) {
    console.log(`- ${entry.id}: ${entry.destination}`);
    console.log(`  candidates: ${entry.candidates.join(", ")}`);
  }
  console.log("- manual-review-<work-order-id>: .tmp/manual-review/external/<work-order-id>/signoff.json");
  console.log("  candidates: any returned neditor.manual-review.signoff.v1 JSON file, with optional sibling artifacts/ directory");
  process.exit(0);
}

const copied = [];
const missing = [];
const categories = new Set();

if (!existsSync(sourceDir)) {
  writeReport("failed", copied, evidenceItems.map((entry) => missingEntry(entry, "source directory is missing")), []);
  console.error(`Release evidence return directory does not exist: ${sourceDir}`);
  process.exit(1);
}

for (const entry of evidenceItems) {
  const found = findCandidate(entry);
  if (!found) {
    missing.push(missingEntry(entry, "not supplied"));
    continue;
  }
  const destination = join(root, entry.destination);
  const stat = statSync(found);
  if (!dryRun) {
    mkdirSync(dirname(destination), { recursive: true });
    cpSync(found, destination);
  }
  copied.push({
    id: entry.id,
    category: entry.category,
    source: relative(found),
    destination: entry.destination,
    bytes: stat.size,
    dryRun,
  });
  categories.add(entry.category);
}

for (const found of findManualReviewSignoffs(sourceDir)) {
  const workOrderId = String(found.data.workOrderId || "").trim();
  const destinationDir = join(root, ".tmp", "manual-review", "external", workOrderId);
  const destination = join(destinationDir, "signoff.json");
  if (!dryRun) {
    mkdirSync(destinationDir, { recursive: true });
    cpSync(found.path, destination);
    if (existsSync(found.artifactsDir) && statSync(found.artifactsDir).isDirectory()) {
      cpSync(found.artifactsDir, join(destinationDir, "artifacts"), { recursive: true });
    }
  }
  copied.push({
    id: `manual-review-${workOrderId}`,
    category: "manual-review",
    source: relative(found.path),
    destination: `.tmp/manual-review/external/${workOrderId}/signoff.json`,
    bytes: statSync(found.path).size,
    dryRun,
  });
  categories.add("manual-review");
}

const validations = validate ? runValidations(categories) : [];
const failedValidations = validations.filter((validation) => validation.status !== "passed");
const status =
  copied.length === 0
    ? "empty"
    : failedValidations.length > 0
      ? "failed"
      : missing.length > 0
        ? "partial"
        : "complete";
writeReport(status, copied, missing, validations);

if (copied.length === 0) {
  console.error(`No recognized release evidence files were found under ${sourceDir}.`);
  process.exit(1);
}
if (failedValidations.length > 0) {
  console.error("Release evidence ingest validation failed:");
  for (const validation of failedValidations) console.error(`- ${validation.label}: ${validation.status}`);
  process.exit(1);
}

console.log(`Release evidence ingest ${dryRun ? "dry run " : ""}${status}; copied ${copied.length} item(s).`);

function item(id, destination, category, candidates) {
  return {
    id,
    destination,
    category,
    candidates: Array.from(new Set([destination, destination.replace(/^\.tmp\//, ""), ...candidates])),
  };
}

function findCandidate(entry) {
  for (const candidate of entry.candidates) {
    const path = resolve(sourceDir, candidate);
    if (existsSync(path) && statSync(path).isFile()) return path;
  }
  return null;
}

function runValidations(categories) {
  const commands = [];
  if (categories.has("platform")) commands.push(command("platform evidence", "pnpm", ["run", "check:platform-evidence"]));
  if (categories.has("signing")) commands.push(command("release signing evidence", "pnpm", ["run", "check:release-signing"]));
  if (categories.has("homebrew")) commands.push(command("Homebrew release evidence", "pnpm", ["run", "check:homebrew"]));
  if (categories.has("google-docs")) commands.push(command("Google Docs import evidence", "pnpm", ["run", "check:google-docs-import"]));
  if (categories.has("ai-provider")) commands.push(command("AI provider evidence", "pnpm", ["run", "check:ai-provider"]));
  if (categories.has("ai-runtime")) commands.push(command("AI runtime evidence", "pnpm", ["run", "check:ai-runtime"]));
  if (categories.has("security-review")) commands.push(command("security review evidence", "pnpm", ["run", "check:security-review"]));
  if (categories.has("performance-profile")) commands.push(command("performance profile evidence", "pnpm", ["run", "check:performance-profile"]));
  if (categories.has("external-engine")) commands.push(command("external engine evidence", "pnpm", ["run", "check:engines"]));
  if (categories.has("rendered-signoff")) {
    commands.push(
      command("rendered export signoff", "pnpm", ["run", "test:rendered-exports", "--", "--validate-signoff-only"], {
        NEDITOR_RENDERED_EXPORT_SIGNOFF: join(root, ".tmp", "rendered-export-audit", "external", "visual-review-signoff.json"),
      }),
    );
  }
  if (categories.has("accessibility-signoff")) {
    commands.push(
      command("accessibility signoff", "pnpm", ["run", "check:a11y:manual"], {
        NEDITOR_ACCESSIBILITY_SIGNOFF: join(root, ".tmp", "accessibility", "external", "manual-review-signoff.json"),
      }),
    );
  }
  if (categories.has("table-editor-signoff")) {
    commands.push(
      command("table editor signoff", "pnpm", ["run", "check:tables:manual"], {
        NEDITOR_TABLE_EDITOR_SIGNOFF: join(root, ".tmp", "table-editor", "external", "manual-review-signoff.json"),
      }),
    );
  }
  if (categories.has("manual-review")) commands.push(command("spec manual review evidence", "pnpm", ["run", "check:manual-review"]));

  return commands.map((entry) => {
    const result = spawnSync(entry.cmd, entry.args, {
      cwd: root,
      env: { ...process.env, ...entry.env },
      shell: process.platform === "win32",
      encoding: "utf8",
    });
    return {
      label: entry.label,
      command: [entry.cmd, ...entry.args],
      status: result.status === 0 ? "passed" : "failed",
      exitCode: result.status ?? 1,
      stdoutTail: tail(result.stdout),
      stderrTail: tail(result.stderr),
    };
  });
}

function findManualReviewSignoffs(dir) {
  if (!existsSync(dir)) return [];
  return walkJson(dir).flatMap((path) => {
    let data;
    try {
      data = JSON.parse(readFileSync(path, "utf8"));
    } catch {
      return [];
    }
    if (data?.schema !== "neditor.manual-review.signoff.v1" || !data.workOrderId) return [];
    return [
      {
        path,
        data,
        artifactsDir: join(dirname(path), "artifacts"),
      },
    ];
  });
}

function walkJson(dir) {
  const entries = [];
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      entries.push(...walkJson(path));
    } else if (entry.isFile() && entry.name.endsWith(".json")) {
      entries.push(path);
    }
  }
  return entries;
}

function command(label, cmd, args, env = {}) {
  return { label, cmd, args, env };
}

function missingEntry(entry, detail) {
  return {
    id: entry.id,
    category: entry.category,
    destination: entry.destination,
    candidates: entry.candidates,
    detail,
  };
}

function writeReport(status, copied, missing, validations) {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(
    reportPath,
    `${JSON.stringify(
      {
        schema: "neditor.release-evidence-ingest.v1",
        generatedAt: new Date().toISOString(),
        status,
        dryRun,
        sourceDir: relative(sourceDir),
        copied,
        missing,
        validations,
        summary: {
          recognized: copied.length,
          missing: missing.length,
          validations: validations.length,
          failedValidations: validations.filter((validation) => validation.status !== "passed").length,
        },
      },
      null,
      2,
    )}\n`,
  );
}

function parseArgs(values) {
  const parsed = {};
  for (let index = 0; index < values.length; index += 1) {
    const value = values[index];
    if (!value.startsWith("--")) continue;
    const key = value.slice(2);
    if (key === "dry-run" || key === "list" || key === "no-validate") {
      parsed[key] = true;
      continue;
    }
    const next = values[index + 1];
    if (!next || next.startsWith("--")) {
      parsed[key] = true;
    } else {
      parsed[key] = next;
      index += 1;
    }
  }
  return parsed;
}

function tail(value) {
  return String(value || "")
    .split(/\r?\n/)
    .filter(Boolean)
    .slice(-40);
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
