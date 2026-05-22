import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const requireInstalled = process.argv.includes("--require-installed");
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const reportPath = join(root, ".tmp", "external-engines", "probe-report.json");
const artifactDir = join(root, ".tmp", "external-engines", "artifacts");
const engines = [
  {
    name: "Graphviz / DOT",
    command: "dot",
    env: "NEDITOR_TEST_DOT",
    versionArgs: ["-V"],
    smoke: {
      kind: "graphviz",
      source: 'digraph NEditor { Intake -> Export [label="NEditor engine smoke"]; }',
      args: ["-Tsvg"],
      artifact: "dot.svg",
      needles: ["<svg", "NEditor"],
    },
  },
  {
    name: "Graphviz / circo",
    command: "circo",
    env: "NEDITOR_TEST_CIRCO",
    versionArgs: ["-V"],
    smoke: {
      kind: "graphviz",
      source: 'graph NEditor { Intake -- Export [label="NEditor engine smoke"]; }',
      args: ["-Tsvg"],
      artifact: "circo.svg",
      needles: ["<svg", "NEditor"],
    },
  },
  {
    name: "Graphviz / neato",
    command: "neato",
    env: "NEDITOR_TEST_NEATO",
    versionArgs: ["-V"],
    smoke: {
      kind: "graphviz",
      source: 'graph NEditor { Intake -- Export [label="NEditor engine smoke"]; }',
      args: ["-Tsvg"],
      artifact: "neato.svg",
      needles: ["<svg", "NEditor"],
    },
  },
  {
    name: "Graphviz / fdp",
    command: "fdp",
    env: "NEDITOR_TEST_FDP",
    versionArgs: ["-V"],
    smoke: {
      kind: "graphviz",
      source: 'graph NEditor { Intake -- Export [label="NEditor engine smoke"]; }',
      args: ["-Tsvg"],
      artifact: "fdp.svg",
      needles: ["<svg", "NEditor"],
    },
  },
  {
    name: "Graphviz / osage",
    command: "osage",
    env: "NEDITOR_TEST_OSAGE",
    versionArgs: ["-V"],
    smoke: {
      kind: "graphviz",
      source: 'graph NEditor { Intake -- Export [label="NEditor engine smoke"]; }',
      args: ["-Tsvg"],
      artifact: "osage.svg",
      needles: ["<svg", "NEditor"],
    },
  },
  {
    name: "Graphviz / twopi",
    command: "twopi",
    env: "NEDITOR_TEST_TWOPI",
    versionArgs: ["-V"],
    smoke: {
      kind: "graphviz",
      source: 'graph NEditor { Intake -- Export [label="NEditor engine smoke"]; }',
      args: ["-Tsvg"],
      artifact: "twopi.svg",
      needles: ["<svg", "NEditor"],
    },
  },
  {
    name: "D2",
    command: "d2",
    env: "NEDITOR_TEST_D2",
    versionArgs: ["--version"],
    smoke: {
      kind: "stdin",
      source: "intake -> export: NEditor engine smoke",
      args: ["-", "-"],
      artifact: "d2.svg",
      needles: ["<svg"],
    },
  },
  {
    name: "PlantUML",
    command: "plantuml",
    env: "NEDITOR_TEST_PLANTUML",
    versionArgs: ["-version"],
    smoke: {
      kind: "plantuml-file",
      source: "@startuml\nAlice -> Bob: NEditor engine smoke\n@enduml\n",
      args: ["-tsvg"],
      artifact: "plantuml.svg",
      needles: ["<svg", "NEditor"],
    },
  },
  {
    name: "Pikchr",
    command: "pikchr",
    alternateCommands: ["pikchr-cli"],
    env: "NEDITOR_TEST_PIKCHR",
    versionArgs: ["--version"],
    smoke: {
      kind: "pikchr",
      source: 'box "NEditor"; arrow; box "Export"',
      artifact: "pikchr.svg",
      needles: ["<svg", "NEditor"],
    },
  },
];

rmSync(artifactDir, { recursive: true, force: true });
mkdirSync(artifactDir, { recursive: true });

