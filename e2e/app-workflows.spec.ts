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
    const confirmResponses: boolean[] = [];
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
      const frontMatterTitle = frontMatterValue(text, "title");
      if (frontMatterTitle) return frontMatterTitle;
      return text.match(/^#\s+(.+)$/m)?.[1].trim() || "Untitled";
    }

    function statusFromMarkdown(text: string) {
      return frontMatterValue(text, "status") || "draft";
    }

    function frontMatterValue(text: string, key: string) {
      const frontMatter = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
      if (!frontMatter) return "";
      const escapedKey = key.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      const match = frontMatter[1].match(new RegExp(`^${escapedKey}:\\s*(.+?)\\s*$`, "m"));
      return match?.[1].replace(/^["']|["']$/g, "").trim() || "";
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

    function citationReferencesFromMarkdown(text: string) {
      return text.split(/\r?\n/).flatMap((line, lineIndex) => {
        const references: Array<{ key: string; locator: string | null; line: number; column: number; end_column: number }> = [];
        const citationPattern = /\[[^\]\n]*@([A-Za-z0-9_:-]+)([^\]\n]*)\]/g;
        for (const match of line.matchAll(citationPattern)) {
          if (match.index === undefined) continue;
          const locator = match[2]?.replace(/^,\s*/, "").trim() || null;
          references.push({
            key: match[1],
            locator,
            line: lineIndex + 1,
            column: match.index + 1,
            end_column: match.index + match[0].length + 1,
          });
        }
        return references;
      });
    }

    function bibliographyFromMarkdown(text: string) {
      const entries: Array<{ key: string; title: string; author: string | null; issued: string | null }> = [];
      for (const fence of text.matchAll(/```(?:bibtex|bibliography)\s*\n([\s\S]*?)```/g)) {
        for (const entry of fence[1].matchAll(/@\w+\s*\{\s*([^,\s]+)\s*,([\s\S]*?)(?=\n@\w+\s*\{|$)/g)) {
          const body = entry[2] || "";
          entries.push({
            key: entry[1].trim(),
            title: bibtexField(body, "title") || entry[1].trim(),
            author: bibtexField(body, "author"),
            issued: bibtexField(body, "year"),
          });
        }
      }
      return entries;
    }

    function bibtexField(body: string, field: string) {
      const match = body.match(new RegExp(`${field}\\s*=\\s*[{\"]([^}\"]+)`, "i"));
      return match?.[1].trim() || null;
    }

    function duplicateBibliographyKeys(entries: Array<{ key: string }>) {
      const counts = new Map<string, number>();
      for (const entry of entries) counts.set(entry.key, (counts.get(entry.key) || 0) + 1);
      return Array.from(counts.entries()).filter(([, count]) => count > 1).map(([key]) => key);
    }

    function glossaryFromMarkdown(text: string) {
      const glossary: Record<string, string> = {};
      const glossaryBlockPattern = /```glossary\s*\n([\s\S]*?)```/g;
      for (const block of text.matchAll(glossaryBlockPattern)) {
        for (const line of block[1].split(/\r?\n/)) {
          const match = line.match(/^\s*([^:]+):\s*(.+?)\s*$/);
          if (match) glossary[match[1].trim()] = match[2].trim();
        }
      }
      return glossary;
    }

    function indexTermsFromMarkdown(text: string) {
      const terms = new Set<string>();
      for (const match of text.matchAll(/#index:([^}\n]+)/g)) {
        terms.add(match[1].trim());
      }
      return Array.from(terms);
    }

    function mockTransformEngines() {
      return [
        {
          name: "d2",
          execution: "external",
          available: false,
          bundled: false,
          installationLabel: "Install D2 locally",
          setupHint: "Choose a trusted d2 executable path.",
          securitySummary: "Runs only with explicit trust, timeout, and output limits.",
          adapterProfile: "d2-cli",
          diagnosticProfile: {
            versionProbe: "d2 --version",
            failureHint: "Check that the configured D2 executable exists and is trusted.",
            stderrHint: "D2 writes syntax and executable errors to stderr.",
            successRelated: ["External D2 probe succeeded."],
            failureRelated: ["External D2 probe failed."],
            cacheKeyIncludes: ["path", "input_mode", "timeout"],
          },
          defaultCommand: "d2",
          requiresExecution: true,
          preferenceKey: "transform.d2.path",
          inputModes: ["stdin", "file"],
          limits: {
            timeoutMs: 5000,
            maxTimeoutMs: 30000,
            maxInputBytes: 65536,
            maxOutputBytes: 1048576,
          },
          cacheScope: "external",
          exportTargets: ["html", "pdf", "docx", "pptx"],
        },
      ];
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
      const documentSet = frontMatterValue(text, "documentSet") || frontMatterValue(text, "document_set") || frontMatterValue(text, "set");
      const headings = headingsFromMarkdown(expanded.compiled);
      const citationReferences = citationReferencesFromMarkdown(expanded.compiled);
      const bibliography = bibliographyFromMarkdown(expanded.compiled);
      const duplicateBibliographyKeys = duplicateBibliographyKeys(bibliography);
      const glossary = glossaryFromMarkdown(expanded.compiled);
      const indexTerms = indexTermsFromMarkdown(expanded.compiled);
      const sourceHash = hash(text);
      const metadata = {
        title,
        status,
        version: "1.0.0",
        ...(documentSet ? { documentSet, document_set: documentSet } : {}),
      };
      const diagnostics = expanded.diagnostics;
      const diagnosticLineIndex = expanded.compiled.split("\n").findIndex((line) => line.includes("DIAGNOSTIC_TARGET"));
      if (diagnosticLineIndex >= 0) {
        const line = expanded.compiled.split("\n")[diagnosticLineIndex] || "";
        diagnostics.push({
          severity: "warning",
          message: "Mock diagnostic target needs review.",
          source_file: filePath,
          line: diagnosticLineIndex + 1,
          column: Math.max(1, line.indexOf("DIAGNOSTIC_TARGET") + 1),
          end_line: diagnosticLineIndex + 1,
          end_column: Math.max(1, line.indexOf("DIAGNOSTIC_TARGET") + "DIAGNOSTIC_TARGET".length + 1),
          suggestion: "Resolve the marked diagnostic target before publishing.",
          related: ["mock-diagnostic"],
        });
      }
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
          citations: citationReferences,
          citation_references: citationReferences,
          duplicate_bibliography_keys: duplicateBibliographyKeys,
          glossary,
          layout_directives: [],
          comments: [],
          change_notes: [],
          ai_sources: [],
          ai_assisted_sections: [],
          labels: [],
          cross_references: [],
        },
        document_ast: {
          metadata: { ...metadata, source_hash: sourceHash },
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
        bibliography,
        index_terms: indexTerms,
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
      if (cmd === "plugin:dialog|message") return confirmResponses.length ? (confirmResponses.shift() ? "Ok" : "Cancel") : "Ok";
      if (cmd === "plugin:dialog|confirm") return confirmResponses.length ? confirmResponses.shift() : true;
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
      if (cmd === "list_transform_engines") return mockTransformEngines();
      if (cmd === "run_external_transform") {
        const request = args.request as { name: string; engine_path?: string; trusted?: boolean; input_mode?: string; timeout_ms?: number };
        if (!request.engine_path) throw new Error(`${request.name} engine path is not configured.`);
        if (!request.trusted) throw new Error(`${request.name} external transform is not trusted.`);
        if (request.engine_path.includes("missing")) throw new Error(`${request.name} executable not found at ${request.engine_path}.`);
        return {
          diagnostics: [
            {
              message: `${request.name} probe ok via ${request.input_mode || "stdin"} with timeout ${request.timeout_ms || 0}`,
            },
          ],
          cache_key: `${request.name}:${request.engine_path}:${request.input_mode || "stdin"}:${request.timeout_ms || 0}`,
        };
      }
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
        if (!entries.length) throw new Error(`Mock workspace not found: ${root}`);
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
        const request = args.request as { text: string; target?: string; options?: { includeManifest?: boolean } };
        const response = compileMarkdown(request.text);
        const diagnostics = request.text.includes("EXPORT_BLOCKER")
          ? [
              {
                severity: "error",
                message: `${(request.target || "html").toUpperCase()} export requires approved metadata before writing.`,
                source_file: "/workspace/market.md",
                line: 3,
                column: 1,
                end_line: 3,
                end_column: 16,
                suggestion: "Set status: approved before exporting this target.",
                related: [`target:${request.target || "html"}`],
              },
            ]
          : [];
        return {
          ready: diagnostics.length === 0,
          error_count: diagnostics.filter((diagnostic) => diagnostic.severity === "error").length,
          warning_count: 0,
          info_count: request.target === "pptx" ? 1 : 0,
          source_map: [],
          paged_document: response.paged_document,
          diagnostics,
          manifest: {
            ...response.export_manifest,
            export_target: request.target || "html",
            export_options: request.options || {},
          },
        };
      }
      if (cmd === "cleanup_ai_paste") {
        const request = args.request as {
          text: string;
          add_provenance?: boolean;
          mark_as_draft?: boolean;
          insert_citation_todos?: boolean;
        };
        const issues = ["Normalized AI paste in browser workflow test."];
        let cleanedMarkdown = `Cleaned AI output\n\n${request.text.trim()}`;
        if (request.insert_citation_todos && /\d|revenue|growth|market|report/i.test(cleanedMarkdown)) {
          cleanedMarkdown = `${cleanedMarkdown} <!-- TODO: citation needed -->`;
          issues.push("Inserted 1 citation TODO marker.");
        }
        if (request.mark_as_draft) {
          cleanedMarkdown = `<!-- ai-assisted: status=needs-review | reviewedBy= | reviewedAt= | source=AI paste cleanup | promptSummary=AI paste cleanup review required -->\n\n${cleanedMarkdown}`;
          issues.push("Marked inserted content as draft.");
        }
        const provenanceBlock = request.add_provenance
          ? "```ai-source\nprovider: unknown\nmodel: unknown\ndate: 2026-05-20\npromptSummary: AI paste cleanup\nreviewedBy: \nstatus: needs-review\n```"
          : null;
        if (provenanceBlock) {
          cleanedMarkdown = `${cleanedMarkdown}\n\n${provenanceBlock}`;
        }
        return {
          cleaned_markdown: cleanedMarkdown,
          issues,
          provenance_block: provenanceBlock,
        };
      }
      if (cmd === "create_snapshot") return { snapshot_path: "/tmp/mock-snapshot.md" };
      if (cmd === "export_document") {
        const request = args.request as { output_path: string; target?: string; options?: { includeManifest?: boolean } };
        if (request.output_path.includes("fail")) throw new Error(`Mock export writer failed for ${request.output_path}`);
        const manifestPath = request.options?.includeManifest === false ? null : `${request.output_path}.manifest.json`;
        return {
          output_path: request.output_path,
          manifest_path: manifestPath,
          diagnostics: [
            {
              severity: "info",
              message: `Mock ${request.target || "html"} export wrote ${request.output_path}`,
              source_file: "/workspace/market.md",
              line: null,
              column: null,
              end_line: null,
              end_column: null,
              suggestion: null,
              related: manifestPath ? [manifestPath] : [],
            },
          ],
        };
      }
      return null;
    }

    window.__NEDITOR_E2E__ = {
      queueDialogSelection(path: string | null) {
        dialogSelections.push(path);
      },
      queueConfirmResponse(value: boolean) {
        confirmResponses.push(value);
      },
      getFile(path: string) {
        return readMockFile(path).text;
      },
      setFile(path: string, text: string) {
        setFile(path, text);
      },
      deleteFile(path: string) {
        files.delete(normalizePath(path));
        persistE2eState();
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

async function queueConfirmResponse(page: Page, value: boolean) {
  await page.evaluate((response) => window.__NEDITOR_E2E__.queueConfirmResponse(response), value);
}

async function mockFileText(page: Page, path: string) {
  return page.evaluate((selectedPath) => window.__NEDITOR_E2E__.getFile(selectedPath), path);
}

async function setMockFileText(page: Page, path: string, text: string) {
  await page.evaluate(({ selectedPath, content }) => window.__NEDITOR_E2E__.setFile(selectedPath, content), { selectedPath: path, content: text });
}

async function deleteMockFile(page: Page, path: string) {
  await page.evaluate((selectedPath) => window.__NEDITOR_E2E__.deleteFile(selectedPath), path);
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

  await page.getByLabel("View mode").selectOption("focus");
  await expect(page.getByRole("region", { name: "Markdown source" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeHidden();

  await page.getByLabel("View mode").selectOption("export");
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("exports");
  await expect(page.getByRole("region", { name: "Markdown source" })).toBeHidden();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeVisible();
  await expect(page.locator(".sidebar").getByRole("heading", { name: "Export" })).toBeVisible();
  await expect(page.locator(".sidebar").getByText("Manifest")).toBeVisible();

  await page.getByLabel("View mode").selectOption("review");
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("review");
  await expect(page.getByRole("region", { name: "Markdown source" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeVisible();
  await expect(page.locator(".sidebar").getByRole("heading", { name: "Review" })).toBeVisible();

  await page.getByLabel("View mode").selectOption("presentation");
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("outline");
  await expect(page.getByRole("region", { name: "Markdown source" })).toBeHidden();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeVisible();
  await expect(page.locator(".sidebar").getByRole("heading", { name: "Outline" })).toBeVisible();
});

test("syncs editor and preview scrolling and jumps preview headings to source", async ({ page }) => {
  const longPreviewDocument = [
    "---",
    "title: Preview Navigation",
    "status: draft",
    "---",
    "",
    "# Preview Navigation",
    "",
    "Opening context for preview navigation.",
    "",
    ...Array.from({ length: 36 }, (_, index) => [
      `## Section ${index + 1}`,
      "",
      `Narrative paragraph ${index + 1} with enough text to create scrollable preview content.`,
      "",
    ]).flat(),
    "## Navigation Target",
    "",
    "The preview heading click should focus this source line.",
    "",
    ...Array.from({ length: 24 }, (_, index) => [
      `## Follow-up ${index + 1}`,
      "",
      `Trailing paragraph ${index + 1} keeps the target away from the document end.`,
      "",
    ]).flat(),
  ].join("\n");

  await setMockFileText(page, "/workspace/preview-navigation.md", longPreviewDocument);
  await queueDialogSelection(page, "/workspace/preview-navigation.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  const editorScroller = page.locator(".cm-scroller");
  const previewPane = page.locator(".preview-pane");

  await expect(page.getByRole("heading", { name: "Preview Navigation" })).toBeVisible();
  await editorScroller.evaluate((element) => {
    element.scrollTop = element.scrollHeight * 0.55;
    element.dispatchEvent(new Event("scroll", { bubbles: true }));
  });
  await expect.poll(() => previewPane.evaluate((element) => element.scrollTop)).toBeGreaterThan(20);

  await previewPane.evaluate((element) => {
    element.scrollTop = 0;
    element.dispatchEvent(new Event("scroll", { bubbles: true }));
  });
  await expect.poll(() => editorScroller.evaluate((element) => element.scrollTop)).toBeLessThan(20);

  const targetHeading = page.getByRole("heading", { name: "Navigation Target" });
  await targetHeading.scrollIntoViewIfNeeded();
  await targetHeading.click();

  await expect(page.locator(".cm-line").filter({ hasText: "## Navigation Target" })).toBeVisible();
  await expect.poll(() => editorScroller.evaluate((element) => element.scrollTop)).toBeGreaterThan(20);
});

test("updates the live preview after source edits", async ({ page }) => {
  const editorContent = page.locator(".cm-content");
  await editorContent.click();
  await page.keyboard.press("Control+End");
  await page.keyboard.type("\n\n## Live Typing Target\nLive preview text from source editing.");

  await expect.poll(() => editorText(page)).toContain("## Live Typing Target");
  await expect(page.getByRole("heading", { name: "Live Typing Target" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toContainText("Live preview text from source editing.");
});

test("persists editor settings and runs search plus heading commands", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/editor-ergonomics.md",
    [
      "---",
      "title: Editor Ergonomics",
      "status: draft",
      "---",
      "",
      "# Editor Ergonomics",
      "",
      "Find target Acme should be replaceable.",
      "",
      "## Command Target",
      "",
      "Heading command should focus this source line.",
      "",
      "- First item",
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/editor-ergonomics.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  const editorContent = page.locator(".cm-content");
  await expect(page.locator(".status-bar")).toContainText("29 words | 189 characters | 1 min read");
  await expect(editorContent).toHaveAttribute("spellcheck", "true");
  await expect(editorContent).toHaveAttribute("autocapitalize", "sentences");
  await page.getByLabel("Sidebar panel").selectOption("settings");

  const appShell = page.locator(".app-shell");
  const previewPane = page.locator(".preview-pane");
  const previewDocument = page.locator(".preview-document");
  await page.getByLabel("Theme", { exact: true }).selectOption("dark");
  await page.getByLabel("Preview theme").selectOption("dark");
  await page.getByLabel("High contrast").check();
  await page.getByLabel("Reduced motion").check();
  await page.getByLabel("Editor font").fill("Courier New, monospace");
  await page.getByLabel("Editor font size").fill("18");
  await page.getByLabel("Editor line height").fill("1.8");
  await page.getByLabel("Preview font").fill("Georgia, serif");
  await page.getByLabel("Preview font size").fill("19");
  await page.getByLabel("Preview line height").fill("1.9");
  await expect(appShell).toHaveAttribute("data-theme", "dark");
  await expect(appShell).toHaveAttribute("data-high-contrast", "true");
  await expect(appShell).toHaveAttribute("data-reduced-motion", "true");
  await expect(previewPane).toHaveAttribute("data-preview-theme", "dark");
  await expect(previewDocument).toHaveAttribute("style", /font-family: Georgia, serif; font-size: 19px; line-height: 1\.9/);
  await expect.poll(() => editorContent.evaluate((element) => getComputedStyle(element).fontSize)).toBe("18px");

  await expect(page.getByLabel("Word wrap")).toBeChecked();
  await expect(page.getByLabel("Line numbers")).toBeChecked();
  await expect.poll(() => editorContent.evaluate((element) => element.classList.contains("cm-lineWrapping"))).toBe(true);
  await expect(page.locator(".cm-lineNumbers")).toHaveCount(1);

  await page.getByLabel("Word wrap").uncheck();
  await page.getByLabel("Line numbers").uncheck();
  await expect.poll(() => editorContent.evaluate((element) => element.classList.contains("cm-lineWrapping"))).toBe(false);
  await expect(page.locator(".cm-lineNumbers")).toHaveCount(0);
  await page.getByRole("button", { name: "Save Workspace" }).click();

  await page.reload();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("settings");
  await expect(appShell).toHaveAttribute("data-theme", "dark");
  await expect(appShell).toHaveAttribute("data-high-contrast", "true");
  await expect(appShell).toHaveAttribute("data-reduced-motion", "true");
  await expect(previewPane).toHaveAttribute("data-preview-theme", "dark");
  await expect.poll(() => editorContent.evaluate((element) => getComputedStyle(element).fontSize)).toBe("18px");
  await expect(page.getByLabel("Word wrap")).not.toBeChecked();
  await expect(page.getByLabel("Line numbers")).not.toBeChecked();
  await expect.poll(() => editorContent.evaluate((element) => element.classList.contains("cm-lineWrapping"))).toBe(false);
  await expect(page.locator(".cm-lineNumbers")).toHaveCount(0);

  await editorContent.click();
  await page.keyboard.press("Control+f");
  await page.getByRole("textbox", { name: "Find" }).fill("Acme");
  await page.getByRole("textbox", { name: "Replace" }).fill("Globex");
  await page.locator(".cm-search").getByRole("button", { name: "replace all" }).click();
  await expect.poll(() => editorText(page)).toContain("Find target Globex should be replaceable.");
  await expect.poll(() => editorText(page)).not.toContain("Acme");

  await editorContent.click();
  await page.keyboard.press("Control+End");
  await page.keyboard.press("Enter");
  await page.keyboard.type("Second item");
  await expect.poll(() => editorText(page)).toContain("- Second item");

  await page.keyboard.press("Control+End");
  await page.keyboard.press("Enter");
  await page.keyboard.type("(");
  await expect.poll(() => editorText(page)).toContain("()");

  await page.keyboard.press("Control+End");
  await page.keyboard.press("Enter");
  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Bold selection");
  await page.getByRole("button", { name: /Bold selection Markdown/ }).click();
  await page.keyboard.type("bold shortcut");
  await expect.poll(() => editorText(page)).toContain("**bold shortcut**");

  await page.keyboard.press("Control+End");
  await page.keyboard.press("Enter");
  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Italic selection");
  await page.getByRole("button", { name: /Italic selection Markdown/ }).click();
  await page.keyboard.type("italic shortcut");
  await expect.poll(() => editorText(page)).toContain("*italic shortcut*");

  await page.keyboard.press("Control+End");
  await page.keyboard.press("Enter");
  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Inline code selection");
  await page.getByRole("button", { name: /Inline code selection Markdown/ }).click();
  await page.keyboard.type("code shortcut");
  await expect.poll(() => editorText(page)).toContain("`code shortcut`");

  await page.keyboard.press("Control+End");
  await page.keyboard.press("Enter");
  await page.keyboard.type('"');
  await expect.poll(() => editorText(page)).toContain('""');

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Insert code fence");
  await page.getByRole("button", { name: /Insert code fence Snippet/ }).click();
  await expect.poll(() => editorText(page)).toContain("```markdown\n\n```");

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Command Target");
  await page.getByRole("button", { name: /Command Target Heading line/ }).click();
  await expect(page.locator(".cm-line").filter({ hasText: "## Command Target" })).toBeVisible();
});

test("navigates source from the outline sidebar", async ({ page }) => {
  const outlineDocument = [
    "---",
    "title: Outline Navigation",
    "status: draft",
    "---",
    "",
    "# Outline Navigation",
    "",
    ...Array.from({ length: 24 }, (_, index) => [`## Context ${index + 1}`, "", `Context paragraph ${index + 1}.`, ""]).flat(),
    "## Outline Target",
    "",
    "The outline sidebar should focus this heading in source.",
    "",
  ].join("\n");

  await setMockFileText(page, "/workspace/outline-navigation.md", outlineDocument);
  await queueDialogSelection(page, "/workspace/outline-navigation.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await page.getByLabel("Sidebar panel").selectOption("outline");

  await page.locator(".sidebar").getByRole("button", { name: "Outline Target" }).click();

  await expect(page.locator(".cm-line").filter({ hasText: "## Outline Target" })).toBeVisible();
  await expect.poll(() => page.locator(".cm-scroller").evaluate((element) => element.scrollTop)).toBeGreaterThan(20);
});

test("navigates compiler diagnostics to the source range", async ({ page }) => {
  const diagnosticDocument = [
    "---",
    "title: Diagnostic Navigation",
    "status: draft",
    "---",
    "",
    "# Diagnostic Navigation",
    "",
    ...Array.from({ length: 20 }, (_, index) => [`## Section ${index + 1}`, "", `Context paragraph ${index + 1}.`, ""]).flat(),
    "## Diagnostic Target",
    "",
    "This line contains DIAGNOSTIC_TARGET for source navigation.",
    "",
  ].join("\n");

  await setMockFileText(page, "/workspace/diagnostic-navigation.md", diagnosticDocument);
  await queueDialogSelection(page, "/workspace/diagnostic-navigation.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await page.getByLabel("Sidebar panel").selectOption("diagnostics");

  const diagnostic = page.locator(".sidebar .diagnostic").filter({ hasText: "Mock diagnostic target needs review." });
  await expect(diagnostic).toContainText("/workspace/diagnostic-navigation.md: line");
  await expect(diagnostic).toContainText("Resolve the marked diagnostic target before publishing.");
  await diagnostic.getByRole("button", { name: "Go to source" }).click();

  await expect(page.locator(".cm-line").filter({ hasText: "DIAGNOSTIC_TARGET" })).toBeVisible();
  await expect.poll(() => page.locator(".cm-scroller").evaluate((element) => element.scrollTop)).toBeGreaterThan(20);
});

test("edits with explicit multi-cursor commands", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/multi-cursor.md",
    [
      "---",
      "title: Multi Cursor",
      "status: draft",
      "---",
      "",
      "# Multi Cursor",
      "",
      "Row A",
      "Row B",
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/multi-cursor.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  const editorContent = page.locator(".cm-content");
  await editorContent.click();
  await page.keyboard.press("Control+End");
  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Add cursor above");
  await page.getByRole("button", { name: /Add cursor above Edit/ }).click();
  await page.keyboard.type("!");

  await expect.poll(() => editorText(page)).toContain("Row A!");
  await expect.poll(() => editorText(page)).toContain("Row B!");
});

test("runs command palette citation glossary and index navigation", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/reference-navigation.md",
    [
      "---",
      "title: Reference Navigation",
      "status: draft",
      "---",
      "",
      "# Reference Navigation",
      "",
      "Citation target cites [@risk2026, p. 4] for the operating model.",
      "Missing source cites [@missing2026] while duplicates cite [@dup2026].",
      "",
      "```bibtex",
      "@article{risk2026,",
      "  title={Risk Operating Model},",
      "  author={Risk Team},",
      "  year={2026}",
      "}",
      "@book{dup2026,",
      "  title={Duplicate Reference A},",
      "  author={One},",
      "  year={2026}",
      "}",
      "@book{dup2026,",
      "  title={Duplicate Reference B},",
      "  author={Two},",
      "  year={2026}",
      "}",
      "```",
      "",
      "```glossary",
      "ARR: Annual recurring revenue.",
      "```",
      "",
      "Working capital{#index:Working Capital} should be easy to find.",
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/reference-navigation.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("risk2026");
  await page.getByRole("button", { name: "[@risk2026] Citation" }).click();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("references");
  await expect(page.locator(".cm-line").filter({ hasText: "Citation target cites" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Resolved references" })).toBeVisible();
  await expect(page.getByText("Risk Operating Model")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Missing keys" })).toBeVisible();
  await expect(page.getByText("@missing2026")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Duplicate keys" })).toBeVisible();
  await expect(page.getByText("dup2026")).toBeVisible();

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("ARR");
  await page.getByRole("button", { name: "ARR Glossary" }).click();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("references");
  await expect(page.locator(".cm-line").filter({ hasText: "ARR: Annual recurring revenue." })).toBeVisible();

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Working Capital");
  await page.getByRole("button", { name: "Working Capital Index" }).click();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("references");
  await expect(page.locator(".cm-line").filter({ hasText: "Working capital" })).toBeVisible();
});

test("runs command palette open document and workspace file navigation", async ({ page }) => {
  await setMockFileText(page, "/workspace/command-first.md", "# Command First\n\nFirst command document body.");
  await setMockFileText(page, "/workspace/command-second.md", "# Command Second\n\nSecond command document body.");
  await setMockFileText(page, "/workspace/reports/workspace-target.md", "# Workspace Target\n\nWorkspace command body.");

  await queueDialogSelection(page, "/workspace/command-first.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await queueDialogSelection(page, "/workspace/command-second.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  await expect.poll(() => editorText(page)).toContain("Second command document body.");
  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Command First");
  await page.getByRole("button", { name: "Command First Open document" }).click();
  await expect(page.locator(".document-tabs .tab.active")).toContainText("Command First");
  await expect.poll(() => editorText(page)).toContain("First command document body.");

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("workspace-target");
  await page.getByRole("button", { name: "reports/workspace-target.md Workspace file" }).click();
  await expect(page.locator(".document-tabs .tab.active")).toContainText("Workspace Target");
  await expect.poll(() => editorText(page)).toContain("Workspace command body.");
  await page.getByLabel("Sidebar panel").selectOption("files");
  await expect.poll(() => activeFileRowText(page)).toContain("workspace-target.md");
});

test("manages external transform engine trust and probe diagnostics", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/diagram.md",
    [
      "# Diagram Probe",
      "",
      "```d2",
      "direction: right",
      "A -> B",
      "```",
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/diagram.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await expect.poll(() => editorText(page)).toContain("A -> B");

  await page.getByLabel("Sidebar panel").selectOption("settings");
  const engine = page.locator(".engine-row").filter({ has: page.getByRole("heading", { name: "d2" }) });
  const enginePath = engine.getByLabel("Engine path");
  const trusted = engine.getByLabel("Trusted");

  await expect(engine).toContainText("Runs only with explicit trust");
  await expect(engine).toContainText("Version probe: d2 --version");

  await enginePath.fill("/usr/local/bin/d2");
  await enginePath.dispatchEvent("change");
  await expect(trusted).not.toBeChecked();
  await expect(engine).toContainText("Probe required after engine path change.");
  await expect(engine).toContainText("Trust was cleared because the executable path changed.");
  await expect(page.getByRole("region", { name: "External transform trust prompts" })).toContainText("/usr/local/bin/d2");

  await queueConfirmResponse(page, true);
  await page.getByRole("region", { name: "External transform trust prompts" }).getByRole("button", { name: "Trust" }).click();
  await expect(page.getByRole("region", { name: "External transform trust prompts" })).toBeHidden();
  await expect(trusted).toBeChecked();

  await engine.getByLabel("Input").selectOption("file");
  await page.getByLabel("Timeout").fill("7750");
  await page.getByLabel("Timeout").dispatchEvent("change");
  await engine.getByRole("button", { name: "Probe" }).click();
  await expect(engine).toContainText("Probe passed");
  await expect(engine).toContainText("d2 probe ok via file with timeout 7750");
  await expect(engine).toContainText("Cache: d2:/usr/local/bin/d2:file:7750");

  await enginePath.fill("/missing/d2");
  await enginePath.dispatchEvent("change");
  await expect(trusted).not.toBeChecked();
  await expect(page.getByRole("region", { name: "External transform trust prompts" })).toContainText("/missing/d2");

  await queueConfirmResponse(page, true);
  await trusted.check();
  await expect(page.getByRole("region", { name: "External transform trust prompts" })).toBeHidden();
  await engine.getByRole("button", { name: "Probe" }).click();
  await expect(engine).toContainText("Probe failed");
  await expect(engine).toContainText("d2 executable not found at /missing/d2.");

  await enginePath.fill("/opt/bin/d2");
  await enginePath.dispatchEvent("change");
  await expect(trusted).not.toBeChecked();
  await queueConfirmResponse(page, false);
  await trusted.click();
  await expect(trusted).not.toBeChecked();
  await expect(page.getByRole("region", { name: "External transform trust prompts" })).toContainText("/opt/bin/d2");
});

test("runs command palette insertion and table editor workflows", async ({ page }) => {
  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Insert table");
  await page.getByRole("button", { name: "Insert table Snippet" }).click();
  await expect.poll(() => editorText(page)).toContain("| Item | Value |");

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Insert table of contents");
  await page.getByRole("button", { name: "Insert table of contents Snippet" }).click();
  await expect.poll(() => editorText(page)).toContain("[TOC]");

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Insert index");
  await page.getByRole("button", { name: "Insert index Snippet" }).click();
  await expect.poll(() => editorText(page)).toContain("[INDEX]");

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Insert bibliography");
  await page.getByRole("button", { name: "Insert bibliography Snippet" }).click();
  await expect.poll(() => editorText(page)).toContain("[BIBLIOGRAPHY]");

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Insert list of figures");
  await page.getByRole("button", { name: "Insert list of figures Snippet" }).click();
  await expect.poll(() => editorText(page)).toContain("[LIST_OF_FIGURES]");

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Insert list of tables");
  await page.getByRole("button", { name: "Insert list of tables Snippet" }).click();
  await expect.poll(() => editorText(page)).toContain("[LIST_OF_TABLES]");

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
  await expect(page).toHaveTitle("market.md - NEditor");
  await expect(page.getByText("/workspace")).toBeVisible();
  await expect(page.getByRole("button", { name: /market\.md/ }).first()).toBeVisible();

  await page.getByLabel("Sidebar panel").selectOption("review");
  await page.locator(".sidebar").getByLabel("Status").selectOption("in-review");
  await expect.poll(() => editorText(page)).toContain("status: in-review");
  await expect(page).toHaveTitle("* market.md - NEditor");
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("status: in-review");
  await expect(page).toHaveTitle("market.md - NEditor");

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

test("switches tabs, guards dirty closes, and prunes stale recent document paths", async ({ page }) => {
  await setMockFileText(page, "/workspace/first.md", "# First File\n\nFirst body.");
  await setMockFileText(page, "/workspace/second.md", "# Second File\n\nSecond body.");
  await setMockFileText(page, "/workspace/rename-source.md", "# Rename Source\n\nRename source body.");
  await setMockFileText(page, "/workspace/delete-after-close.md", "# Delete After Close\n\nDelete body.");

  await queueDialogSelection(page, "/workspace/first.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await queueDialogSelection(page, "/workspace/second.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  await expect(page.locator(".document-tabs .tab.active")).toContainText(/Second File|second\.md/);
  await expect.poll(() => editorText(page)).toContain("Second body.");
  await page.locator(".document-tabs .tab").filter({ hasText: /First File|first\.md/ }).getByRole("button").first().click();
  await expect(page.locator(".document-tabs .tab.active")).toContainText(/First File|first\.md/);
  await expect.poll(() => editorText(page)).toContain("First body.");

  const tabCountBeforeNew = await page.locator(".document-tabs .tab").count();
  await page.getByRole("button", { name: "New" }).click();
  await expect(page.locator(".document-tabs .tab")).toHaveCount(tabCountBeforeNew + 1);
  await queueConfirmResponse(page, false);
  await page.locator(".document-tabs .tab.active").getByLabel("Close document").click();
  await expect(page.locator(".document-tabs .tab")).toHaveCount(tabCountBeforeNew + 1);
  await queueConfirmResponse(page, true);
  await page.locator(".document-tabs .tab.active").getByLabel("Close document").click();
  await expect(page.locator(".document-tabs .tab")).toHaveCount(tabCountBeforeNew);

  await queueDialogSelection(page, "/workspace/rename-source.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await queueDialogSelection(page, "/workspace/rename-target.md");
  await page.getByRole("button", { name: "Rename", exact: true }).click();
  await expect(page.locator(".document-tabs .tab.active")).toContainText("rename-target.md");

  await page.locator(".document-tabs .tab.active").getByLabel("Close document").click();
  await page.getByLabel("Sidebar panel").selectOption("settings");
  await expect(page.getByLabel("Recent files").getByRole("button", { name: "/workspace/rename-source.md" })).toHaveCount(0);
  await expect(page.getByLabel("Recent files").getByRole("button", { name: "/workspace/rename-target.md" })).toBeVisible();
  await expect(page.getByLabel("Recently closed documents").getByRole("button", { name: "/workspace/rename-source.md" })).toHaveCount(0);
  await expect(page.getByLabel("Recently closed documents").getByRole("button", { name: "/workspace/rename-target.md" })).toBeVisible();

  await queueDialogSelection(page, "/workspace/delete-after-close.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await page.locator(".document-tabs .tab.active").getByLabel("Close document").click();
  await deleteMockFile(page, "/workspace/delete-after-close.md");
  await page.getByLabel("Sidebar panel").selectOption("settings");
  await expect(page.getByLabel("Recently closed documents").getByRole("button", { name: "/workspace/delete-after-close.md" })).toBeVisible();
  await page.getByLabel("Recently closed documents").getByRole("button", { name: "/workspace/delete-after-close.md" }).click();
  await expect(page.getByLabel("Recently closed documents").getByRole("button", { name: "/workspace/delete-after-close.md" })).toHaveCount(0);
  await expect(page.getByText("Removed missing recent file delete-after-close.md")).toBeVisible();
});

test("reopens recent folders and prunes moved workspace paths", async ({ page }) => {
  await setMockFileText(page, "/client/project-a.md", "# Project A\n\nClient workspace body.");
  await setMockFileText(page, "/workspace/move-source.md", "# Move Source\n\nMove source body.");

  await queueDialogSelection(page, "/client");
  await page.getByRole("button", { name: "Open Folder", exact: true }).click();
  await page.getByLabel("Sidebar panel").selectOption("files");
  await expect(page.getByText("/client")).toBeVisible();
  await expect(page.getByRole("button", { name: /project-a\.md/ })).toBeVisible();

  await page.getByLabel("Sidebar panel").selectOption("settings");
  const recentFolders = page.getByLabel("Recent folders");
  await expect(recentFolders.getByRole("button", { name: "/client" })).toBeVisible();
  await deleteMockFile(page, "/client/project-a.md");
  await recentFolders.getByRole("button", { name: "/client" }).click();
  await expect(recentFolders.getByRole("button", { name: "/client" })).toHaveCount(0);
  await expect(page.getByText("Removed missing recent folder client")).toBeVisible();

  await queueDialogSelection(page, "/workspace/move-source.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await expect.poll(() => editorText(page)).toContain("Move source body.");
  await page.locator(".document-tabs .tab.active").getByLabel("Close document").click();

  await setMockFileText(page, "/workspace/move-target.md", "# Move Target\n\nMoved body.");
  await deleteMockFile(page, "/workspace/move-source.md");

  await page.getByLabel("Sidebar panel").selectOption("settings");
  const recentlyClosed = page.getByLabel("Recently closed documents");
  await expect(recentlyClosed.getByRole("button", { name: "/workspace/move-source.md" })).toBeVisible();
  await recentlyClosed.getByRole("button", { name: "/workspace/move-source.md" }).click();
  await expect(recentlyClosed.getByRole("button", { name: "/workspace/move-source.md" })).toHaveCount(0);
  await expect(page.getByText("Removed missing recent file move-source.md")).toBeVisible();

  await page.getByLabel("Sidebar panel").selectOption("files");
  await page.getByRole("button", { name: "Refresh" }).click();
  await expect(page.getByRole("button", { name: /move-target\.md/ })).toBeVisible();
  await expect(page.getByRole("button", { name: /move-source\.md/ })).toHaveCount(0);
});

test("restores workspace tabs, active document, pins, mode, and sidebar after reload", async ({ page }) => {
  const longFieldNote = [
    "---",
    "title: Field Notes",
    "status: draft",
    "---",
    "",
    "# Field Notes",
    "",
    "Reloaded workspace note.",
    "",
    ...Array.from({ length: 70 }, (_, index) => [`## Observation ${index + 1}`, "", `Field observation ${index + 1}.`]).flat(),
  ].join("\n");

  await setMockFileText(
    page,
    "/workspace/pinned-brief.md",
    [
      "---",
      "title: Pinned Brief",
      "status: draft",
      "---",
      "",
      "# Pinned Brief",
      "",
      "Pinned workspace note.",
    ].join("\n"),
  );
  await setMockFileText(
    page,
    "/workspace/field-notes.md",
    longFieldNote,
  );

  await queueDialogSelection(page, "/workspace/pinned-brief.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await queueDialogSelection(page, "/workspace/field-notes.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await queueConfirmResponse(page, true);
  await page
    .locator(".document-tabs .tab")
    .filter({ hasText: "Market Entry Report" })
    .getByLabel("Close document")
    .click();

  const pinnedTab = page.locator(".document-tabs .tab").filter({ hasText: "Pinned Brief" });
  await pinnedTab.getByLabel("Pin document").click();
  await expect(page.getByLabel("Pinned tabs").getByRole("button", { name: /Pinned Brief|pinned-brief\.md/ })).toBeVisible();
  await expect(page.locator(".document-tabs .tab.active")).toContainText("Field Notes");

  await page.getByLabel("View mode").selectOption("review");
  await page.getByLabel("Sidebar panel").selectOption("settings");
  await page.locator(".cm-scroller").evaluate((element) => {
    element.scrollTop = element.scrollHeight;
    element.dispatchEvent(new Event("scroll", { bubbles: true }));
  });
  await expect.poll(() => page.locator(".cm-scroller").evaluate((element) => element.scrollTop)).toBeGreaterThan(20);
  await page.getByRole("button", { name: "Save Workspace" }).click();

  await page.reload();

  await expect(page.getByLabel("View mode")).toHaveValue("review");
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("settings");
  await expect(page.locator(".document-tabs .tab")).toHaveCount(2);
  await expect(page.getByLabel("Pinned tabs").getByRole("button", { name: /Pinned Brief|pinned-brief\.md/ })).toBeVisible();
  await expect(page.locator(".document-tabs .tab.active")).toContainText("Field Notes");
  await expect.poll(() => page.locator(".cm-scroller").evaluate((element) => element.scrollTop)).toBeGreaterThan(20);
  await expect.poll(() => editorText(page)).toContain("Observation 70");
  await expect.poll(() => page.locator(".preview-pane").evaluate((element) => element.scrollTop)).toBeGreaterThan(20);
  await expect(page.getByLabel("Recent files").getByRole("button", { name: "/workspace/field-notes.md" })).toBeVisible();

  await page.getByLabel("Sidebar panel").selectOption("files");
  await expect(page.getByText("/workspace")).toBeVisible();
  await expect.poll(() => activeFileRowText(page)).toContain("field-notes.md");
});

test("groups documents by document set and folder", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/board/executive-summary.md",
    [
      "---",
      "title: Executive Summary",
      "status: draft",
      "documentSet: Board Pack",
      "---",
      "",
      "# Executive Summary",
      "",
      "Board pack introduction.",
    ].join("\n"),
  );
  await setMockFileText(
    page,
    "/workspace/board/risk-register.md",
    [
      "---",
      "title: Risk Register",
      "status: draft",
      "document_set: Board Pack",
      "---",
      "",
      "# Risk Register",
      "",
      "Board risk notes.",
    ].join("\n"),
  );
  await setMockFileText(
    page,
    "/workspace/research/interview-notes.md",
    [
      "---",
      "title: Interview Notes",
      "status: draft",
      "---",
      "",
      "# Interview Notes",
      "",
      "Research notes remain grouped by folder.",
    ].join("\n"),
  );
  await setMockFileText(
    page,
    "/workspace/loose-note.md",
    [
      "---",
      "title: Loose Note",
      "status: draft",
      "---",
      "",
      "# Loose Note",
      "",
      "This note will move into the board pack.",
    ].join("\n"),
  );

  for (const path of [
    "/workspace/board/executive-summary.md",
    "/workspace/board/risk-register.md",
    "/workspace/research/interview-notes.md",
    "/workspace/loose-note.md",
  ]) {
    await queueDialogSelection(page, path);
    await page.getByRole("button", { name: "Open", exact: true }).click();
  }

  const boardPack = page.getByLabel("Board Pack tabs");
  await expect(boardPack).toContainText("2");
  await expect(boardPack.getByRole("button", { name: /Executive Summary/ })).toBeVisible();
  await expect(boardPack.getByRole("button", { name: /Risk Register/ })).toBeVisible();

  const researchGroup = page.getByLabel("research tabs");
  await expect(researchGroup).toContainText("1");
  await expect(researchGroup.getByRole("button", { name: /Interview Notes/ })).toBeVisible();

  const looseTab = page.locator(".document-tabs .tab").filter({ hasText: "Loose Note" });
  await looseTab.dragTo(boardPack);
  await expect(boardPack).toContainText("3");
  await expect(boardPack.getByRole("button", { name: /Loose Note/ })).toBeVisible();
  await expect.poll(() => editorText(page)).toContain("documentSet: Board Pack");

  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect.poll(() => mockFileText(page, "/workspace/loose-note.md")).toContain("documentSet: Board Pack");

  await boardPack.getByLabel("Close tab group").click();
  await expect(page.getByLabel("Board Pack tabs")).toHaveCount(0);
  await expect(researchGroup.getByRole("button", { name: /Interview Notes/ })).toBeVisible();
});

test("skips missing restored files with a clear restore warning after reload", async ({ page }) => {
  await setMockFileText(page, "/workspace/kept-brief.md", "# Kept Brief\n\nRestored document body.");
  await setMockFileText(page, "/workspace/missing-brief.md", "# Missing Brief\n\nDeleted before restart.");

  await queueDialogSelection(page, "/workspace/kept-brief.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await queueDialogSelection(page, "/workspace/missing-brief.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await queueConfirmResponse(page, true);
  await page
    .locator(".document-tabs .tab")
    .filter({ hasText: "Market Entry Report" })
    .getByLabel("Close document")
    .click();
  await expect(page.locator(".document-tabs .tab")).toHaveCount(2);

  await page.getByRole("button", { name: "Save Workspace" }).click();
  await deleteMockFile(page, "/workspace/missing-brief.md");
  await page.reload();

  await expect(page.getByRole("region", { name: "Missing restored documents" })).toContainText("/workspace/missing-brief.md");
  await expect(page.locator(".document-tabs .tab")).toHaveCount(1);
  await expect(page.locator(".document-tabs .tab.active")).toContainText(/Kept Brief|kept-brief\.md/);
  await expect.poll(() => editorText(page)).toContain("Restored document body.");
  await page.getByLabel("Sidebar panel").selectOption("settings");
  await expect(page.getByLabel("Recent files").getByRole("button", { name: "/workspace/missing-brief.md" })).toHaveCount(0);
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

test("applies AI paste provenance citation TODO and draft governance toggles", async ({ page }) => {
  await page.getByRole("button", { name: "AI Paste" }).click();
  await page.getByRole("textbox", { name: "Original" }).fill("Revenue grew 24%.");

  await expect(page.getByLabel("Mark as draft")).toBeChecked();
  await expect(page.getByLabel("Add provenance block")).toBeChecked();
  await expect(page.getByLabel("Insert citation TODOs")).toBeChecked();

  await page.getByRole("button", { name: "Preview cleanup" }).click();
  const preview = page.getByRole("textbox", { name: "Cleaned preview" });
  await expect(preview).toHaveValue(/TODO: citation needed/);
  await expect(preview).toHaveValue(/ai-assisted: status=needs-review/);
  await expect(preview).toHaveValue(/```ai-source/);
  await expect(page.getByText("Inserted 1 citation TODO marker.")).toBeVisible();
  await expect(page.getByText("Marked inserted content as draft.")).toBeVisible();

  await page.getByRole("button", { name: "Insert cleaned" }).click();
  await expect.poll(() => editorText(page)).toContain("Revenue grew 24%. <!-- TODO: citation needed -->");
  await expect.poll(() => editorText(page)).toContain("ai-assisted: status=needs-review");
  await expect.poll(() => editorText(page)).toContain("```ai-source");
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

test("runs export readiness, success, and failure workflows", async ({ page }) => {
  await page.getByLabel("Sidebar panel").selectOption("exports");
  await page.getByLabel("Target").selectOption("pptx");
  await page.getByRole("button", { name: "Prepare for export" }).click();

  await expect(page.locator("article.readiness").getByText("Ready", { exact: true })).toBeVisible();
  await expect(page.getByText("0 errors, 0 warnings, 1 info")).toBeVisible();
  await expect(page.locator(".sidebar").getByText('"export_target": "pptx"')).toBeVisible();

  await queueDialogSelection(page, "/exports/market.pptx");
  await page.getByRole("button", { name: "Export document" }).click();
  const exportResult = page.getByRole("region", { name: "Export result" });
  await expect(exportResult).toContainText("Output: /exports/market.pptx");
  await expect(exportResult).toContainText("Manifest: /exports/market.pptx.manifest.json");
  await expect(exportResult).toContainText("Mock pptx export wrote /exports/market.pptx");
  await expect(page.locator(".status-bar")).toContainText("Exported /exports/market.pptx with manifest /exports/market.pptx.manifest.json");

  await queueDialogSelection(page, "/exports/fail.pptx");
  await page.getByRole("button", { name: "Export document" }).click();
  await expect(exportResult).toContainText("Mock export writer failed for /exports/fail.pptx");
  await expect(exportResult).toContainText("Review export readiness diagnostics and target settings before retrying.");
  await expect(page.locator(".status-bar")).toContainText("Export failed: Mock export writer failed for /exports/fail.pptx");

  await page.locator(".cm-content").click();
  await page.keyboard.press(process.platform === "darwin" ? "Meta+A" : "Control+A");
  await page.keyboard.insertText(["# Blocked Export", "", "EXPORT_BLOCKER"].join("\n"));
  await expect.poll(() => editorText(page)).toContain("EXPORT_BLOCKER");
  await page.getByRole("button", { name: "Export document" }).click();
  await expect(page.locator("article.readiness").getByText("Needs attention", { exact: true })).toBeVisible();
  await expect(page.getByRole("region", { name: "Export readiness diagnostics" })).toContainText("PPTX export requires approved metadata before writing.");
  await expect(page.getByRole("region", { name: "Export readiness diagnostics" })).toContainText("target:pptx");
  await expect(page.locator(".status-bar")).toContainText("1 errors block export");
});
