import { chromium } from "@playwright/test";
import { spawn } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = process.cwd();
const host = "127.0.0.1";
const port = Number(process.env.NEDITOR_SCREENSHOT_PORT || 5176);
const baseUrl = process.env.NEDITOR_SCREENSHOT_URL || `http://${host}:${port}`;
const showcaseRoot = join(root, "examples", "showcase");
const outputRoot = join(root, "docs", "screenshots");
const rootShowcasePath = "/workspace/showcase/neditor-capability-showcase.md";

function walk(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
    const path = join(dir, entry.name);
    if (entry.isDirectory()) return walk(path);
    return [path];
  });
}

function readShowcaseFiles() {
  const files = {};
  for (const path of walk(showcaseRoot)) {
    const relativePath = relative(showcaseRoot, path).replace(/\\/g, "/");
    files[`/workspace/showcase/${relativePath}`] = readFileSync(path, "utf8");
  }
  return files;
}

async function waitForServer(url, deadlineMs = 30_000) {
  const deadline = Date.now() + deadlineMs;
  while (Date.now() < deadline) {
    try {
      const response = await fetch(url);
      if (response.ok) return;
    } catch {
      // Vite is still starting.
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  throw new Error(`Timed out waiting for ${url}`);
}

function startServerIfNeeded() {
  if (process.env.NEDITOR_SCREENSHOT_URL) return null;
  const child = spawn("pnpm", ["exec", "vite", "--host", host, "--port", String(port)], {
    cwd: root,
    stdio: ["ignore", "pipe", "pipe"],
    env: { ...process.env, BROWSER: "none" },
    detached: true,
  });
  child.stdout.on("data", (chunk) => process.stdout.write(chunk));
  child.stderr.on("data", (chunk) => process.stderr.write(chunk));
  return child;
}

function stopServer(server) {
  if (!server) return;
  try {
    process.kill(-server.pid, "SIGTERM");
  } catch {
    try {
      server.kill("SIGTERM");
    } catch {
      // The process may already be gone.
    }
  }
}

function installNeditorMock(files) {
  const rootShowcasePath = "/workspace/showcase/neditor-capability-showcase.md";
  const callbacks = new Map();
  const eventListeners = new Map();
  const stores = new Map();
  const storeResources = new Map();
  const localHash = (text) => {
    let value = 0;
    const source = String(text);
    for (let index = 0; index < source.length; index += 1) {
      value = (value * 31 + source.charCodeAt(index)) >>> 0;
    }
    return `mock-${value.toString(16)}`;
  };
  const mockFiles = new Map(
    Object.entries(files).map(([path, text]) => [
      path,
      { text, hash: localHash(text), modified: new Date(0).toISOString() },
    ]),
  );
  let callbackId = 1;
  let storeId = 1;

  const normalizePath = (path) => String(path || "").replace(/\\/g, "/");
  const dirnameOf = (path) => {
    const normalized = normalizePath(path);
    const index = normalized.lastIndexOf("/");
    return index <= 0 ? "/" : normalized.slice(0, index);
  };
  const titleFromPath = (path) => normalizePath(path).split("/").pop() || "Untitled";
  const escapeHtml = (text) =>
    String(text ?? "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  const slug = (text) =>
    String(text || "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-|-$/g, "");
  const frontMatterValue = (text, key) => {
    const frontMatter = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
    if (!frontMatter) return "";
    const match = frontMatter[1].match(new RegExp(`^${key}:\\s*(.+?)\\s*$`, "m"));
    return match?.[1].replace(/^["']|["']$/g, "").trim() || "";
  };
  const stripFrontMatter = (text) => text.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n?/, "");
  const resolveRelativePath = (basePath, target) => {
    if (target.startsWith("/")) return normalizePath(target);
    const stack = [];
    for (const part of `${dirnameOf(basePath)}/${target}`.split("/")) {
      if (!part || part === ".") continue;
      if (part === "..") stack.pop();
      else stack.push(part);
    }
    return `/${stack.join("/")}`;
  };
  const readMockFile = (path) => {
    const normalized = normalizePath(path);
    const file = mockFiles.get(normalized);
    if (!file) throw new Error(`Mock file not found: ${normalized}`);
    return { path: normalized, ...file };
  };
  const setFile = (path, text) => {
    mockFiles.set(normalizePath(path), { text, hash: localHash(text), modified: new Date(0).toISOString() });
  };
  const parseAttributes = (raw) => {
    const attrs = {};
    const id = raw.match(/#([A-Za-z0-9_.:-]+)/)?.[1];
    if (id) attrs.id = id;
    for (const match of raw.matchAll(/\b(caption|float|fit|position)=["']([^"']+)["']/g)) {
      attrs[match[1]] = match[2];
    }
    return attrs;
  };
  const splitTableRow = (line) =>
    line
      .trim()
      .replace(/^\|/, "")
      .replace(/\|$/, "")
      .split("|")
      .map((cell) => cell.trim());
  const parseTableCaption = (line) => {
    const match = line.match(/^Table:\s+(.+?)(?:\s+\{([^}]*)\})?\s*$/);
    if (!match) return null;
    const attrs = parseAttributes(match[2] || "");
    return { text: match[1].trim(), id: attrs.id || null };
  };
  const includeTarget = (line) => {
    const trimmed = line.trim();
    return (
      trimmed.match(/^!include\s+(.+)$/)?.[1] ||
      trimmed.match(/^\{\{\s*include\s+(.+?)\s*\}\}$/)?.[1] ||
      trimmed.match(/^<!--\s*include:\s*(.+?)\s*-->$/)?.[1] ||
      ""
    )
      .trim()
      .replace(/^["']|["']$/g, "");
  };
  const expandIncludes = (text, filePath, seen = new Set()) => {
    const lines = [];
    const includeGraph = [];
    const includedFiles = [];
    for (const line of text.split(/\r?\n/)) {
      const target = includeTarget(line);
      if (!target) {
        lines.push(line);
        continue;
      }
      const childPath = resolveRelativePath(filePath, target);
      includeGraph.push({ parent: filePath, child: childPath, depth: seen.size });
      const file = mockFiles.get(childPath);
      if (!file || seen.has(childPath)) continue;
      includedFiles.push({ path: childPath, hash: file.hash });
      lines.push(expandIncludes(stripFrontMatter(file.text), childPath, new Set([...seen, childPath])).compiled);
    }
    return { compiled: lines.join("\n"), includeGraph, includedFiles };
  };
  const headingsFromMarkdown = (text) =>
    text.split(/\r?\n/).flatMap((line, index) => {
      const match = line.match(/^(#{1,6})\s+(.+?)(?:\s+\{#[^}]+\})?\s*$/);
      if (!match) return [];
      const headingText = match[2].trim();
      return [{ level: match[1].length, text: headingText, anchor: slug(headingText), line: index + 1 }];
    });
  const injectToc = (text) => {
    if (!text.includes("[TOC]")) return text;
    const toc = headingsFromMarkdown(text.replace("[TOC]", ""))
      .filter((heading) => heading.level <= 3)
      .map((heading) => `${"  ".repeat(Math.max(0, heading.level - 1))}- [${heading.text}](#${heading.anchor})`)
      .join("\n");
    return text.replace("[TOC]", `## Table of Contents\n\n${toc}`);
  };
  const renderTable = (caption, tableLines) => {
    const rows = tableLines.map(splitTableRow);
    const headers = rows[0] || [];
    const bodyRows = rows.slice(2);
    const id = caption.id ? ` id="${escapeHtml(caption.id)}"` : "";
    return `<table${id}><caption>${escapeHtml(caption.text)}</caption><thead><tr>${headers
      .map((header) => `<th>${escapeHtml(header)}</th>`)
      .join("")}</tr></thead><tbody>${bodyRows
      .map((row) => `<tr>${row.map((cell) => `<td>${escapeHtml(cell)}</td>`).join("")}</tr>`)
      .join("")}</tbody></table>`;
  };
  const transformLabel = (name) =>
    ({
      calc: "Calculation",
      chart: "Chart",
      mermaid: "Mermaid diagram",
      d2: "D2 diagram",
      plantuml: "PlantUML diagram",
      timeline: "Timeline",
      roadmap: "Roadmap",
      "vega-lite": "Vega-Lite chart",
      csv: "CSV table",
      tsv: "TSV table",
      json: "JSON block",
      yaml: "YAML block",
      openapi: "OpenAPI contract",
      "json-schema": "JSON Schema",
      qr: "QR code",
      geojson: "GeoJSON map",
      topojson: "TopoJSON map",
      stl: "STL model",
    })[name] || name;
  const markdownToHtml = (text) => {
    const lines = text.split(/\r?\n/);
    const html = [];
    for (let index = 0; index < lines.length; index += 1) {
      const line = lines[index];
      const heading = line.match(/^(#{1,6})\s+(.+?)(?:\s+\{#([^}]+)\})?\s*$/);
      if (heading) {
        const level = Math.min(6, heading[1].length);
        html.push(`<h${level} id="${escapeHtml(heading[3] || slug(heading[2]))}">${escapeHtml(heading[2])}</h${level}>`);
        continue;
      }
      const caption = parseTableCaption(line);
      if (caption && lines[index + 1]?.trim().startsWith("|")) {
        const tableLines = [];
        let cursor = index + 1;
        while (lines[cursor]?.trim().startsWith("|")) {
          tableLines.push(lines[cursor]);
          cursor += 1;
        }
        html.push(renderTable(caption, tableLines));
        index = cursor - 1;
        continue;
      }
      const fence = line.match(/^```([A-Za-z0-9_-]+)(?:\s+caption=["']?([^"']+)["']?)?/);
      if (fence) {
        const body = [];
        let cursor = index + 1;
        while (cursor < lines.length && lines[cursor] !== "```") {
          body.push(lines[cursor]);
          cursor += 1;
        }
        const name = fence[1];
        html.push(
          `<figure class="transform transform-${escapeHtml(name)}"><figcaption>${escapeHtml(
            fence[2] || transformLabel(name),
          )}</figcaption><pre>${escapeHtml(body.join("\n"))}</pre></figure>`,
        );
        index = cursor;
        continue;
      }
      if (line.trim() === "$$") {
        const body = [];
        let cursor = index + 1;
        while (cursor < lines.length && !lines[cursor].trim().startsWith("$$")) {
          body.push(lines[cursor]);
          cursor += 1;
        }
        const attrs = parseAttributes((lines[cursor] || "").replace(/^\s*\$\$\s*/, ""));
        html.push(
          `<figure${attrs.id ? ` id="${escapeHtml(attrs.id)}"` : ""} class="equation"><pre>${escapeHtml(
            body.join("\n"),
          )}</pre><figcaption>${escapeHtml(attrs.caption || "Equation")}</figcaption></figure>`,
        );
        index = cursor;
        continue;
      }
      const figure = line.match(/^!\[([^\]]*)\]\(([^)]+)\)(?:\{([^}]*)\})?/);
      if (figure) {
        const attrs = parseAttributes(figure[3] || "");
        html.push(
          `<figure${attrs.id ? ` id="${escapeHtml(attrs.id)}"` : ""} class="figure"><img src="${escapeHtml(
            figure[2],
          )}" alt="${escapeHtml(figure[1])}"/><figcaption>${escapeHtml(attrs.caption || figure[1] || "Figure")}</figcaption></figure>`,
        );
        continue;
      }
      const tocItem = line.match(/^\s*-\s+\[(.+?)\]\(#(.+?)\)$/);
      if (tocItem) {
        html.push(`<li><a href="#${escapeHtml(tocItem[2])}">${escapeHtml(tocItem[1])}</a></li>`);
        continue;
      }
      if (!line.trim() || line.trim() === "---" || /^\s*[A-Za-z][\w-]*:\s/.test(line)) {
        continue;
      }
      html.push(`<p>${escapeHtml(line)}</p>`);
    }
    return html.join("\n");
  };
  const tableBlocksFromMarkdown = (text, sourceFile) => {
    const lines = text.split(/\r?\n/);
    const tables = [];
    for (let index = 0; index < lines.length; index += 1) {
      const caption = parseTableCaption(lines[index]);
      if (!caption || !lines[index + 1]?.trim().startsWith("|")) continue;
      const tableLines = [];
      let cursor = index + 1;
      while (lines[cursor]?.trim().startsWith("|")) {
        tableLines.push(lines[cursor]);
        cursor += 1;
      }
      const rows = tableLines.map(splitTableRow);
      tables.push({
        kind: "table",
        line: index + 1,
        end_line: cursor,
        id: caption.id,
        caption: caption.text,
        headers: rows[0] || [],
        alignments: rows[1] || [],
        rows: rows.slice(2),
        source: { source_file: sourceFile, source_line: index + 1, end_source_line: cursor },
      });
      index = cursor - 1;
    }
    return tables;
  };
  const regexItems = (text, pattern, mapper) => Array.from(text.matchAll(pattern), mapper);
  const compileMarkdown = (sourceText, filePath = rootShowcasePath) => {
    const expanded = expandIncludes(sourceText, filePath);
    const compiled = injectToc(expanded.compiled);
    const title = frontMatterValue(sourceText, "title") || headingsFromMarkdown(compiled)[0]?.text || titleFromPath(filePath);
    const status = frontMatterValue(sourceText, "status") || "draft";
    const headings = headingsFromMarkdown(compiled);
    const tables = tableBlocksFromMarkdown(compiled, filePath);
    const comments = regexItems(compiled, /^\s*<!--\s*comment:\s*([\s\S]*?)-->\s*$/gm, (match) => ({
      line: compiled.slice(0, match.index).split("\n").length,
      author: match[1].match(/author:\s*([^|]+)/)?.[1]?.trim() || "Reviewer",
      created_at: match[1].match(/at:\s*([^|]+)/)?.[1]?.trim() || "",
      state: /resolved/.test(match[1]) ? "resolved" : "unresolved",
      text: match[1].split("|").at(-1)?.trim() || "Review comment",
    }));
    const changeNotes = regexItems(compiled, /^\s*<!--\s*change:\s*([\s\S]*?)-->\s*$/gm, (match) => ({
      line: compiled.slice(0, match.index).split("\n").length,
      author: match[1].match(/author:\s*([^|]+)/)?.[1]?.trim() || "Reviewer",
      created_at: match[1].match(/at:\s*([^|]+)/)?.[1]?.trim() || "",
      text: match[1].split("|").at(-1)?.trim() || "Change note",
    }));
    const aiSources = regexItems(compiled, /```ai-source\s*\n([\s\S]*?)```/g, (match) => {
      const body = match[1];
      const field = (name) => body.match(new RegExp(`^${name}:\\s*(.*?)\\s*$`, "m"))?.[1]?.trim() || "";
      return {
        line: compiled.slice(0, match.index).split("\n").length,
        provider: field("provider"),
        model: field("model"),
        date: field("date"),
        prompt_summary: field("promptSummary") || field("prompt_summary"),
        reviewed_by: field("reviewedBy") || field("reviewed_by"),
        reviewed_at: field("reviewedAt") || field("reviewed_at"),
        status: field("status") || "unreviewed",
      };
    });
    const transformArtifacts = regexItems(
      compiled,
      /^```(calc|chart|mermaid|d2|plantuml|timeline|roadmap|vega-lite|csv|tsv|json|yaml|openapi|json-schema|qr|geojson|topojson|stl|sql-query)\b[\s\S]*?^```/gm,
      (match, index) => ({
        id: `mock-transform-${index}`,
        name: match[1],
        output_kind: "html",
        source_hash: localHash(match[0]),
        source: match[0],
        source_file: filePath,
        source_line: compiled.slice(0, match.index).split("\n").length,
        end_source_line: compiled.slice(0, match.index + match[0].length).split("\n").length,
        options: {},
        output_hash: localHash(`${match[1]}:${match[0]}`),
        cache_key: `${match[1]}:${localHash(match[0])}`,
        execution_kind: match[1] === "d2" ? "external" : "native",
        engine_version: null,
        engine_path: null,
        input_mode: "inline",
        duration_ms: 5,
        html: "",
        diagnostics: [],
      }),
    );
    const diagnostics = compiled.includes("TODO: citation needed")
      ? [
          {
            severity: "warning",
            message: "Citation TODO needs source verification before publishing.",
            source_file: filePath,
            line: compiled.slice(0, compiled.indexOf("TODO: citation needed")).split("\n").length,
            column: 1,
            end_line: compiled.slice(0, compiled.indexOf("TODO: citation needed")).split("\n").length,
            end_column: 2,
            suggestion: "Attach or cite the source document before external distribution.",
            related: ["citation-review"],
          },
        ]
      : [];
    const manifest = {
      document_title: title,
      document_version: frontMatterValue(sourceText, "version") || "1.0.0",
      status,
      approved_by: frontMatterValue(sourceText, "approvedBy") || null,
      approved_at: frontMatterValue(sourceText, "approvedAt") || null,
      owner: frontMatterValue(sourceText, "owner") || null,
      release_target: frontMatterValue(sourceText, "releaseTarget") || null,
      exported_at: new Date(0).toISOString(),
      source_hash: localHash(sourceText),
      output_path: null,
      output_hash: null,
      included_files: expanded.includedFiles,
      media_files: [],
      layout_sections: [],
      export_target: "html",
      export_options: {},
      transform_artifacts: transformArtifacts,
      progress_steps: [{ id: "compile", label: "Compile document model", state: "complete", detail: "Screenshot mock compile completed.", work_units: 1 }],
      diagnostics,
      source_map: [],
      app_version: "screenshot-mock",
    };
    return {
      compiled_markdown: compiled,
      html: markdownToHtml(compiled),
      semantic: {
        title,
        status,
        headings,
        outline: headings,
        tables: tables.length,
        table_summaries: tables.map((table) => ({
          line: table.line,
          id: table.id,
          caption: table.caption,
          rows: table.rows.length,
          columns: table.headers,
          formulas: [],
        })),
        figures: (compiled.match(/^!\[[^\]]*\]\([^)]+\)/gm) || []).length,
        equations: (compiled.match(/^\$\$/gm) || []).length / 2,
        citations: regexItems(compiled, /\[[^\]\n]*@([A-Za-z0-9_:-]+)([^\]\n]*)\]/g, (match) => ({ key: match[1], locator: match[2] || null })),
        citation_references: regexItems(compiled, /\[[^\]\n]*@([A-Za-z0-9_:-]+)([^\]\n]*)\]/g, (match) => ({ key: match[1], locator: match[2] || null })),
        duplicate_bibliography_keys: [],
        glossary: {},
        layout_directives: [],
        comments,
        change_notes: changeNotes,
        ai_sources: aiSources,
        ai_assisted_sections: [],
        labels: [],
        cross_references: [],
      },
      document_ast: {
        metadata: { title, status, source_hash: localHash(sourceText) },
        blocks: [
          ...headings.map((heading) => ({ kind: "heading", ...heading, end_line: heading.line, source: null })),
          ...tables,
        ],
      },
      paged_document: { sections: [] },
      diagnostics,
      include_graph: expanded.includeGraph,
      source_map: [],
      metadata: { title, status, source_hash: localHash(sourceText) },
      bibliography: [],
      index_terms: [],
      formula_graph: [],
      formula_dependency_edges: [],
      transform_artifacts: transformArtifacts,
      export_manifest: manifest,
    };
  };

  async function invoke(cmd, args = {}) {
    if (cmd === "plugin:store|load") {
      const rid = storeId++;
      const path = String(args.path || "settings.json");
      if (!stores.has(path)) stores.set(path, new Map());
      storeResources.set(rid, stores.get(path));
      return rid;
    }
    if (cmd === "plugin:store|get") {
      const store = storeResources.get(args.rid);
      const exists = Boolean(store?.has(String(args.key || "")));
      return [exists ? store.get(String(args.key || "")) : undefined, exists];
    }
    if (cmd === "plugin:store|set") {
      storeResources.get(args.rid)?.set(String(args.key || ""), args.value);
      return null;
    }
    if (cmd.startsWith("plugin:store|")) return null;
    if (cmd === "plugin:event|listen") {
      const eventName = args.event;
      const handler = args.handler;
      eventListeners.set(eventName, [...(eventListeners.get(eventName) || []), handler]);
      return handler;
    }
    if (cmd === "plugin:event|unlisten") return null;
    if (cmd === "plugin:window|set_title") return null;
    if (cmd === "plugin:dialog|open" || cmd === "plugin:dialog|save") return rootShowcasePath;
    if (cmd === "plugin:dialog|message") return "Ok";
    if (cmd === "plugin:dialog|confirm") return true;
    if (cmd === "pending_cli_open_paths") return [];
    if (cmd === "read_file") return readMockFile(args.path);
    if (cmd === "save_file") {
      setFile(args.request.path, args.request.text);
      return readMockFile(args.request.path);
    }
    if (cmd === "file_metadata") {
      const file = mockFiles.get(normalizePath(args.path));
      return { path: normalizePath(args.path), exists: Boolean(file), hash: file?.hash || null, modified: file?.modified || null };
    }
    if (cmd === "list_workspace_files") {
      const root = normalizePath(args.request.root).replace(/\/$/, "");
      return Array.from(mockFiles.keys())
        .filter((path) => path.startsWith(`${root}/`))
        .map((path) => ({
          path,
          name: titleFromPath(path),
          relative_path: path.slice(root.length + 1),
          kind: path.split(".").pop() || "file",
          depth: path.slice(root.length + 1).split("/").length - 1,
        }));
    }
    if (cmd === "compile_document_with_options") return compileMarkdown(args.request.text, args.request.file_path || rootShowcasePath);
    if (cmd === "prepare_for_export") {
      const compiled = compileMarkdown(args.request.text, args.request.file_path || rootShowcasePath);
      const diagnostics = compiled.diagnostics;
      const progressSteps = [
        ...compiled.export_manifest.progress_steps,
        { id: "render", label: `Render ${args.request.target || "html"} artifact`, state: "pending", detail: "Ready to write export artifact.", work_units: 1 },
        { id: "manifest", label: "Attach export manifest", state: "pending", detail: "Manifest captures hashes, options, diagnostics, and provenance.", work_units: 1 },
      ];
      return {
        ready: diagnostics.filter((diagnostic) => diagnostic.severity === "error").length === 0,
        error_count: diagnostics.filter((diagnostic) => diagnostic.severity === "error").length,
        warning_count: diagnostics.filter((diagnostic) => diagnostic.severity === "warning").length,
        info_count: 1,
        source_map: [],
        paged_document: compiled.paged_document,
        diagnostics,
        manifest: {
          ...compiled.export_manifest,
          export_target: args.request.target || "html",
          export_options: args.request.options || {},
          progress_steps: progressSteps,
        },
        progress_steps: progressSteps,
      };
    }
    if (cmd === "list_transform_engines") {
      return [
        {
          name: "d2",
          execution: "external",
          available: false,
          bundled: false,
          installationLabel: "Install D2 locally",
          setupHint: "Choose a trusted d2 executable path.",
          securitySummary: "Runs with explicit trust, timeout, and output limits.",
          adapterProfile: "d2-cli",
          diagnosticProfile: { versionProbe: "d2 --version", failureHint: "Configure D2.", stderrHint: "Review stderr.", successRelated: [], failureRelated: [], cacheKeyIncludes: [] },
          defaultCommand: "d2",
          requiresExecution: true,
          preferenceKey: "transform.d2.path",
          inputModes: ["stdin", "file"],
          limits: { timeoutMs: 5000, maxTimeoutMs: 30000, maxInputBytes: 65536, maxOutputBytes: 1048576 },
          cacheScope: "external",
          exportTargets: ["html", "pdf", "docx", "pptx"],
        },
      ];
    }
    if (cmd === "list_transform_handler_installers") return [];
    if (cmd === "default_markdown_reader_plan") return { supported: true, applied: false, platform: "screenshot", appName: "NEditor", fileExtensions: ["md"], message: "Ready.", commands: [], manual_steps: [] };
    if (cmd === "get_git_status") return { inside_repo: true, branch: "main", dirty: false, summary: ["clean working tree"] };
    if (cmd === "start_file_watcher") return { paths: [], native_watcher: true, watcher_error: null };
    if (cmd === "stop_file_watcher" || cmd === "write_desktop_ui_smoke_report" || cmd === "write_desktop_workflow_smoke_report") return null;
    if (cmd === "desktop_workflow_smoke_enabled" || cmd === "desktop_workflow_smoke_autorun_enabled") return false;
    return null;
  }

  window.__NEDITOR_E2E__ = {
    setFile,
    getFile: (path) => readMockFile(path).text,
    queueDialogSelection() {},
    queueConfirmResponse() {},
    setCompileDelay() {},
    deleteFile(path) {
      mockFiles.delete(normalizePath(path));
    },
    emitFileWatch() {},
    revealedPaths: () => [],
    setClipboardText() {},
  };
  window.__TAURI_INTERNALS__ = {
    metadata: { currentWindow: { label: "main" }, currentWebview: { label: "main" } },
    transformCallback(callback) {
      const id = callbackId++;
      callbacks.set(id, callback);
      return id;
    },
    unregisterCallback(id) {
      callbacks.delete(id);
    },
    invoke,
    convertFileSrc(path) {
      return path;
    },
  };
  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
  window.isTauri = true;
}

async function loadShowcase(page) {
  page.on("console", (message) => {
    if (["error", "warning"].includes(message.type())) console.error(`[browser:${message.type()}] ${message.text()}`);
  });
  page.on("pageerror", (error) => console.error(`[browser:pageerror] ${error.stack || error.message}`));
  await page.goto(baseUrl, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => Boolean(window.__NEDITOR_APP_E2E__), null, { timeout: 15_000 });
  await page.evaluate(async (path) => {
    const store = document.querySelector("#app")?.__vue_app__?.config?.globalProperties?.$pinia?._s?.get("documents");
    if (!store) throw new Error("NEditor documents store was not available.");
    await store.openPath(path);
    store.mode = "split";
    store.sidebar = "outline";
    await store.compileActive();
  }, rootShowcasePath);
  await page.getByRole("heading", { name: /NEditor Capability Showcase/ }).first().waitFor({ timeout: 15_000 });
}

async function configureState(page, { mode = "split", sidebar = "outline", exportTarget = "html", previewScroll = 0 }) {
  await page.evaluate(
    async ({ mode, sidebar, exportTarget, previewScroll }) => {
      const store = window.__vue_app__?.config?.globalProperties?.$pinia?._s?.get("documents");
      const appStore = document.querySelector("#app")?.__vue_app__?.config?.globalProperties?.$pinia?._s?.get("documents");
      if (!store && appStore) {
        await appStore.compileActive();
      }
      if (!store && !appStore) throw new Error("NEditor documents store was not available.");
      const documentsStore = store || appStore;
      documentsStore.mode = mode;
      documentsStore.sidebar = sidebar;
      documentsStore.exportTarget = exportTarget;
      if (sidebar === "exports") await documentsStore.prepareForExport();
      await documentsStore.compileActive();
      await new Promise((resolve) => requestAnimationFrame(() => requestAnimationFrame(resolve)));
      const preview = document.querySelector(".preview-pane");
      if (preview) preview.scrollTop = previewScroll;
    },
    { mode, sidebar, exportTarget, previewScroll },
  );
  await page.waitForTimeout(350);
}

async function capture(page, name) {
  const path = join(outputRoot, `${name}.png`);
  await page.screenshot({ path, fullPage: false });
  const size = statSync(path).size;
  if (size < 50_000) throw new Error(`${name}.png appears too small to be a useful screenshot (${size} bytes).`);
  console.log(`Wrote ${relative(root, path)} (${Math.round(size / 1024)} KB)`);
}

async function main() {
  if (!existsSync(showcaseRoot)) throw new Error(`Missing showcase directory: ${showcaseRoot}`);
  mkdirSync(outputRoot, { recursive: true });
  const files = readShowcaseFiles();
  const server = startServerIfNeeded();
  try {
    await waitForServer(baseUrl);
    const browser = await chromium.launch();
    try {
      const page = await browser.newPage({ viewport: { width: 1440, height: 980 }, deviceScaleFactor: 1 });
      await page.addInitScript(installNeditorMock, files);
      await loadShowcase(page);

      await configureState(page, { mode: "split", sidebar: "outline", previewScroll: 0 });
      await capture(page, "workbench");

      await configureState(page, { mode: "export", sidebar: "exports", exportTarget: "html", previewScroll: 0 });
      await capture(page, "export-readiness");

      await configureState(page, { mode: "review", sidebar: "review", previewScroll: 3100 });
      await capture(page, "review-governance");

      await configureState(page, { mode: "split", sidebar: "tables", previewScroll: 850 });
      await capture(page, "tables-transforms");
    } finally {
      await browser.close();
    }
  } finally {
    stopServer(server);
  }
}

main().catch((error) => {
  console.error(error);
  process.exitCode = 1;
});