const rows = engines.map(probeEngine);
const missing = rows.filter((row) => row.status === "missing");
const incompatible = rows.filter((row) => row.status === "incompatible");
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
  if (row.smoke) {
    console.log(`  smoke: ${row.smoke.status}`);
    if (row.smoke.artifact) {
      console.log(`  artifact: ${row.smoke.artifact}`);
    }
    if (row.smoke.bytes) {
      console.log(`  bytes: ${row.smoke.bytes}`);
    }
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

if (incompatible.length > 0) {
  console.log("");
  console.log(
    `Installed engines with failed smoke proof: ${incompatible.map((row) => row.command).join(", ")}`,
  );
  process.exit(1);
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
  const smoke = runSmoke(engine, path || command);
  if (!smoke.passed) {
    return {
      name: engine.name,
      command,
      path: path || command,
      status: "incompatible",
      version: version || "version probe did not return output",
      smoke: {
        status: "failed",
        ...smoke,
      },
      note: smoke.error || smoke.stderr || "Installed engine did not produce the expected smoke artifact.",
    };
  }
  return {
    name: engine.name,
    command,
    path: path || command,
    status: "installed",
    version: version || "version probe did not return output",
    smoke: {
      status: "passed",
      ...smoke,
    },
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

function runSmoke(engine, commandPath) {
  if (!engine.smoke) {
    return { passed: true };
  }
  const artifactPath = join(artifactDir, engine.smoke.artifact);
  const tempDir = join(artifactDir, ".tmp");
  mkdirSync(tempDir, { recursive: true });

  let result;
  let output = "";
  try {
    if (engine.smoke.kind === "plantuml-file") {
      const sourcePath = join(tempDir, `${safeName(engine.command)}.puml`);
      writeFileSync(sourcePath, engine.smoke.source);
      result = spawnSync(commandPath, [...engine.smoke.args, sourcePath], {
        encoding: "utf8",
        shell: false,
        timeout: 20_000,
      });
      const sidecarPath = sourcePath.replace(/\.puml$/, ".svg");
      output = existsSync(sidecarPath) ? readFileSync(sidecarPath, "utf8") : `${result.stdout || ""}${result.stderr || ""}`;
    } else if (engine.smoke.kind === "pikchr" && usesSourceFileArgument(commandPath)) {
      const sourcePath = join(tempDir, `${safeName(engine.command)}.pikchr`);
      writeFileSync(sourcePath, engine.smoke.source);
      result = spawnSync(commandPath, [sourcePath], {
        encoding: "utf8",
        shell: false,
        timeout: 20_000,
      });
      output = result.stdout || "";
    } else {
      const args = engine.smoke.kind === "pikchr" ? [] : engine.smoke.args;
      result = spawnSync(commandPath, args, {
        input: engine.smoke.source,
        encoding: "utf8",
        shell: false,
        timeout: 20_000,
      });
      output = result.stdout || "";
    }
  } catch (error) {
    return {
      passed: false,
      error: String(error),
    };
  }

  const stderr = result.stderr?.trim() || "";
  const missingNeedles = engine.smoke.needles.filter((needle) => !output.includes(needle));
  const passed = result.status === 0 && output.length > 200 && missingNeedles.length === 0;
  if (output) {
    writeFileSync(artifactPath, output);
  }
  return {
    passed,
    artifact: output ? relative(artifactPath) : "",
    bytes: output ? statSync(artifactPath).size : 0,
    exitStatus: result.status,
    missingNeedles,
    stderr,
  };
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
        artifactDir: relative(artifactDir),
        engines: rows,
        missing: missing.map((row) => row.command),
        incompatible: rows
          .filter((row) => row.status === "incompatible")
          .map((row) => row.command),
      },
      null,
      2,
    )}\n`,
  );
}

function relative(path) {
  return path.startsWith(root) ? path.slice(root.length + 1) : path;
}

function safeName(value) {
  return String(value).replace(/[^a-z0-9_-]+/gi, "-").toLowerCase();
}

function usesSourceFileArgument(commandPath) {
  const stem = basename(commandPath).replace(/\.[^.]+$/, "").toLowerCase();
  return stem === "pikchr-cli" || stem.startsWith("pikchr-cli-");
}
