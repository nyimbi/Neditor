import { spawn, spawnSync } from "node:child_process";
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const launchRequested =
  process.argv.includes("--launch") || process.env.NEDITOR_DESKTOP_SMOKE_LAUNCH === "1";
const launchTimeoutMs = Number(process.env.NEDITOR_DESKTOP_SMOKE_TIMEOUT_MS || 3000);
const issues = [];

const tauriConfig = readJson("src-tauri/tauri.conf.json");
const packageJson = readJson("package.json");
const cargoToml = readText("src-tauri/Cargo.toml");
const binaryPath = desktopBinaryPath();

requireEqual(packageJson.license, "MIT", "package.json must declare MIT license");
requireEqual(tauriConfig.productName, "NEditor", "Tauri productName must remain NEditor");
requireEqual(tauriConfig.identifier, "com.neditor.desktop", "Tauri bundle identifier must remain stable");
requireEqual(tauriConfig.build?.frontendDist, "../dist", "Tauri frontendDist must point at the built Vite dist");
requireEqual(tauriConfig.bundle?.license, "MIT", "Tauri bundle must declare MIT license");
requireEqual(tauriConfig.bundle?.licenseFile, "../LICENSE", "Tauri bundle must point at the repository license file");
requireIncludes(cargoToml, 'name = "neditor"', "Cargo package must remain named neditor");
requireIncludes(cargoToml, 'license = "MIT"', "Cargo package must declare MIT license");

requireFile("dist/index.html", "frontend build output is missing; run pnpm run build first");
const assetDir = join(root, "dist", "assets");
if (!existsSync(assetDir) || !readdirSync(assetDir).some((name) => name.endsWith(".js"))) {
  issues.push("frontend asset bundle is missing from dist/assets");
}
requireExecutable(binaryPath, "desktop release binary is missing; run ./node_modules/.bin/tauri build --no-bundle first");

if (issues.length === 0) {
  runNativeCommandWorkflowSmoke();
}

if (issues.length === 0 && launchRequested) {
  await launchDesktop(binaryPath);
}

if (issues.length > 0) {
  console.error("Desktop smoke check failed:");
  for (const issue of issues) {
    console.error(`- ${issue}`);
  }
  process.exit(1);
}

console.log(
  launchRequested
    ? "Checked NEditor desktop build artifacts and bounded launch smoke."
    : "Checked NEditor desktop build artifacts and native command workflow smoke. Set NEDITOR_DESKTOP_SMOKE_LAUNCH=1 for bounded GUI launch smoke.",
);

function readJson(relativePath) {
  return JSON.parse(readText(relativePath));
}

function readText(relativePath) {
  return readFileSync(join(root, relativePath), "utf8");
}

function requireEqual(actual, expected, message) {
  if (actual !== expected) {
    issues.push(`${message}: expected ${JSON.stringify(expected)}, found ${JSON.stringify(actual)}`);
  }
}

function requireIncludes(text, needle, message) {
  if (!text.includes(needle)) {
    issues.push(message);
  }
}

function requireFile(relativePath, message) {
  const path = join(root, relativePath);
  if (!existsSync(path) || !statSync(path).isFile()) {
    issues.push(message);
  }
}

function requireExecutable(path, message) {
  if (!existsSync(path) || !statSync(path).isFile()) {
    issues.push(message);
    return;
  }
  if (process.platform !== "win32" && (statSync(path).mode & 0o111) === 0) {
    issues.push(`desktop release binary is not executable: ${relative(path)}`);
  }
}

function desktopBinaryPath() {
  const name = process.platform === "win32" ? "neditor.exe" : "neditor";
  return join(root, "src-tauri", "target", "release", name);
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}

function runNativeCommandWorkflowSmoke() {
  const result = spawnSync(
    "cargo",
    ["test", "--locked", "desktop_native_command_workflow_smoke", "--lib"],
    {
      cwd: join(root, "src-tauri"),
      encoding: "utf8",
      shell: process.platform === "win32",
    },
  );
  if (result.status !== 0) {
    const detail = [result.stdout?.trim(), result.stderr?.trim()].filter(Boolean).join("\n");
    issues.push(
      `native command workflow smoke failed with exit code ${result.status ?? 1}${
        detail ? `:\n${detail}` : ""
      }`,
    );
  }
}

async function launchDesktop(path) {
  await new Promise((resolveLaunch) => {
    const child = spawn(path, [], {
      cwd: root,
      env: {
        ...process.env,
        RUST_BACKTRACE: "1",
      },
      stdio: ["ignore", "pipe", "pipe"],
    });
    let stdout = "";
    let stderr = "";
    let settled = false;
    const timeout = setTimeout(() => {
      if (settled) return;
      settled = true;
      child.kill("SIGTERM");
      resolveLaunch();
    }, launchTimeoutMs);
    child.stdout.on("data", (chunk) => {
      stdout += chunk.toString();
    });
    child.stderr.on("data", (chunk) => {
      stderr += chunk.toString();
    });
    child.on("error", (error) => {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      issues.push(`desktop launch failed: ${error.message}`);
      resolveLaunch();
    });
    child.on("exit", (code, signal) => {
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      if (code !== 0) {
        const detail = [stdout.trim(), stderr.trim()].filter(Boolean).join("\n");
        issues.push(`desktop launch exited early with code ${code ?? "none"} signal ${signal ?? "none"}${detail ? `: ${detail}` : ""}`);
      }
      resolveLaunch();
    });
  });
}
