import { spawnSync } from "node:child_process";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const requireInstalled = process.argv.includes("--require-installed");
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const reportPath = join(root, ".tmp", "external-engines", "probe-report.json");
const engines = [
  {
    name: "Graphviz / DOT",
    command: "dot",
    env: "NEDITOR_TEST_DOT",
    versionArgs: ["-V"],
  },
  {
    name: "Graphviz / circo",
    command: "circo",
    env: "NEDITOR_TEST_CIRCO",
    versionArgs: ["-V"],
  },
  {
    name: "Graphviz / neato",
    command: "neato",
    env: "NEDITOR_TEST_NEATO",
    versionArgs: ["-V"],
  },
  {
    name: "Graphviz / fdp",
    command: "fdp",
    env: "NEDITOR_TEST_FDP",
    versionArgs: ["-V"],
  },
  {
    name: "Graphviz / osage",
    command: "osage",
    env: "NEDITOR_TEST_OSAGE",
    versionArgs: ["-V"],
  },
  {
    name: "Graphviz / twopi",
    command: "twopi",
    env: "NEDITOR_TEST_TWOPI",
    versionArgs: ["-V"],
  },
  {
    name: "D2",
    command: "d2",
    env: "NEDITOR_TEST_D2",
    versionArgs: ["--version"],
  },
  {
    name: "PlantUML",
    command: "plantuml",
    env: "NEDITOR_TEST_PLANTUML",
    versionArgs: ["-version"],
  },
  {
    name: "Pikchr",
    command: "pikchr",
    alternateCommands: ["pikchr-cli"],
    env: "NEDITOR_TEST_PIKCHR",
    versionArgs: ["--version"],
  },
];

const rows = engines.map(probeEngine);
const missing = rows.filter((row) => row.status === "missing");
writeReport(rows, missing);

console.log(`NEditor external transform engine probe`);
console.log(`Platform: ${process.platform} ${process.arch}`);
console.log("");
for (const row of rows) {
  console.log(`${row.name}: ${row.status}`);
  console.log(`  command: ${row.command}`);
  if (row.path) {
    console.log(`  path: ${row.path}`);
  }
  if (row.version) {
    console.log(`  version: ${row.version}`);
  }
  if (row.note) {
    console.log(`  note: ${row.note}`);
  }
}

if (missing.length > 0) {
  console.log("");
  console.log(
    `Missing optional engines: ${missing.map((row) => row.command).join(", ")}`,
  );
  if (requireInstalled) {
    process.exit(1);
  }
}

console.log("");
console.log(`Wrote external transform engine probe report to ${relative(reportPath)}`);

function probeEngine(engine) {
  const command = process.env[engine.env] || findFirstCommand([
    engine.command,
    ...(engine.alternateCommands || []),
  ]);
  if (!command) {
    return {
      name: engine.name,
      command: [engine.command, ...(engine.alternateCommands || [])].join(" or "),
      status: "missing",
      note: `Set ${engine.env} to an absolute executable path to force a probe.`,
    };
  }

  const path = resolveCommand(command);
  const version = runVersion(command, engine.versionArgs);
  return {
    name: engine.name,
    command,
    path: path || command,
    status: "installed",
    version: version || "version probe did not return output",
  };
}

function findFirstCommand(commands) {
  for (const command of commands) {
    if (resolveCommand(command)) {
      return command;
    }
  }
  return null;
}

function resolveCommand(command) {
  if (command.includes("/") || command.includes("\\")) {
    return command;
  }
  const lookup = process.platform === "win32" ? "where" : "which";
  const args = [command];
  const result = spawnSync(lookup, args, {
    encoding: "utf8",
    shell: false,
  });
  if (result.status !== 0) {
    return null;
  }
  return firstLine(`${result.stdout}${result.stderr}`);
}

function runVersion(command, args) {
  const result = spawnSync(command, args, {
    encoding: "utf8",
    shell: false,
    timeout: 10_000,
  });
  return firstLine(`${result.stdout}${result.stderr}`);
}

function firstLine(text) {
  return text
    .split(/\r?\n/)
    .map((line) => line.trim())
    .find(Boolean);
}

function writeReport(rows, missing) {
  mkdirSync(dirname(reportPath), { recursive: true });
  writeFileSync(
    reportPath,
    `${JSON.stringify(
      {
        generatedAt: new Date().toISOString(),
        platform: process.platform,
        arch: process.arch,
        requireInstalled,
        engines: rows,
        missing: missing.map((row) => row.command),
      },
      null,
      2,
    )}\n`,
  );
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}
