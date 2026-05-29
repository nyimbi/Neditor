import { chmodSync, copyFileSync, existsSync, mkdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { spawnSync } from "node:child_process";
import process from "node:process";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const manifestPath = join(root, "src-tauri", "Cargo.toml");
const binariesDir = join(root, "src-tauri", "binaries");
const extension = process.platform === "win32" ? ".exe" : "";
const targetTriple = process.env.NEDITOR_SIDECAR_TARGET_TRIPLE || rustTargetTriple();
const source = join(root, "src-tauri", "target", "release", `ned${extension}`);
const target = join(binariesDir, `ned-${targetTriple}${extension}`);

run("cargo", ["build", "--manifest-path", manifestPath, "--locked", "--release", "--bin", "ned"]);
if (!existsSync(source)) {
  fail(`ned release binary was not produced at ${relative(source)}`);
}
mkdirSync(binariesDir, { recursive: true });
copyFileSync(source, target);
chmodSync(target, 0o755);

const stat = statSync(target);
if (!stat.isFile() || stat.size < 100_000) {
  fail(`prepared ned sidecar is unexpectedly small or missing: ${relative(target)}`);
}
smokePreparedSidecar(target);

console.log(`Prepared ned sidecar ${relative(target)} (${stat.size} bytes).`);

function smokePreparedSidecar(path) {
  const result = spawnSync(path, ["--version"], { encoding: "utf8" });
  if (result.status !== 0) {
    const detail = result.stderr || result.stdout || `exit ${result.status}`;
    fail(`prepared ned sidecar did not run --version successfully: ${detail}`);
  }
  const expected = `ned ${packageJson.version}`;
  const actual = result.stdout.trim();
  if (actual !== expected) {
    fail(`prepared ned sidecar version mismatch: expected ${expected}, found ${actual || "(empty output)"}`);
  }
}

function rustTargetTriple() {
  const direct = spawnSync("rustc", ["--print", "host-tuple"], { encoding: "utf8" });
  if (direct.status === 0 && direct.stdout.trim()) return direct.stdout.trim();

  const verbose = spawnSync("rustc", ["-vV"], { encoding: "utf8" });
  if (verbose.status !== 0) fail("could not determine Rust host target triple");
  const match = verbose.stdout.match(/^host:\s*(\S+)/m);
  if (!match) fail("rustc -vV did not report a host target triple");
  return match[1];
}

function run(command, args) {
  const result = spawnSync(command, args, {
    cwd: root,
    stdio: "inherit",
  });
  if (result.status !== 0) {
    fail(`${command} ${args.join(" ")} failed with exit code ${result.status}`);
  }
}

function fail(message) {
  console.error(message);
  process.exit(1);
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
