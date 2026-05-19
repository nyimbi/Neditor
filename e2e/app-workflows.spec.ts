import { expect, test, type Page } from "@playwright/test";

async function installTauriMock(page: Page) {
  await page.addInitScript(() => {
    const callbacks = new Map<number, unknown>();
    const stores = new Map<number, Map<string, unknown>>();
    let callbackId = 1;
    let storeId = 1;

    function hash(text: string) {
      let value = 0;
      for (let index = 0; index < text.length; index += 1) {
        value = (value * 31 + text.charCodeAt(index)) >>> 0;
      }
      return `mock-${value.toString(16)}`;
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
      if (cmd === "compile_document_with_options") {
        const request = args.request as { text: string };
        return compileMarkdown(request.text);
      }
      if (cmd === "list_transform_engines") return [];
      if (cmd === "list_workspace_files") return [];
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

  await expect(page.getByText("Ready")).toBeVisible();
  await expect(page.getByText("0 errors, 0 warnings, 0 info")).toBeVisible();
});
