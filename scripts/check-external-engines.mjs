import { createHash } from "node:crypto";
import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "node:fs";
import { basename, dirname, join, resolve } from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

const requireInstalled = process.argv.includes("--require-installed");
const writeEvidence = process.argv.includes("--write-evidence") || process.argv.includes("--collect-evidence");
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const packageJson = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const currentSourceCommit = gitCommit();
const currentSourceTreeClean = gitTreeClean();
const reportPath = join(root, ".tmp", "external-engines", "probe-report.json");
const artifactDir = join(root, ".tmp", "external-engines", "artifacts");
const evidenceDir = resolve(process.env.NEDITOR_EXTERNAL_ENGINE_EVIDENCE_DIR || join(root, ".tmp", "external-engines", "external"));
const templateDir = join(root, ".tmp", "external-engines", "templates");
const engines = [
  {
    key: "graphviz-dot",
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
    key: "graphviz-circo",
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
    key: "graphviz-neato",
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
    key: "graphviz-fdp",
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
    key: "graphviz-osage",
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
    key: "graphviz-twopi",
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
    key: "d2",
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
    key: "plantuml",
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
    key: "pikchr",
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
  {
    key: "sqlite",
    name: "SQLite / sqlite3",
    command: "sqlite3",
    env: "NEDITOR_TEST_SQLITE3",
    versionArgs: ["--version"],
    smoke: {
      kind: "sqlite",
      source:
        "WITH RECURSIVE n(value) AS (VALUES(1) UNION ALL SELECT value + 1 FROM n WHERE value < 12) SELECT 'NEditor SQL smoke' AS marker, value, printf('row-%02d-read-only-sql-transform-proof', value) AS detail FROM n;",
      artifact: "sqlite.csv",
      needles: ["marker,value,detail", "NEditor SQL smoke", "read-only-sql-transform-proof"],
    },
  },
];

rmSync(artifactDir, { recursive: true, force: true });
mkdirSync(artifactDir, { recursive: true });
mkdirSync(templateDir, { recursive: true });
writeEvidenceTemplates();

const rows = engines.map(probeEngine);
const missing = rows.filter((row) => row.status === "missing");
const incompatible = rows.filter((row) => row.status === "incompatible");
const invalidExternalEvidence = rows
  .map((row) => row.externalEvidence)
  .filter((evidence) => evidence?.status === "invalid");
writeReport(rows, missing, invalidExternalEvidence);

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
  if (row.externalEvidence) {
    console.log(`  external evidence: ${row.externalEvidence.status}`);
    if (row.externalEvidence.path) {
      console.log(`  evidence path: ${row.externalEvidence.path}`);
    }
    if (row.externalEvidence.detail) {
      console.log(`  evidence detail: ${row.externalEvidence.detail}`);
    }
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

if (invalidExternalEvidence.length > 0) {
  console.log("");
  console.log(
    `Invalid external engine evidence: ${invalidExternalEvidence.map((evidence) => evidence.path).join(", ")}`,
  );
  process.exit(1);
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
  let externalEvidence = evaluateExternalEvidence(engine);
  const command = process.env[engine.env] || findFirstCommand([
    engine.command,
    ...(engine.alternateCommands || []),
  ]);
  if (!command) {
    return {
      key: engine.key,
      name: engine.name,
      command: [engine.command, ...(engine.alternateCommands || [])].join(" or "),
      status: "missing",
      externalEvidence,
      note: `Set ${engine.env} to an absolute executable path to force a probe.`,
    };
  }

  const path = resolveCommand(command);
  const version = runVersion(command, engine.versionArgs);
  const smoke = runSmoke(engine, path || command);
  if (!smoke.passed) {
    return {
      key: engine.key,
      name: engine.name,
      command,
      path: path || command,
      status: "incompatible",
      version: version || "version probe did not return output",
      externalEvidence,
      smoke: {
        status: "failed",
        ...smoke,
      },
      note: smoke.error || smoke.stderr || "Installed engine did not produce the expected smoke artifact.",
    };
  }
  if (writeEvidence) {
    writeExternalEvidence(engine, {
      command,
      path: path || command,
      version: version || "version probe did not return output",
      smoke,
    });
    externalEvidence = evaluateExternalEvidence(engine);
  }
  return {
    key: engine.key,
    name: engine.name,
    command,
    path: path || command,
    status: "installed",
    version: version || "version probe did not return output",
    externalEvidence,
    smoke: {
      status: "passed",
      ...smoke,
    },
  };
}

function writeExternalEvidence(engine, proof) {
  mkdirSync(evidenceDir, { recursive: true });
  const evidence = {
    schema: "neditor.external-engine-evidence.v1",
    engine: engine.key,
    status: "passed",
    generatedAt: new Date().toISOString(),
    appVersion: packageJson.version,
    sourceCommit: currentSourceCommit,
    sourceTreeClean: currentSourceTreeClean,
    platform: process.platform,
    arch: process.arch,
    command: proof.command,
    path: proof.path,
    version: proof.version,
    adapter: {
      smokeKind: engine.smoke?.kind || "none",
      versionArgs: engine.versionArgs || [],
    },
    smoke: {
      status: "passed",
      artifact: proof.smoke.artifact || "",
      bytes: proof.smoke.bytes || 0,
      sha256: proof.smoke.sha256 || "",
      needles: engine.smoke?.needles || [],
    },
    unresolvedBlockers: [],
    notes:
      "Collected by pnpm run collect:engine-evidence after the installed engine produced the required smoke artifact.",
  };
  const evidenceJson = `${JSON.stringify(evidence, null, 2)}\n`;
  const platformEvidenceDir = join(evidenceDir, process.platform);
  mkdirSync(platformEvidenceDir, { recursive: true });
  writeFileSync(join(evidenceDir, `${engine.key}.json`), evidenceJson);
  writeFileSync(join(platformEvidenceDir, `${engine.key}.json`), evidenceJson);
}

function evaluateExternalEvidence(engine) {
  const paths = externalEvidencePathsForEngine(engine);
  if (paths.length === 0) {
    return {
      status: "missing",
      path: relative(join(evidenceDir, `${engine.key}.json`)),
      items: [],
      accepted: [],
      invalid: [],
      detail: `No copied external evidence supplied for ${engine.name}.`,
    };
  }

  const items = paths.map((path) => evaluateExternalEvidenceFile(engine, path));
  const invalid = items.filter((item) => item.status === "invalid");
  const accepted = items.filter((item) => item.status === "accepted");
  if (invalid.length > 0) {
    return {
      status: "invalid",
      path: invalid.map((item) => item.path).join(", "),
      items,
      accepted,
      invalid,
      detail: invalid.map((item) => `${item.path}: ${item.detail}`).join("; "),
    };
  }
  if (accepted.length > 0) {
    const platforms = accepted.map((item) => `${item.platform}/${item.arch}`).join(", ");
    return {
      status: "accepted",
      path: accepted.map((item) => item.path).join(", "),
      generatedAt: accepted.map((item) => item.generatedAt).sort().at(-1),
      platform: accepted.map((item) => item.platform).join(", "),
      arch: accepted.map((item) => item.arch).join(", "),
      command: accepted.map((item) => item.command).join(", "),
      version: accepted.map((item) => item.version).join(" | "),
      smoke: accepted[0].smoke,
      items,
      accepted,
      invalid,
      detail: `${engine.name} evidence accepted from ${platforms}.`,
    };
  }
  return {
    status: "missing",
    path: relative(join(evidenceDir, `${engine.key}.json`)),
    items,
    accepted,
    invalid,
    detail: `No accepted copied external evidence supplied for ${engine.name}.`,
  };
}

function externalEvidencePathsForEngine(engine) {
  const filename = `${engine.key}.json`;
  const paths = [];
  collectExternalEvidencePaths(evidenceDir, filename, paths, 0);
  return Array.from(new Set(paths)).sort();
}

function collectExternalEvidencePaths(dir, filename, paths, depth) {
  if (depth > 3 || !existsSync(dir)) return;
  for (const entry of readdirSync(dir, { withFileTypes: true })) {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) {
      collectExternalEvidencePaths(path, filename, paths, depth + 1);
    } else if (entry.isFile() && entry.name === filename) {
      paths.push(path);
    }
  }
}

function evaluateExternalEvidenceFile(engine, path) {
  let evidence;
  try {
    evidence = JSON.parse(readFileSync(path, "utf8"));
  } catch (error) {
    return {
      status: "invalid",
      path: relative(path),
      detail: `Evidence JSON could not be parsed: ${error.message}`,
    };
  }

  const problems = [];
  requireValue(evidence.schema === "neditor.external-engine-evidence.v1", problems, "schema must be neditor.external-engine-evidence.v1");
  requireValue(evidence.engine === engine.key, problems, `engine must be ${engine.key}`);
  requireValue(evidence.status === "passed", problems, "status must be passed");
  requireValue(isIsoDate(evidence.generatedAt), problems, "generatedAt must be an ISO timestamp");
  if ("appVersion" in evidence) requireValue(Boolean(String(evidence.appVersion || "").trim()), problems, "appVersion must be non-empty when supplied");
  if ("sourceCommit" in evidence) requireValue(Boolean(String(evidence.sourceCommit || "").trim()), problems, "sourceCommit must be non-empty when supplied");
  if ("sourceTreeClean" in evidence) requireValue(typeof evidence.sourceTreeClean === "boolean", problems, "sourceTreeClean must be boolean when supplied");
  requireValue(Boolean(String(evidence.platform || "").trim()), problems, "platform is required");
  requireValue(Boolean(String(evidence.arch || "").trim()), problems, "arch is required");
  requireValue(Boolean(String(evidence.command || "").trim()), problems, "command is required");
  requireValue(Boolean(String(evidence.path || "").trim()), problems, "path is required");
  requireValue(Boolean(String(evidence.version || "").trim()), problems, "version is required");
  requireValue(evidence.smoke?.status === "passed", problems, "smoke.status must be passed");
  const minimumSmokeBytes = engine.smoke?.minimumBytes || 200;
  requireValue(Number(evidence.smoke?.bytes) > minimumSmokeBytes, problems, `smoke.bytes must be > ${minimumSmokeBytes}`);
  requireValue(isSha256(evidence.smoke?.sha256), problems, "smoke.sha256 must be a sha256");
  const needles = Array.isArray(evidence.smoke?.needles) ? evidence.smoke.needles : [];
  for (const needle of engine.smoke?.needles || []) {
    requireValue(needles.includes(needle), problems, `smoke.needles must include ${needle}`);
  }
  requireValue(Array.isArray(evidence.unresolvedBlockers) && evidence.unresolvedBlockers.length === 0, problems, "unresolvedBlockers must be an empty array");

  if (problems.length > 0) {
    return {
      status: "invalid",
      path: relative(path),
      detail: problems.join("; "),
    };
  }

  return {
    status: "accepted",
    path: relative(path),
    generatedAt: evidence.generatedAt,
    appVersion: evidence.appVersion || null,
    sourceCommit: evidence.sourceCommit || null,
    sourceTreeClean: evidence.sourceTreeClean ?? null,
    platform: evidence.platform,
    arch: evidence.arch,
    command: evidence.command,
    version: evidence.version,
    smoke: {
      bytes: evidence.smoke.bytes,
      sha256: evidence.smoke.sha256,
      artifact: evidence.smoke.artifact || "",
    },
    detail: `${engine.name} evidence accepted from ${evidence.platform}/${evidence.arch}.`,
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
    } else if (engine.smoke.kind === "sqlite") {
      result = spawnSync(commandPath, ["-header", "-csv", ":memory:", engine.smoke.source], {
        encoding: "utf8",
        shell: false,
        timeout: 20_000,
      });
      output = result.stdout || "";
    } else {
      const args = engine.smoke.kind === "pikchr" ? ["-"] : engine.smoke.args;
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
    sha256: output ? sha256File(artifactPath) : "",
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

function writeReport(rows, missing, invalidExternalEvidence) {
  mkdirSync(dirname(reportPath), { recursive: true });
  const externalEvidence = rows.map((row) => row.externalEvidence).filter(Boolean);
  const externalEvidenceItems = flattenEvidenceItems(rows);
  const acceptedExternalEvidence = externalEvidenceItems.filter((item) => item.status === "accepted");
  writeFileSync(
    reportPath,
    `${JSON.stringify(
      {
        generatedAt: new Date().toISOString(),
        appVersion: packageJson.version,
        sourceCommit: currentSourceCommit,
        sourceTreeClean: currentSourceTreeClean,
        platform: process.platform,
        arch: process.arch,
        requireInstalled,
        artifactDir: relative(artifactDir),
        evidenceDir: relative(evidenceDir),
        templateDir: relative(templateDir),
        status:
          incompatible.length > 0 || invalidExternalEvidence.length > 0
            ? "failed"
            : missing.some((row) => row.externalEvidence?.status !== "accepted")
              ? "partial"
              : "complete",
        summary: {
          installed: rows.filter((row) => row.status === "installed").length,
          missingLocal: missing.length,
          incompatible: incompatible.length,
          acceptedExternalEvidence: acceptedExternalEvidence.length,
          acceptedExternalEnginePlatforms: acceptedExternalEvidencePlatforms(acceptedExternalEvidence),
          invalidExternalEvidence: invalidExternalEvidence.length,
          unresolvedMissingEvidence: missing.filter((row) => row.externalEvidence?.status !== "accepted").length,
        },
        engines: rows,
        missing: missing.map((row) => row.command),
        missingEvidence: missing
          .filter((row) => row.externalEvidence?.status !== "accepted")
          .map((row) => row.command),
        incompatible: rows
          .filter((row) => row.status === "incompatible")
          .map((row) => row.command),
        externalEvidence,
        externalEvidenceItems,
        invalidExternalEvidence,
      },
      null,
      2,
    )}\n`,
  );
}

function flattenEvidenceItems(rows) {
  return rows.flatMap((row) => {
    const items = Array.isArray(row.externalEvidence?.items) ? row.externalEvidence.items : [];
    return items.map((item) => ({
      engine: row.key,
      name: row.name,
      ...item,
    }));
  });
}

function acceptedExternalEvidencePlatforms(items) {
  return Array.from(
    new Set(
      items
        .filter((item) => item.status === "accepted")
        .map((item) => `${item.platform}/${item.arch}`)
        .filter(Boolean),
    ),
  ).sort();
}

function writeEvidenceTemplates() {
  for (const engine of engines) {
    const templatePath = join(templateDir, `${engine.key}.template.json`);
    writeFileSync(
      templatePath,
      `${JSON.stringify(
        {
          schema: "neditor.external-engine-evidence.v1",
          engine: engine.key,
          status: "passed",
          generatedAt: new Date().toISOString(),
          platform: process.platform,
          arch: process.arch,
          command: engine.command,
          path: "/absolute/path/to/executable",
          version: "paste version output",
          adapter: {
            smokeKind: engine.smoke?.kind || "none",
            versionArgs: engine.versionArgs || [],
          },
          smoke: {
            status: "passed",
            artifact: `.tmp/external-engines/artifacts/${engine.smoke?.artifact || `${engine.key}.svg`}`,
            bytes: 12345,
            sha256: "replace-with-64-character-sha256",
            needles: engine.smoke?.needles || [],
          },
          unresolvedBlockers: [],
          notes:
            "Copy the real evidence from a host where this optional engine is installed after running pnpm run check:engines.",
        },
        null,
        2,
      )}\n`,
    );
  }
}

function requireValue(condition, problems, message) {
  if (!condition) problems.push(message);
}

function isIsoDate(value) {
  return typeof value === "string" && !Number.isNaN(Date.parse(value));
}

function isSha256(value) {
  return typeof value === "string" && /^[a-f0-9]{64}$/i.test(value);
}

function sha256File(path) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function gitCommit() {
  const result = spawnSync("git", ["rev-parse", "HEAD"], {
    cwd: root,
    encoding: "utf8",
    shell: false,
  });
  return result.status === 0 ? result.stdout.trim() : "";
}

function gitTreeClean() {
  const result = spawnSync("git", ["status", "--porcelain"], {
    cwd: root,
    encoding: "utf8",
    shell: false,
  });
  return result.status === 0 && result.stdout.trim().length === 0;
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
