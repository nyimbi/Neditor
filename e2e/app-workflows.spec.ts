import { expect, test, type Page } from "@playwright/test";

async function installTauriMock(page: Page) {
  await page.addInitScript(() => {
    const callbacks = new Map<number, unknown>();
    const eventListeners = new Map<string, number[]>();
    const e2eStateKey = "__neditor_e2e_state__";
    const persistentState = (() => {
      try {
        const encoded = window.sessionStorage.getItem(e2eStateKey);
        if (encoded) return JSON.parse(encoded) as { files?: Array<[string, { text: string; hash: string; modified: string }]>; stores?: Record<string, Record<string, unknown>> };
      } catch {
        // Fall back to an in-memory mock if session storage is unavailable.
      }
      return { files: [], stores: {} as Record<string, Record<string, unknown>> };
    })();
    const storesByPath = new Map<string, Map<string, unknown>>(
      Object.entries(persistentState.stores || {}).map(([path, values]) => [path, new Map(Object.entries(values))]),
    );
    const storeResources = new Map<number, Map<string, unknown>>();
    const files = new Map<string, { text: string; hash: string; modified: string }>(persistentState.files || []);
    const dialogSelections: Array<string | null> = [];
    const revealedPaths: string[] = [];
    let callbackId = 1;
    let storeId = 1;

    function persistE2eState() {
      try {
        const stores = Object.fromEntries(Array.from(storesByPath.entries()).map(([path, values]) => [path, Object.fromEntries(values.entries())]));
        window.sessionStorage.setItem(e2eStateKey, JSON.stringify({ files: Array.from(files.entries()), stores }));
      } catch {
        // The browser tests can still exercise the current page without persistence.
      }
    }

    function hash(text: string) {
      let value = 0;
      for (let index = 0; index < text.length; index += 1) {
        value = (value * 31 + text.charCodeAt(index)) >>> 0;
      }
      return `mock-${value.toString(16)}`;
    }

    function normalizePath(path: string) {
      return path.replace(/\\/g, "/");
    }

    function dirname(path: string) {
      const normalized = normalizePath(path);
      const index = normalized.lastIndexOf("/");
      if (index <= 0) return "/";
      return normalized.slice(0, index);
    }

    function resolveRelativePath(basePath: string, target: string) {
      if (target.startsWith("/")) return normalizePath(target);
      const parts = `${dirname(basePath)}/${target}`.split("/");
      const stack: string[] = [];
      for (const part of parts) {
        if (!part || part === ".") continue;
        if (part === "..") {
          stack.pop();
        } else {
          stack.push(part);
        }
      }
      return `/${stack.join("/")}`;
    }

    function titleFromPath(path: string) {
      return normalizePath(path).split("/").pop() || path;
    }

    function includeTarget(line: string) {
      const trimmed = line.trim();
      const bang = trimmed.match(/^!include\s+(.+)$/);
      if (bang) return bang[1].trim().replace(/^["']|["']$/g, "");
      const braces = trimmed.match(/^\{\{\s*include\s+(.+?)\s*\}\}$/);
      if (braces) return braces[1].trim().replace(/^["']|["']$/g, "");
      const comment = trimmed.match(/^<!--\s*include:\s*(.+?)\s*-->$/);
      if (comment) return comment[1].trim().replace(/^["']|["']$/g, "");
      return "";
    }

    function stripFrontMatter(text: string) {
      return text.replace(/^---\r?\n[\s\S]*?\r?\n---\r?\n?/, "");
    }

    function setFile(path: string, text: string) {
      files.set(normalizePath(path), {
        text,
        hash: hash(text),
        modified: new Date(0).toISOString(),
      });
      persistE2eState();
    }

    function readMockFile(path: string) {
      const normalized = normalizePath(path);
      const file = files.get(normalized);
      if (!file) throw new Error(`Mock file not found: ${normalized}`);
      return { path: normalized, ...file };
    }

    if (!files.size) {
      setFile(
        "/workspace/market.md",
        [
          "---",
          "title: Market Entry Report",
          "status: draft",
          "---",
          "",
          "# Market Entry Report",
          "",
          "Original saved content.",
        ].join("\n"),
      );
      setFile(
        "/workspace/chapters/risk.md",
        [
          "---",
          "title: Risk Include",
          "---",
          "",
          "## Risk Notes",
          "",
          "Original included risk note.",
        ].join("\n"),
      );
    }

    function escapeHtml(text: string) {
      return text.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
    }

    function titleFromMarkdown(text: string) {
      const frontMatterTitle = text.match(/^---[\s\S]*?\ntitle:\s*(.+?)\n[\s\S]*?---/);
      if (frontMatterTitle) return frontMatterTitle[1].replace(/^["']|["']$/g, "").trim();
      return text.match(/^#\s+(.+)$/m)?.[1].trim() || "Untitled";
    }

    function statusFromMarkdown(text: string) {
      return text.match(/^---[\s\S]*?\nstatus:\s*(.+?)\n[\s\S]*?---/)?.[1].trim() || "draft";
    }

    function htmlFromMarkdown(text: string) {
      return text
        .split(/\r?\n/)
        .map((line) => {
          if (line.startsWith("# ")) return `<h1 id="${escapeHtml(line.slice(2).toLowerCase().replace(/\s+/g, "-"))}">${escapeHtml(line.slice(2))}</h1>`;
          if (line.startsWith("## ")) return `<h2 id="${escapeHtml(line.slice(3).toLowerCase().replace(/\s+/g, "-"))}">${escapeHtml(line.slice(3))}</h2>`;
          if (line.trim().startsWith("|")) return `<pre>${escapeHtml(line)}</pre>`;
          if (!line.trim() || line.trim() === "---") return "";
          return `<p>${escapeHtml(line)}</p>`;
        })
        .join("\n");
    }

    function headingsFromMarkdown(text: string) {
      return text.split(/\r?\n/).flatMap((line, index) => {
        const match = line.match(/^(#{1,6})\s+(.+)$/);
        if (!match) return [];
        const textValue = match[2].trim();
        return [
          {
            level: match[1].length,
            text: textValue,
            anchor: textValue.toLowerCase().replace(/\s+/g, "-"),
            line: index + 1,
          },
        ];
      });
    }

    function expandIncludes(text: string, filePath = "/workspace/market.md", depth = 0, seen = new Set<string>()) {
      const compiledLines: string[] = [];
      const includeGraph: Array<{ parent: string; child: string; depth: number }> = [];
      const includedFiles = new Map<string, { path: string; hash: string }>();
      const diagnostics: unknown[] = [];
      for (const line of text.split(/\r?\n/)) {
        const target = includeTarget(line);
        if (!target) {
          compiledLines.push(line);
          continue;
        }
        const childPath = resolveRelativePath(filePath, target);
        includeGraph.push({ parent: normalizePath(filePath), child: childPath, depth });
        const file = files.get(childPath);
        if (!file) {
          diagnostics.push({ severity: "error", message: `Missing include file: ${childPath}`, source: childPath });
          continue;
        }
        includedFiles.set(childPath, { path: childPath, hash: file.hash });
        if (seen.has(childPath)) {
          diagnostics.push({ severity: "error", message: `Circular include detected: ${childPath}`, source: childPath });
          continue;
        }
        const child = expandIncludes(stripFrontMatter(file.text), childPath, depth + 1, new Set([...seen, childPath]));
        compiledLines.push(child.compiled);
        child.includeGraph.forEach((edge) => includeGraph.push(edge));
        child.includedFiles.forEach((included) => includedFiles.set(included.path, included));
        child.diagnostics.forEach((diagnostic) => diagnostics.push(diagnostic));
      }
      return {
        compiled: compiledLines.join("\n"),
        includeGraph,
        includedFiles: Array.from(includedFiles.values()),
        diagnostics,
      };
    }

    function compileMarkdown(text: string, filePath = "/workspace/market.md") {
      const expanded = expandIncludes(text, filePath);
      const title = titleFromMarkdown(text);
      const status = statusFromMarkdown(text);
      const headings = headingsFromMarkdown(expanded.compiled);
      const sourceHash = hash(text);
      const metadata = { title, status, version: "1.0.0" };
      const diagnostics = expanded.diagnostics;
      const manifest = {
        document_title: title,
        document_version: "1.0.0",
        status,
        exported_at: new Date(0).toISOString(),
        source_hash: sourceHash,
        output_path: null,
        output_hash: null,
        included_files: expanded.includedFiles,
        media_files: [],
        layout_sections: [],
        export_target: "html",
        export_options: {},
        transform_artifacts: [],
        diagnostics,
        source_map: [],
        app_version: "e2e-mock",
      };
      return {
        compiled_markdown: expanded.compiled,
        html: htmlFromMarkdown(expanded.compiled),
        semantic: {
          title,
          status,
          headings,
          outline: headings,
          tables: (text.match(/^\|/gm) || []).length ? 1 : 0,
          table_summaries: [],
          figures: 0,
          equations: 0,
          citations: [],
          citation_references: [],
          duplicate_bibliography_keys: [],
          glossary: {},
          layout_directives: [],
          comments: [],
          change_notes: [],
          ai_sources: [],
          ai_assisted_sections: [],
          labels: [],
          cross_references: [],
        },
        document_ast: {
          metadata: { title, status, version: "1.0.0", source_hash: sourceHash },
          blocks: headings.map((heading) => ({
            kind: "heading",
            level: heading.level,
            text: heading.text,
            anchor: heading.anchor,
            line: heading.line,
            end_line: heading.line,
            source: null,
          })),
        },
        paged_document: { sections: [] },
        diagnostics,
        include_graph: expanded.includeGraph,
        source_map: [],
        metadata,
        bibliography: [],
        index_terms: [],
        formula_graph: [],
        formula_dependency_edges: [],
        transform_artifacts: [],
        export_manifest: manifest,
      };
    }

    async function invoke(cmd: string, args: Record<string, unknown> = {}) {
      if (cmd === "plugin:store|load") {
        const path = String(args.path || "settings.json");
        if (!storesByPath.has(path)) storesByPath.set(path, new Map());
        const rid = storeId;
        storeId += 1;
        storeResources.set(rid, storesByPath.get(path) as Map<string, unknown>);
        return rid;
      }
      if (cmd === "plugin:store|get") {
        const store = storeResources.get(args.rid as number);
        const key = String(args.key || "");
        const exists = Boolean(store?.has(key));
        return [exists ? store?.get(key) : undefined, exists];
      }
      if (cmd === "plugin:store|set") {
        const store = storeResources.get(args.rid as number);
        if (store) store.set(String(args.key || ""), args.value);
        persistE2eState();
        return null;
      }
      if (cmd === "plugin:store|save") {
        persistE2eState();
        return null;
      }
      if (cmd === "plugin:store|has") return Boolean(storeResources.get(args.rid as number)?.has(String(args.key || "")));
      if (cmd === "plugin:store|clear") {
        storeResources.get(args.rid as number)?.clear();
        persistE2eState();
        return null;
      }
      if (cmd.startsWith("plugin:store|")) return null;
      if (cmd === "plugin:event|listen") {
        const eventName = args.event as string;
        const handler = args.handler as number;
        eventListeners.set(eventName, [...(eventListeners.get(eventName) || []), handler]);
        return handler;
      }
      if (cmd === "plugin:event|unlisten") {
        const eventName = args.event as string;
        const id = args.eventId as number;
        eventListeners.set(
          eventName,
          (eventListeners.get(eventName) || []).filter((listenerId) => listenerId !== id),
        );
        return null;
      }
      if (cmd === "plugin:window|set_title") return null;
      if (cmd === "plugin:dialog|open") {
        return dialogSelections.length ? dialogSelections.shift() : "/workspace/market.md";
      }
      if (cmd === "plugin:dialog|save") {
        return dialogSelections.length ? dialogSelections.shift() : "/workspace/market.md";
      }
      if (cmd === "plugin:dialog|confirm") return true;
      if (cmd === "read_file") {
        return readMockFile(args.path as string);
      }
      if (cmd === "save_file") {
        const request = args.request as { path: string; text: string };
        const path = normalizePath(request.path);
        setFile(path, request.text);
        return readMockFile(path);
      }
      if (cmd === "file_metadata") {
        const path = normalizePath(args.path as string);
        const file = files.get(path);
        return {
          path,
          exists: Boolean(file),
          hash: file?.hash || null,
          modified: file?.modified || null,
        };
      }
      if (cmd === "rename_file") {
        const request = args.request as { from: string; to: string };
        const from = normalizePath(request.from);
        const to = normalizePath(request.to);
        const file = readMockFile(from);
        files.delete(from);
        setFile(to, file.text);
        return readMockFile(to);
      }
      if (cmd === "duplicate_file") {
        const request = args.request as { from: string; to: string };
        const source = readMockFile(request.from);
        const to = normalizePath(request.to);
        setFile(to, source.text);
        return readMockFile(to);
      }
      if (cmd === "reveal_path") {
        revealedPaths.push(normalizePath(args.path as string));
        return null;
      }
      if (cmd === "compile_document_with_options") {
        const request = args.request as { text: string; file_path?: string | null };
        return compileMarkdown(request.text, request.file_path || "/workspace/market.md");
      }
      if (cmd === "list_transform_engines") return [];
      if (cmd === "list_workspace_files") {
        const request = args.request as { root: string };
        const root = normalizePath(request.root).replace(/\/$/, "");
        const folders = new Set<string>();
        const entries = Array.from(files.keys()).flatMap((path) => {
          if (!path.startsWith(`${root}/`)) return [];
          const relativePath = path.slice(root.length + 1);
          const parts = relativePath.split("/");
          if (parts.length > 1) {
            folders.add(parts[0]);
          }
          return [
            {
              path,
              name: titleFromPath(path),
              relative_path: relativePath,
              kind: titleFromPath(path).split(".").pop() || "file",
              depth: parts.length - 1,
            },
          ];
        });
        return [
          ...Array.from(folders).map((folder) => ({
            path: `${root}/${folder}`,
            name: folder,
            relative_path: folder,
            kind: "directory",
            depth: 0,
          })),
          ...entries,
        ].sort((left, right) => left.relative_path.localeCompare(right.relative_path));
      }
      if (cmd === "list_snapshots") return [];
      if (cmd === "get_git_status") return { inside_repo: false, dirty: false, summary: [] };
      if (cmd === "start_file_watcher") {
        const request = args.request as { root: string; included?: string[] };
        const rootPath = normalizePath(request.root);
        const rootFile = files.get(rootPath);
        const includedFiles = (request.included || []).map((path) => {
          const normalizedPath = normalizePath(path);
          const file = files.get(normalizedPath);
          return {
            path: normalizedPath,
            exists: Boolean(file),
            hash: file?.hash || null,
            modified: file?.modified || null,
            role: "include",
          };
        });
        return {
          paths: [
            {
              path: rootPath,
              exists: Boolean(rootFile),
              hash: rootFile?.hash || null,
              modified: rootFile?.modified || null,
              role: "root",
            },
            ...includedFiles,
          ],
          native_watcher: true,
          watcher_error: null,
        };
      }
      if (cmd === "stop_file_watcher") return null;
      if (cmd === "prepare_for_export") {
        const request = args.request as { text: string };
        const response = compileMarkdown(request.text);
        return {
          ready: true,
          error_count: 0,
          warning_count: 0,
          info_count: 0,
          source_map: [],
          paged_document: response.paged_document,
          diagnostics: [],
          manifest: response.export_manifest,
        };
      }
      if (cmd === "cleanup_ai_paste") {
        const request = args.request as { text: string };
        return {
          cleaned_markdown: `Cleaned AI output\n\n${request.text.trim()}`,
          issues: ["Normalized AI paste in browser workflow test."],
          provenance_block: null,
        };
      }
      if (cmd === "create_snapshot") return { snapshot_path: "/tmp/mock-snapshot.md" };
      if (cmd === "export_document") return { output_path: "/tmp/mock-export.html", manifest_path: "/tmp/mock-export.html.manifest.json" };
      return null;
    }

    window.__NEDITOR_E2E__ = {
      queueDialogSelection(path: string | null) {
        dialogSelections.push(path);
      },
      getFile(path: string) {
        return readMockFile(path).text;
      },
      setFile(path: string, text: string) {
        setFile(path, text);
      },
      emitFileWatch(path: string, kind = "modify") {
        const listeners = eventListeners.get("neditor-file-watch-event") || [];
        for (const listenerId of listeners) {
          const callback = callbacks.get(listenerId) as ((event: unknown) => void) | undefined;
          callback?.({
            event: "neditor-file-watch-event",
            payload: { paths: [normalizePath(path)], kind },
          });
        }
      },
      revealedPaths() {
        return [...revealedPaths];
      },
    };

    window.__TAURI_INTERNALS__ = {
      metadata: {
        currentWindow: { label: "main" },
        currentWebview: { label: "main" },
      },
      transformCallback(callback: unknown) {
        const id = callbackId;
        callbackId += 1;
        callbacks.set(id, callback);
        return id;
      },
      unregisterCallback(id: number) {
        callbacks.delete(id);
      },
      invoke,
      convertFileSrc(path: string) {
        return path;
      },
    };
    window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
      unregisterListener() {},
    };
    window.isTauri = true;
  });
}

async function editorText(page: Page) {
  return page.locator(".cm-content").innerText();
}

async function queueDialogSelection(page: Page, path: string | null) {
  await page.evaluate((selectedPath) => window.__NEDITOR_E2E__.queueDialogSelection(selectedPath), path);
}

async function mockFileText(page: Page, path: string) {
  return page.evaluate((selectedPath) => window.__NEDITOR_E2E__.getFile(selectedPath), path);
}

async function setMockFileText(page: Page, path: string, text: string) {
  await page.evaluate(({ selectedPath, content }) => window.__NEDITOR_E2E__.setFile(selectedPath, content), { selectedPath: path, content: text });
}

async function emitMockFileWatch(page: Page, path: string, kind = "modify") {
  await page.evaluate(({ selectedPath, eventKind }) => window.__NEDITOR_E2E__.emitFileWatch(selectedPath, eventKind), { selectedPath: path, eventKind: kind });
}

async function revealedPaths(page: Page) {
  return page.evaluate(() => window.__NEDITOR_E2E__.revealedPaths());
}

async function activeFileRowText(page: Page) {
  return page.locator(".sidebar .file-row.active").innerText();
}

async function insertCleanedAiPaste(page: Page, text: string, mode = "insert") {
  await page.getByRole("button", { name: "AI Paste" }).click();
  await page.getByRole("textbox", { name: "Original" }).fill(text);
  await page.getByLabel("Insert mode").selectOption(mode);
  await page.getByRole("button", { name: "Insert cleaned" }).click();
  await expect(page.getByRole("dialog", { name: "AI paste cleanup" })).toBeHidden();
}

function externalApprovedDocument() {
  return [
    "---",
    "title: Market Entry Report",
    "status: approved",
    "---",
    "",
    "# Market Entry Report",
    "",
    "External disk edit.",
  ].join("\n");
}

test.beforeEach(async ({ page }) => {
  await installTauriMock(page);
  await page.goto("/");
});

test("boots the workbench and switches core view modes", async ({ page }) => {
  await expect(page.getByRole("heading", { name: "Market Entry Report" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeVisible();

  await page.getByLabel("View mode").selectOption("preview");
  await expect(page.getByRole("region", { name: "Markdown source" })).toBeHidden();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeVisible();

  await page.getByLabel("View mode").selectOption("source");
  await expect(page.getByRole("region", { name: "Markdown source" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeHidden();

  await page.getByLabel("View mode").selectOption("split");
  await expect(page.getByRole("region", { name: "Markdown source" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeVisible();
});

test("runs command palette insertion and table editor workflows", async ({ page }) => {
  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Insert table");
  await page.getByRole("button", { name: "Insert table Snippet" }).click();
  await expect.poll(() => editorText(page)).toContain("| Item | Value |");

  await page.getByLabel("Sidebar panel").selectOption("tables");
  await page.getByRole("button", { name: "New table" }).click();
  await page.getByLabel("Caption").fill("Workflow budget");
  await page.getByRole("button", { name: "Add totals row" }).click();
  await page.getByRole("button", { name: "Insert table" }).click();

  await expect.poll(() => editorText(page)).toContain("Table: Workflow budget");
  await expect.poll(() => editorText(page)).toContain("Total");
});

test("opens, saves, duplicates, renames, reveals, and reverts mocked files", async ({ page }) => {
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  await expect.poll(() => editorText(page)).toContain("Original saved content.");
  await expect(page.getByText("/workspace")).toBeVisible();
  await expect(page.getByRole("button", { name: /market\.md/ }).first()).toBeVisible();

  await page.getByLabel("Sidebar panel").selectOption("review");
  await page.locator(".sidebar").getByLabel("Status").selectOption("in-review");
  await expect.poll(() => editorText(page)).toContain("status: in-review");
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("status: in-review");

  await queueDialogSelection(page, "/workspace/market copy.md");
  await page.getByRole("button", { name: "Duplicate" }).click();
  await expect.poll(() => mockFileText(page, "/workspace/market copy.md")).toContain("status: in-review");
  await page.getByLabel("Sidebar panel").selectOption("files");
  await page.locator(".sidebar").getByRole("button", { name: /market copy\.md/ }).click();
  await expect.poll(() => activeFileRowText(page)).toContain("market copy.md");

  await queueDialogSelection(page, "/workspace/renamed.md");
  await page.getByRole("button", { name: "Rename" }).click();
  await expect.poll(() => activeFileRowText(page)).toContain("renamed.md");

  await page.locator(".document-tabs .tab.active").getByLabel("Pin document").click();
  await expect(page.getByLabel("Pinned tabs").getByRole("button", { name: /renamed\.md/ })).toBeVisible();

  await page.getByRole("button", { name: "Reveal" }).click();
  await expect.poll(() => revealedPaths(page)).toContain("/workspace/renamed.md");

  await setMockFileText(page, "/workspace/renamed.md", "# Renamed\n\nDisk version after rename.");
  await page.getByRole("button", { name: "Revert" }).click();
  await expect.poll(() => editorText(page)).toContain("Disk version after rename.");
});

test("saves a document as a new file and reopens it from recently closed", async ({ page }) => {
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  await page.getByLabel("Sidebar panel").selectOption("review");
  await page.locator(".sidebar").getByLabel("Status").selectOption("approved");
  await expect.poll(() => editorText(page)).toContain("status: approved");

  await queueDialogSelection(page, "/workspace/market-approved.md");
  await page.getByRole("button", { name: "Save As" }).click();
  await expect.poll(() => mockFileText(page, "/workspace/market-approved.md")).toContain("status: approved");
  await expect(page.locator(".document-tabs .tab.active")).toContainText("market-approved.md");

  await page.locator(".document-tabs .tab.active").getByLabel("Close document").click();
  await expect(page.locator(".document-tabs .tab.active")).not.toContainText("market-approved.md");

  await page.getByLabel("Sidebar panel").selectOption("settings");
  const recentlyClosed = page.getByLabel("Recently closed documents");
  await recentlyClosed.getByRole("button", { name: "/workspace/market-approved.md" }).click();

  await expect.poll(() => editorText(page)).toContain("status: approved");
  await expect(recentlyClosed.getByRole("button", { name: "/workspace/market-approved.md" })).toHaveCount(0);
  await page.getByLabel("Sidebar panel").selectOption("files");
  await expect.poll(() => activeFileRowText(page)).toContain("market-approved.md");
});

test("restores workspace tabs, active document, pins, mode, and sidebar after reload", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/field-notes.md",
    [
      "---",
      "title: Field Notes",
      "status: draft",
      "---",
      "",
      "# Field Notes",
      "",
      "Reloaded workspace note.",
    ].join("\n"),
  );

  await queueDialogSelection(page, "/workspace/field-notes.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  const marketTab = page.locator(".document-tabs .tab").filter({ hasText: "Market Entry Report" });
  await marketTab.getByLabel("Pin document").click();
  await expect(page.getByLabel("Pinned tabs").getByRole("button", { name: /Market Entry Report/ })).toBeVisible();
  await expect(page.locator(".document-tabs .tab.active")).toContainText("Field Notes");

  await page.getByLabel("View mode").selectOption("review");
  await page.getByLabel("Sidebar panel").selectOption("settings");
  await page.getByRole("button", { name: "Save Workspace" }).click();

  await page.reload();

  await expect(page.getByLabel("View mode")).toHaveValue("review");
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("settings");
  await expect(page.locator(".document-tabs .tab")).toHaveCount(2);
  await expect(page.getByLabel("Pinned tabs").getByRole("button", { name: /Market Entry Report/ })).toBeVisible();
  await expect(page.locator(".document-tabs .tab.active")).toContainText("Field Notes");
  await expect.poll(() => editorText(page)).toContain("Reloaded workspace note.");
  await expect(page.getByLabel("Recent files").getByRole("button", { name: "/workspace/field-notes.md" })).toBeVisible();

  await page.getByLabel("Sidebar panel").selectOption("files");
  await expect(page.getByText("/workspace")).toBeVisible();
  await expect.poll(() => activeFileRowText(page)).toContain("field-notes.md");
});

test("blocks stale saves and preserves local conflict copies", async ({ page }) => {
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  await page.getByLabel("Sidebar panel").selectOption("review");
  await page.locator(".sidebar").getByLabel("Status").selectOption("in-review");
  await expect.poll(() => editorText(page)).toContain("status: in-review");

  await setMockFileText(page, "/workspace/market.md", externalApprovedDocument());

  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect(page.getByText("Save blocked; resolve external changes first")).toBeVisible();

  const conflictActions = page.locator(".status-bar .conflict-actions");
  await expect(conflictActions.getByRole("button", { name: "Compare" })).toBeVisible();
  await expect(conflictActions.getByRole("button", { name: "Accept external" })).toBeVisible();
  await expect(conflictActions.getByRole("button", { name: "Keep local" })).toBeVisible();
  await expect(conflictActions.getByRole("button", { name: "Save copy" })).toBeVisible();

  await conflictActions.getByRole("button", { name: "Compare" }).click();
  const conflictDialog = page.getByRole("dialog", { name: "External file conflict" });
  await expect(conflictDialog).toBeVisible();
  await expect(conflictDialog).toContainText("The root file changed outside NEditor before save.");
  await expect(conflictDialog).toContainText("status: in-review");
  await expect(conflictDialog).toContainText("status: approved");
  await expect(conflictDialog).toContainText("External disk edit.");

  await queueDialogSelection(page, "/workspace/market local copy.md");
  await conflictDialog.getByRole("button", { name: "Save copy" }).click();

  await expect(conflictDialog).toBeHidden();
  await expect(page.getByText("Saved local edits as a copy")).toBeVisible();
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("External disk edit.");
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("status: approved");
  await expect.poll(() => mockFileText(page, "/workspace/market local copy.md")).toContain("status: in-review");
  await page.getByLabel("Sidebar panel").selectOption("files");
  await expect.poll(() => activeFileRowText(page)).toContain("market local copy.md");
});

test("merges external conflict text back into the original file", async ({ page }) => {
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  await page.getByLabel("Sidebar panel").selectOption("review");
  await page.locator(".sidebar").getByLabel("Status").selectOption("in-review");
  await setMockFileText(page, "/workspace/market.md", externalApprovedDocument());

  await page.getByRole("button", { name: "Save", exact: true }).click();
  await page.locator(".status-bar .conflict-actions").getByRole("button", { name: "Compare" }).click();

  const conflictDialog = page.getByRole("dialog", { name: "External file conflict" });
  await conflictDialog.getByRole("button", { name: "Use external as merge base" }).click();
  await conflictDialog
    .getByLabel("Merged result")
    .fill(
      [
        "---",
        "title: Market Entry Report",
        "status: approved",
        "---",
        "",
        "# Market Entry Report",
        "",
        "External disk edit.",
        "Local reviewer note retained.",
      ].join("\n"),
    );
  await conflictDialog.getByRole("button", { name: "Apply merged text" }).click();

  await expect(conflictDialog).toBeHidden();
  await expect.poll(() => editorText(page)).toContain("Local reviewer note retained.");

  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("Local reviewer note retained.");
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("status: approved");
});

test("keeps local edits after reviewing an external conflict", async ({ page }) => {
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  await page.getByLabel("Sidebar panel").selectOption("review");
  await page.locator(".sidebar").getByLabel("Status").selectOption("in-review");
  await setMockFileText(page, "/workspace/market.md", externalApprovedDocument());

  await page.getByRole("button", { name: "Save", exact: true }).click();
  const conflictActions = page.locator(".status-bar .conflict-actions");
  await expect(conflictActions.getByRole("button", { name: "Keep local" })).toBeVisible();
  await conflictActions.getByRole("button", { name: "Keep local" }).click();

  await expect(conflictActions).toHaveCount(0);
  await expect.poll(() => editorText(page)).toContain("status: in-review");
  await expect.poll(() => editorText(page)).not.toContain("External disk edit.");
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("status: approved");
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("External disk edit.");
});

test("accepts external conflict changes into the active document", async ({ page }) => {
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  await page.getByLabel("Sidebar panel").selectOption("review");
  await page.locator(".sidebar").getByLabel("Status").selectOption("in-review");
  await setMockFileText(page, "/workspace/market.md", externalApprovedDocument());

  await page.getByRole("button", { name: "Save", exact: true }).click();
  const conflictActions = page.locator(".status-bar .conflict-actions");
  await expect(conflictActions.getByRole("button", { name: "Accept external" })).toBeVisible();
  await conflictActions.getByRole("button", { name: "Accept external" }).click();

  await expect(conflictActions).toHaveCount(0);
  await expect.poll(() => editorText(page)).toContain("status: approved");
  await expect.poll(() => editorText(page)).toContain("External disk edit.");
  await expect.poll(() => editorText(page)).not.toContain("status: in-review");
  await expect(page.locator(".document-tabs .tab.active")).not.toContainText("*");
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toBe(externalApprovedDocument());
});

test("reloads clean documents after watcher-originated external edits", async ({ page }) => {
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await expect(page.locator(".status-bar")).toContainText("Native watch: 1 path");

  await setMockFileText(page, "/workspace/market.md", externalApprovedDocument());
  await emitMockFileWatch(page, "/workspace/market.md");

  await expect(page.locator(".status-bar")).toContainText("Reloaded external changes");
  await expect(page.locator(".status-bar .conflict-actions")).toHaveCount(0);
  await expect.poll(() => editorText(page)).toContain("status: approved");
  await expect.poll(() => editorText(page)).toContain("External disk edit.");
  await expect(page.locator(".document-tabs .tab.active")).not.toContainText("*");
});

test("opens a root-file conflict when watcher events arrive during local edits", async ({ page }) => {
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await expect(page.locator(".status-bar")).toContainText("Native watch: 1 path");

  await page.getByLabel("Sidebar panel").selectOption("review");
  await page.locator(".sidebar").getByLabel("Status").selectOption("in-review");
  await expect.poll(() => editorText(page)).toContain("status: in-review");

  await setMockFileText(page, "/workspace/market.md", externalApprovedDocument());
  await emitMockFileWatch(page, "/workspace/market.md");

  await expect(page.getByText("External changes detected; compare before overwriting")).toBeVisible();
  const conflictActions = page.locator(".status-bar .conflict-actions");
  await expect(conflictActions.getByRole("button", { name: "Compare" })).toBeVisible();
  await conflictActions.getByRole("button", { name: "Compare" }).click();

  const conflictDialog = page.getByRole("dialog", { name: "External file conflict" });
  await expect(conflictDialog).toBeVisible();
  await expect(conflictDialog).toContainText("The root file changed outside NEditor while local edits are unsaved.");
  await expect(conflictDialog).toContainText("status: in-review");
  await expect(conflictDialog).toContainText("status: approved");
  await expect(conflictDialog).toContainText("External disk edit.");
});

test("recompiles clean master documents after included files change", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/market.md",
    [
      "---",
      "title: Market Entry Report",
      "status: draft",
      "---",
      "",
      "# Market Entry Report",
      "",
      "!include chapters/risk.md",
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  const preview = page.getByRole("region", { name: "Live preview" });
  await expect(preview).toContainText("Original included risk note.");
  await page.getByLabel("Sidebar panel").selectOption("references");
  await expect(page.getByRole("button", { name: "/workspace/chapters/risk.md" })).toBeVisible();

  await setMockFileText(page, "/workspace/chapters/risk.md", "## Risk Notes\n\nUpdated included risk note.");
  await emitMockFileWatch(page, "/workspace/chapters/risk.md");

  await expect(preview).toContainText("Updated included risk note.");
  await expect.poll(() => editorText(page)).toContain("!include chapters/risk.md");
  await expect.poll(() => editorText(page)).not.toContain("Updated included risk note.");
});

test("opens included-file conflicts without overwriting dirty master drafts", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/market.md",
    [
      "---",
      "title: Market Entry Report",
      "status: draft",
      "---",
      "",
      "# Market Entry Report",
      "",
      "!include chapters/risk.md",
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await expect(page.getByRole("region", { name: "Live preview" })).toContainText("Original included risk note.");

  await page.getByLabel("Sidebar panel").selectOption("review");
  await page.locator(".sidebar").getByLabel("Status").selectOption("in-review");
  await expect.poll(() => editorText(page)).toContain("status: in-review");

  await setMockFileText(page, "/workspace/chapters/risk.md", "## Risk Notes\n\nDirty included risk note.");
  await emitMockFileWatch(page, "/workspace/chapters/risk.md");

  const conflictActions = page.locator(".status-bar .conflict-actions");
  await expect(conflictActions.getByRole("button", { name: "Compare" })).toBeVisible();
  await conflictActions.getByRole("button", { name: "Compare" }).click();

  const conflictDialog = page.getByRole("dialog", { name: "External file conflict" });
  await expect(conflictDialog).toContainText("An included file changed while local edits are unsaved.");
  await expect(conflictDialog).toContainText("/workspace/chapters/risk.md");
  await expect(conflictDialog).toContainText("Dirty included risk note.");

  await conflictDialog.getByRole("button", { name: "Accept external" }).click();
  await expect(conflictDialog).toBeHidden();
  await expect.poll(() => editorText(page)).toContain("status: in-review");
  await expect.poll(() => editorText(page)).toContain("!include chapters/risk.md");
  await expect(page.getByRole("region", { name: "Live preview" })).toContainText("Dirty included risk note.");
});

test("edits pasted tables with sorting, formulas, and merged cells", async ({ page }) => {
  await page.getByLabel("Sidebar panel").selectOption("tables");
  await page.getByRole("button", { name: "New table" }).click();

  await page.getByLabel("CSV/TSV paste").fill(
    [
      "Table: Regional sales {#tbl:sales}",
      "| Region | Revenue | Margin |",
      "| --- | ---: | ---: |",
      "| West | 900 | 0.12 |",
      "| East | 1200 | 0.18 |",
    ].join("\n"),
  );
  await page.getByRole("button", { name: "Replace from paste" }).click();

  const tableGrid = page.locator(".table-editor-grid");
  const markdownPreview = page.getByLabel("Markdown preview");
  await expect(markdownPreview).toHaveValue(/Table: Regional sales \{#tbl:sales\}/);
  await expect(page.getByLabel("Revenue, row 1, column B")).toHaveValue("900");

  await tableGrid.getByRole("button", { name: "Desc" }).nth(1).click();
  await expect(page.getByLabel("Region, row 1, column A")).toHaveValue("East");
  await expect(page.getByLabel("Revenue, row 1, column B")).toHaveValue("1200");

  await page.getByRole("region", { name: "Table formula builder" }).getByLabel("Function").selectOption("AVG");
  await page.getByRole("region", { name: "Table formula builder" }).getByLabel("Target").selectOption("2");
  await page.getByRole("region", { name: "Table formula builder" }).getByLabel("Label").fill("Average");
  await page.getByRole("button", { name: "Add formula row" }).click();
  await expect(markdownPreview).toHaveValue(/Average\s+\|\s+\|\s+=AVG\(C1:C2\)/);

  await page.getByRole("region", { name: "Merged table cells" }).getByLabel("Columns").fill("2");
  await page.getByRole("button", { name: "Merge cell" }).click();
  await expect(markdownPreview).toHaveValue(/East \{colspan=2\}/);

  await page.getByRole("button", { name: "Insert table" }).click();
  await expect.poll(() => editorText(page)).toContain("Table: Regional sales {#tbl:sales}");
  await expect.poll(() => editorText(page)).toContain("East {colspan=2}");
  await expect.poll(() => editorText(page)).toContain("=AVG(C1:C2)");
});

test("edits table structure with formats and cancels draft changes", async ({ page }) => {
  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Insert table");
  await page.getByRole("button", { name: "Insert table Snippet" }).click();
  await expect.poll(() => editorText(page)).toContain("| Revenue | 125000 |");

  await page.getByLabel("Sidebar panel").selectOption("tables");
  const markdownPreview = page.getByLabel("Markdown preview");
  await expect(page.getByLabel("Value, row 1, column B")).toHaveValue("125000");

  await page.getByLabel("Column B format").selectOption("currency");
  await expect(page.locator(".table-editor-grid output").nth(1)).toHaveText("$125000");

  await page.getByRole("button", { name: "Add row" }).click();
  await page.getByLabel("Item, row 2, column A").fill("Cost");
  await page.getByLabel("Value, row 2, column B").fill("74000");
  await expect(markdownPreview).toHaveValue(/Cost\s+\|\s+\$74000/);

  await page.getByRole("button", { name: "Remove row 2" }).click();
  await expect(markdownPreview).not.toHaveValue(/Cost/);

  await page.getByRole("button", { name: "Add column" }).click();
  await page.getByLabel("Column C header").fill("Margin");
  await page.getByLabel("Margin, row 1, column C").fill("0.42");
  await expect(markdownPreview).toHaveValue(/Margin/);
  await expect(markdownPreview).toHaveValue(/0\.42/);

  await page.getByRole("button", { name: "Remove column C" }).click();
  await expect(markdownPreview).not.toHaveValue(/Margin/);

  await page.getByLabel("Value, row 1, column B").fill("999");
  await expect(markdownPreview).toHaveValue(/999/);
  await page.getByRole("button", { name: "Cancel table edit" }).click();
  await expect(page.getByLabel("Value, row 1, column B")).toHaveValue("125000");
  await expect.poll(() => editorText(page)).not.toContain("999");

  await page.getByLabel("Value, row 1, column B").fill("250000");
  await page.getByRole("button", { name: "Apply" }).click();
  await expect.poll(() => editorText(page)).toContain("| Revenue | 250000 |");
  await expect.poll(() => editorText(page)).not.toContain("| Revenue | 125000 |");
});

test("previews and inserts cleaned AI paste through the modal", async ({ page }) => {
  await page.getByRole("button", { name: "AI Paste" }).click();
  await page.getByRole("textbox", { name: "Original" }).fill("Assistant: Revenue grew 24%.");
  await page.getByRole("button", { name: "Preview cleanup" }).click();

  await expect(page.getByRole("textbox", { name: "Cleaned preview" })).toHaveValue(/Cleaned AI output/);
  await page.getByRole("button", { name: "Insert cleaned" }).click();
  await expect.poll(() => editorText(page)).toContain("Cleaned AI output");
  await expect(page.getByRole("dialog", { name: "AI paste cleanup" })).toBeHidden();
});

test("applies AI paste quote appendix and section merge modes", async ({ page }) => {
  await insertCleanedAiPaste(page, "Assistant: Quote this.", "quote");
  await expect.poll(() => editorText(page)).toContain("> Cleaned AI output");
  await expect.poll(() => editorText(page)).toContain("> Assistant: Quote this.");

  await insertCleanedAiPaste(page, "Assistant: Put this in an appendix.", "appendix");
  await expect.poll(() => editorText(page)).toContain("## AI Draft Appendix");
  await expect.poll(() => editorText(page)).toContain("Assistant: Put this in an appendix.");

  await page.locator(".cm-content").click();
  await page.keyboard.press("Control+End");
  await insertCleanedAiPaste(page, "Assistant: Merge this into the active section.", "section");
  await expect.poll(() => editorText(page)).toContain("Assistant: Merge this into the active section.");
});

test("replaces the full document with cleaned AI paste", async ({ page }) => {
  await insertCleanedAiPaste(page, "Assistant: Replace the whole draft.", "replace");

  await expect.poll(() => editorText(page)).toContain("Cleaned AI output");
  await expect.poll(() => editorText(page)).toContain("Assistant: Replace the whole draft.");
  await expect.poll(() => editorText(page)).not.toContain("Original saved content.");
  await expect.poll(() => editorText(page)).not.toContain("Market Entry Report");
});

test("replaces the selected source with cleaned AI paste", async ({ page }) => {
  await page.locator(".cm-content").click();
  await page.keyboard.press("Control+A");

  await insertCleanedAiPaste(page, "Assistant: Replace the selected draft.", "selection");

  await expect.poll(() => editorText(page)).toContain("Cleaned AI output");
  await expect.poll(() => editorText(page)).toContain("Assistant: Replace the selected draft.");
  await expect.poll(() => editorText(page)).not.toContain("Original saved content.");
});

test("opens export readiness from the export sidebar", async ({ page }) => {
  await page.getByLabel("Sidebar panel").selectOption("exports");
  await page.getByRole("button", { name: "Prepare for export" }).click();

  await expect(page.locator("article.readiness").getByText("Ready", { exact: true })).toBeVisible();
  await expect(page.getByText("0 errors, 0 warnings, 0 info")).toBeVisible();
});
