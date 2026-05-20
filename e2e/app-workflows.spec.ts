import { expect, test, type Page } from "@playwright/test";

async function installTauriMock(page: Page) {
  await page.addInitScript(() => {
    const callbacks = new Map<number, unknown>();
    const stores = new Map<number, Map<string, unknown>>();
    const files = new Map<string, { text: string; hash: string; modified: string }>();
    const dialogSelections: Array<string | null> = [];
    const revealedPaths: string[] = [];
    let callbackId = 1;
    let storeId = 1;

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

    function titleFromPath(path: string) {
      return normalizePath(path).split("/").pop() || path;
    }

    function setFile(path: string, text: string) {
      files.set(normalizePath(path), {
        text,
        hash: hash(text),
        modified: new Date(0).toISOString(),
      });
    }

    function readMockFile(path: string) {
      const normalized = normalizePath(path);
      const file = files.get(normalized);
      if (!file) throw new Error(`Mock file not found: ${normalized}`);
      return { path: normalized, ...file };
    }

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

    function compileMarkdown(text: string) {
      const title = titleFromMarkdown(text);
      const status = statusFromMarkdown(text);
      const headings = headingsFromMarkdown(text);
      const sourceHash = hash(text);
      const metadata = { title, status, version: "1.0.0" };
      const diagnostics: unknown[] = [];
      const manifest = {
        document_title: title,
        document_version: "1.0.0",
        status,
        exported_at: new Date(0).toISOString(),
        source_hash: sourceHash,
        output_path: null,
        output_hash: null,
        included_files: [],
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
        compiled_markdown: text,
        html: htmlFromMarkdown(text),
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
        include_graph: [],
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
        const rid = storeId;
        storeId += 1;
        stores.set(rid, new Map());
        return rid;
      }
      if (cmd === "plugin:store|get") return [undefined, false];
      if (cmd.startsWith("plugin:store|")) return null;
      if (cmd === "plugin:event|listen") return callbackId++;
      if (cmd === "plugin:event|unlisten") return null;
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
        const request = args.request as { text: string };
        return compileMarkdown(request.text);
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
      if (cmd === "start_file_watcher") return { paths: [], native_watcher: false, watcher_error: null };
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

async function replaceEditorText(page: Page, text: string) {
  await page.locator(".cm-content").click();
  await page.keyboard.press("ControlOrMeta+A");
  await page.keyboard.insertText(text);
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

async function revealedPaths(page: Page) {
  return page.evaluate(() => window.__NEDITOR_E2E__.revealedPaths());
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

  await replaceEditorText(
    page,
    [
      "---",
      "title: Market Entry Report",
      "status: draft",
      "---",
      "",
      "# Market Entry Report",
      "",
      "Saved from browser workflow.",
    ].join("\n"),
  );
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("Saved from browser workflow.");

  await queueDialogSelection(page, "/workspace/market copy.md");
  await page.getByRole("button", { name: "Duplicate" }).click();
  await expect(page.getByRole("button", { name: /market copy\.md/ })).toBeVisible();
  await expect.poll(() => mockFileText(page, "/workspace/market copy.md")).toContain("Saved from browser workflow.");

  await queueDialogSelection(page, "/workspace/renamed.md");
  await page.getByRole("button", { name: "Rename" }).click();
  await expect(page.getByRole("button", { name: /renamed\.md/ })).toBeVisible();

  await page.getByLabel("Pin document").click();
  await expect(page.getByLabel("Pinned tabs").getByRole("button", { name: /renamed\.md/ })).toBeVisible();

  await page.getByRole("button", { name: "Reveal" }).click();
  await expect.poll(() => revealedPaths(page)).toContain("/workspace/renamed.md");

  await setMockFileText(page, "/workspace/renamed.md", "# Renamed\n\nDisk version after rename.");
  await replaceEditorText(page, "# Renamed\n\nLocal unsaved edit.");
  await page.getByRole("button", { name: "Revert" }).click();
  await expect.poll(() => editorText(page)).toContain("Disk version after rename.");
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

test("previews and inserts cleaned AI paste through the modal", async ({ page }) => {
  await page.getByRole("button", { name: "AI Paste" }).click();
  await page.getByRole("textbox", { name: "Original" }).fill("Assistant: Revenue grew 24%.");
  await page.getByRole("button", { name: "Preview cleanup" }).click();

  await expect(page.getByRole("textbox", { name: "Cleaned preview" })).toHaveValue(/Cleaned AI output/);
  await page.getByRole("button", { name: "Insert cleaned" }).click();
  await expect.poll(() => editorText(page)).toContain("Cleaned AI output");
  await expect(page.getByRole("dialog", { name: "AI paste cleanup" })).toBeHidden();
});

test("opens export readiness from the export sidebar", async ({ page }) => {
  await page.getByLabel("Sidebar panel").selectOption("exports");
  await page.getByRole("button", { name: "Prepare for export" }).click();

  await expect(page.locator("article.readiness").getByText("Ready", { exact: true })).toBeVisible();
  await expect(page.getByText("0 errors, 0 warnings, 0 info")).toBeVisible();
});
