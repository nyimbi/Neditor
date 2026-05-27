import { expect, test, type Locator, type Page, type TestInfo } from "@playwright/test";

async function installTauriMock(page: Page, stateKey: string) {
  await page.addInitScript((e2eStateKey) => {
    const callbacks = new Map<number, unknown>();
    const eventListeners = new Map<string, number[]>();
    const persistentState = (() => {
      try {
        const encoded = window.sessionStorage.getItem(e2eStateKey);
        if (encoded)
          return JSON.parse(encoded) as {
            files?: Array<[string, { text: string; hash: string; modified: string }]>;
            snapshots?: Array<
              [
                string,
                {
                  text: string;
                  hash: string;
                  modified: string;
                  metadata_path: string;
                  created_at: string;
                  label: string;
                  sourcePath: string | null;
                  documentVersion: string | null;
                  status: string | null;
                  author: string | null;
                  includeGraphHash: string | null;
                },
              ]
            >;
            stores?: Record<string, Record<string, unknown>>;
            releaseTags?: string[];
          };
      } catch {
        // Fall back to an in-memory mock if session storage is unavailable.
      }
      return { files: [], snapshots: [], stores: {} as Record<string, Record<string, unknown>>, releaseTags: [] };
    })();
    const storesByPath = new Map<string, Map<string, unknown>>(
      Object.entries(persistentState.stores || {}).map(([path, values]) => [path, new Map(Object.entries(values))]),
    );
    const storeResources = new Map<number, Map<string, unknown>>();
    const files = new Map<string, { text: string; hash: string; modified: string }>(persistentState.files || []);
    const snapshots = new Map<string, {
      text: string;
      hash: string;
      modified: string;
      metadata_path: string;
      created_at: string;
      label: string;
      sourcePath: string | null;
      documentVersion: string | null;
      status: string | null;
      author: string | null;
      includeGraphHash: string | null;
    }>(persistentState.snapshots || []);
    const releaseTags: string[] = persistentState.releaseTags || [];
    const dialogSelections: Array<string | null> = [];
    const confirmResponses: boolean[] = [];
    const revealedPaths: string[] = [];
    let clipboardText = "";
    let clipboardMime = "text/plain";
    let callbackId = 1;
    let storeId = 1;
    let compileDelayMs = 0;

    function delay(ms: number) {
      return new Promise((resolve) => {
        window.setTimeout(resolve, ms);
      });
    }

    function persistE2eState() {
      try {
        const stores = Object.fromEntries(Array.from(storesByPath.entries()).map(([path, values]) => [path, Object.fromEntries(values.entries())]));
        window.sessionStorage.setItem(e2eStateKey, JSON.stringify({ files: Array.from(files.entries()), snapshots: Array.from(snapshots.entries()), stores, releaseTags }));
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

    function snapshotWorkspaceId(filePath?: string | null) {
      return filePath ? hash(normalizePath(filePath)) : "unsaved";
    }

    function snapshotRoot(filePath?: string | null, storage?: string | null) {
      const workspaceId = snapshotWorkspaceId(filePath);
      if (storage === "project-local" && filePath) return `${dirname(normalizePath(filePath))}/.neditor/snapshots/${workspaceId}`;
      return `/app-data/neditor/snapshots/${workspaceId}`;
    }

    function snapshotListItem(path: string, item: {
      hash: string;
      metadata_path: string;
      created_at: string;
      label: string;
      sourcePath: string | null;
      documentVersion: string | null;
      status: string | null;
      author: string | null;
      includeGraphHash: string | null;
    }) {
      return {
        snapshot_path: path,
        metadata_path: item.metadata_path,
        hash: item.hash,
        created_at: item.created_at,
        label: item.label,
        document_version: item.documentVersion,
        status: item.status,
        author: item.author,
        include_graph_hash: item.includeGraphHash,
      };
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

    function frontMatterScalarMap(text: string) {
      const frontMatter = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
      if (!frontMatter) return {};
      return yamlScalarMap(frontMatter[1]);
    }

    function projectVariableScalars(filePath: string) {
      const folder = dirname(filePath);
      const candidate =
        files.get(`${folder}/.neditor/variables.yaml`)?.text ||
        files.get(`${folder}/.neditor/variables.yml`)?.text ||
        "";
      if (!candidate.trim()) return {};
      const inner = candidate.match(/^variables:\s*\r?\n([\s\S]*)$/m);
      return yamlScalarMap(inner?.[1] || candidate);
    }

    function yamlScalarMap(text: string) {
      const values: Record<string, string> = {};
      for (const raw of text.split(/\r?\n/)) {
        const match = raw.match(/^\s{0,2}([A-Za-z][\w-]*):\s*(.+?)\s*$/);
        if (!match) continue;
        const value = match[2].replace(/^["']|["']$/g, "").trim();
        if (!value || value === "[]" || value === "{}" || /^[>|]$/.test(value) || value.startsWith("[") || value.startsWith("{")) continue;
        values[match[1]] = value;
      }
      return values;
    }

    function htmlFromMarkdown(text: string) {
      const lines = text.split(/\r?\n/);
      const html: string[] = [];
      for (let index = 0; index < lines.length; index += 1) {
        const line = lines[index];
        if (line.startsWith("# ")) {
          html.push(`<h1 id="${escapeHtml(line.slice(2).toLowerCase().replace(/\s+/g, "-"))}">${escapeHtml(line.slice(2))}</h1>`);
          continue;
        }
        if (line.startsWith("## ")) {
          html.push(`<h2 id="${escapeHtml(line.slice(3).toLowerCase().replace(/\s+/g, "-"))}">${escapeHtml(line.slice(3))}</h2>`);
          continue;
        }
        const tableCaption = parseTableCaption(line);
        if (tableCaption && lines[index + 1]?.trim().startsWith("|")) {
          const tableLines: string[] = [];
          let cursor = index + 1;
          while (lines[cursor]?.trim().startsWith("|")) {
            tableLines.push(lines[cursor]);
            cursor += 1;
          }
          html.push(renderMockTable(tableCaption, tableLines));
          index = cursor - 1;
          continue;
        }
        if (line.trim() === "$$") {
          const body: string[] = [];
          let cursor = index + 1;
          while (cursor < lines.length && !lines[cursor].trim().startsWith("$$")) {
            body.push(lines[cursor]);
            cursor += 1;
          }
          const attrs = parseFigureAttributes((lines[cursor] || "").replace(/^\s*\$\$\s*/, ""));
          const id = attrs.id ? ` id="${escapeHtml(attrs.id)}"` : "";
          const caption = attrs.caption || (attrs.id ? `Equation ${attrs.id.replace(/^eq:/, "")}` : "Equation");
          html.push(`<figure${id} class="equation"><pre>${escapeHtml(body.join("\n"))}</pre><figcaption>${escapeHtml(caption)}</figcaption></figure>`);
          index = cursor;
          continue;
        }
        const figure = line.match(/^!\[([^\]]*)\]\(([^)]+)\)(?:\{([^}]*)\})?/);
        if (figure) {
          const attrs = parseFigureAttributes(figure[3] || "");
          const id = attrs.id ? ` id="${escapeHtml(attrs.id)}"` : "";
          const caption = attrs.caption || figure[1] || "Figure";
          html.push(`<figure${id} class="figure"><img src="${escapeHtml(figure[2])}" alt="${escapeHtml(figure[1])}"/><figcaption>${escapeHtml(caption)}</figcaption></figure>`);
          continue;
        }
        const tocItem = line.match(/^\s*-\s+\[(.+?)\]\(#(.+?)\)$/);
        if (tocItem) {
          html.push(`<li><a href="#${escapeHtml(tocItem[2])}">${escapeHtml(tocItem[1])}</a></li>`);
          continue;
        }
        if (line.trim().startsWith("|")) {
          html.push(`<pre>${escapeHtml(line)}</pre>`);
          continue;
        }
        html.push(!line.trim() || line.trim() === "---" ? "" : `<p>${escapeHtml(line)}</p>`);
      }
      return html.join("\n");
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

    function injectGeneratedToc(text: string) {
      if (!text.includes("[TOC]")) return text;
      const headings = headingsFromMarkdown(text.replace("[TOC]", ""));
      const toc = headings
        .filter((heading) => heading.level <= 3)
        .map((heading) => `${"  ".repeat(Math.max(0, heading.level - 1))}- [${heading.text}](#${heading.anchor})`)
        .join("\n");
      return text.replace("[TOC]", `## Table of Contents\n\n${toc}`);
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

    function bibliographyFromMarkdown(text: string, sourceFile: string) {
      const entries: Array<{
        key: string;
        title: string;
        author: string | null;
        issued: string | null;
        raw: string;
        source_file: string;
        line: number;
        column: number;
        end_column: number;
      }> = [];
      for (const fence of text.matchAll(/```(?:bibtex|bibliography)\s*\n([\s\S]*?)```/g)) {
        const bodyStart = (fence.index || 0) + fence[0].indexOf(fence[1]);
        for (const entry of fence[1].matchAll(/@\w+\s*\{\s*([^,\s]+)\s*,([\s\S]*?)(?=\n@\w+\s*\{|$)/g)) {
          const body = entry[2] || "";
          const key = entry[1].trim();
          const keyOffset = bodyStart + (entry.index || 0) + entry[0].indexOf(entry[1]);
          const location = sourcePositionAtOffset(text, keyOffset);
          entries.push({
            key,
            title: bibtexField(body, "title") || key,
            author: bibtexField(body, "author"),
            issued: bibtexField(body, "year"),
            raw: entry[0],
            source_file: sourceFile,
            line: location.line,
            column: location.column,
            end_column: location.column + key.length,
          });
        }
      }
      return entries;
    }

    function sourcePositionAtOffset(text: string, offset: number) {
      const before = text.slice(0, offset);
      const lines = before.split("\n");
      return { line: lines.length, column: lines[lines.length - 1].length + 1 };
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

    function aiFields(text: string) {
      const fields: Record<string, string> = {};
      for (const part of text.split("|")) {
        const [rawKey, ...rawValue] = part.split("=");
        const key = rawKey?.trim();
        if (!key) continue;
        fields[key] = rawValue.join("=").trim();
      }
      return fields;
    }

    function aiSourcesFromMarkdown(text: string) {
      const sources: Array<{
        line: number;
        provider: string;
        model: string;
        date: string;
        prompt_summary: string;
        reviewed_by: string;
        reviewed_at: string;
        status: string;
      }> = [];
      for (const fence of text.matchAll(/```ai-source\s*\n([\s\S]*?)```/g)) {
        const body = fence[1] || "";
        const line = sourcePositionAtOffset(text, fence.index || 0).line;
        const field = (name: string) => body.match(new RegExp(`^${name}:\\s*(.*?)\\s*$`, "m"))?.[1]?.trim() || "";
        sources.push({
          line,
          provider: field("provider"),
          model: field("model"),
          date: field("date"),
          prompt_summary: field("promptSummary") || field("prompt_summary") || field("prompt"),
          reviewed_by: field("reviewedBy") || field("reviewed_by") || field("reviewer"),
          reviewed_at: field("reviewedAt") || field("reviewed_at") || field("reviewDate"),
          status: field("status") || "unreviewed",
        });
      }
      return sources;
    }

    function aiAssistedSectionsFromMarkdown(text: string) {
      const lines = text.split(/\r?\n/);
      return lines.flatMap((line, index) => {
        const marker = line.match(/<!--\s*ai-assisted:(.*?)-->/);
        if (!marker) return [];
        const fields = aiFields(marker[1]);
        const heading = lines.slice(index + 1).find((candidate) => /^#{1,6}\s+/.test(candidate))?.replace(/^#{1,6}\s+/, "").trim() || "";
        return [
          {
            line: index + 1,
            heading,
            status: fields.status || "unreviewed",
            reviewed_by: fields.reviewedBy || fields.reviewed_by || "",
            reviewed_at: fields.reviewedAt || fields.reviewed_at || "",
            source: fields.source || "",
            prompt_summary: fields.promptSummary || fields.prompt_summary || "",
          },
        ];
      });
    }

    function reviewFields(text: string) {
      const fields: Record<string, string> = {};
      const notes: string[] = [];
      for (const part of text.split("|").map((part) => part.trim()).filter(Boolean)) {
        const pair = part.match(/^([A-Za-z][\w-]*)\s*[:=]\s*(.*?)\s*$/);
        if (pair) fields[pair[1]] = pair[2];
        else if (part === "resolved" || part === "unresolved") fields.state = part;
        else notes.push(part);
      }
      fields.text = notes.join(" | ");
      return fields;
    }

    function commentsFromMarkdown(text: string) {
      return text.split(/\r?\n/).flatMap((line, index) => {
        const match = line.match(/^\s*<!--\s*comment:\s*([\s\S]*?)-->\s*$/);
        if (!match) return [];
        const fields = reviewFields(match[1]);
        return [
          {
            line: index + 1,
            author: fields.author || "local",
            created_at: fields.at || fields.createdAt || "",
            state: fields.state || (/\bresolved\b/.test(match[1]) ? "resolved" : "unresolved"),
            text: fields.text || "Review comment",
          },
        ];
      });
    }

    function changeNotesFromMarkdown(text: string) {
      return text.split(/\r?\n/).flatMap((line, index) => {
        const match = line.match(/^\s*<!--\s*change:\s*([\s\S]*?)-->\s*$/);
        if (!match) return [];
        const fields = reviewFields(match[1]);
        return [
          {
            line: index + 1,
            author: fields.author || "local",
            created_at: fields.at || fields.createdAt || "",
            text: fields.text || "Change note",
          },
        ];
      });
    }

    function transformArtifactsFromMarkdown(text: string, sourceFile: string) {
      const artifacts: Array<{
        id: string;
        name: string;
        output_kind: string;
        source_hash: string;
        source: string;
        source_file: string;
        source_line: number;
        end_source_line: number;
        options: Record<string, unknown>;
        output_hash: string;
        cache_key: string;
        execution_kind: string;
        engine_version: string | null;
        engine_path: string | null;
        input_mode: string;
        duration_ms: number;
        html: string;
        diagnostics: unknown[];
      }> = [];
      const lines = text.split(/\r?\n/);
      for (let index = 0; index < lines.length; index += 1) {
        const fence = lines[index].match(/^```(chart|d2|mermaid|timeline|roadmap|adr|qr|json|yaml|openapi|json-schema)(?:\s.*)?$/);
        if (!fence) continue;
        const body: string[] = [];
        let cursor = index + 1;
        while (cursor < lines.length && lines[cursor] !== "```") {
          body.push(lines[cursor]);
          cursor += 1;
        }
        const name = fence[1];
        const source = body.join("\n");
        const id = `mock-${name}-${artifacts.length + 1}`;
        artifacts.push({
          id,
          name,
          output_kind: name === "d2" || name === "mermaid" ? "svg" : "html",
          source_hash: hash(source),
          source,
          source_file: sourceFile,
          source_line: index + 1,
          end_source_line: Math.max(index + 1, cursor + 1),
          options: {},
          output_hash: hash(`${name}:${source}`),
          cache_key: `${name}:${hash(source)}`,
          execution_kind: name === "d2" ? "external" : "native",
          engine_version: name === "d2" ? "mock-d2" : null,
          engine_path: name === "d2" ? "/usr/local/bin/d2" : null,
          input_mode: name === "d2" ? "stdin" : "inline",
          duration_ms: 7,
          html: `<figure class="transform transform-${name}" data-artifact-id="${id}"><figcaption>${name} transform</figcaption><pre>${escapeHtml(source)}</pre></figure>`,
          diagnostics: [],
        });
        index = cursor;
      }
      return artifacts;
    }

    function figureBlocksFromMarkdown(text: string, sourceFile: string) {
      return text.split(/\r?\n/).flatMap((line, index) => {
        const match = line.match(/^!\[([^\]]*)\]\(([^)]+)\)(?:\{([^}]*)\})?/);
        if (!match) return [];
        const attrs = parseFigureAttributes(match[3] || "");
        return [
          {
            kind: "figure",
            line: index + 1,
            end_line: index + 1,
            id: attrs.id || null,
            src: match[2].trim(),
            alt: match[1].trim(),
            caption: attrs.caption || null,
            float: attrs.float || null,
            fit: attrs.fit || null,
            position: attrs.position || null,
            source: {
              source_file: sourceFile,
              source_line: index + 1,
              end_source_line: index + 1,
            },
          },
        ];
      });
    }

    function tableBlocksFromMarkdown(text: string, sourceFile: string) {
      const lines = text.split(/\r?\n/);
      const blocks: Array<{
        kind: "table";
        line: number;
        end_line: number;
        id: string | null;
        caption: string | null;
        headers: string[];
        alignments: string[];
        rows: string[][];
        source: { source_file: string; source_line: number; end_source_line: number };
      }> = [];
      for (let index = 0; index < lines.length; index += 1) {
        const caption = parseTableCaption(lines[index]);
        if (!caption || !lines[index + 1]?.trim().startsWith("|")) continue;
        const tableLines: string[] = [];
        let cursor = index + 1;
        while (lines[cursor]?.trim().startsWith("|")) {
          tableLines.push(lines[cursor]);
          cursor += 1;
        }
        const rows = tableLines.map(splitMarkdownTableRow);
        blocks.push({
          kind: "table",
          line: index + 1,
          end_line: cursor,
          id: caption.id,
          caption: caption.text,
          headers: rows[0] || [],
          alignments: rows[1] || [],
          rows: rows.slice(2),
          source: {
            source_file: sourceFile,
            source_line: index + 1,
            end_source_line: cursor,
          },
        });
        index = cursor - 1;
      }
      return blocks;
    }

    function equationBlocksFromMarkdown(text: string, sourceFile: string) {
      const lines = text.split(/\r?\n/);
      const blocks: Array<{
        kind: "equation";
        line: number;
        end_line: number;
        id: string | null;
        caption: string | null;
        text: string;
        source: { source_file: string; source_line: number; end_source_line: number };
      }> = [];
      for (let index = 0; index < lines.length; index += 1) {
        if (lines[index].trim() !== "$$") continue;
        const body: string[] = [];
        let cursor = index + 1;
        while (cursor < lines.length && !lines[cursor].trim().startsWith("$$")) {
          body.push(lines[cursor]);
          cursor += 1;
        }
        const attrs = parseFigureAttributes((lines[cursor] || "").replace(/^\s*\$\$\s*/, ""));
        blocks.push({
          kind: "equation",
          line: index + 1,
          end_line: Math.min(lines.length, cursor + 1),
          id: attrs.id || null,
          caption: attrs.caption || null,
          text: body.join("\n"),
          source: {
            source_file: sourceFile,
            source_line: index + 1,
            end_source_line: Math.min(lines.length, cursor + 1),
          },
        });
        index = cursor;
      }
      return blocks;
    }

    function parseTableCaption(line: string) {
      const match = line.match(/^Table:\s+(.+?)(?:\s+\{([^}]*)\})?\s*$/);
      if (!match) return null;
      const attrs = parseFigureAttributes(match[2] || "");
      return {
        text: match[1].trim(),
        id: attrs.id || null,
      };
    }

    function splitMarkdownTableRow(line: string) {
      return line
        .trim()
        .replace(/^\|/, "")
        .replace(/\|$/, "")
        .split("|")
        .map((cell) => cell.trim());
    }

    function renderMockTable(caption: { text: string; id: string | null }, tableLines: string[]) {
      const rows = tableLines.map(splitMarkdownTableRow);
      const headers = rows[0] || [];
      const bodyRows = rows.slice(2);
      const id = caption.id ? ` id="${escapeHtml(caption.id)}"` : "";
      const headerHtml = headers.map((header) => `<th>${escapeHtml(header)}</th>`).join("");
      const bodyHtml = bodyRows
        .map((row) => `<tr>${row.map((cell) => `<td>${escapeHtml(cell)}</td>`).join("")}</tr>`)
        .join("");
      return `<table${id}><caption>${escapeHtml(caption.text)}</caption><thead><tr>${headerHtml}</tr></thead><tbody>${bodyHtml}</tbody></table>`;
    }

    function parseFigureAttributes(raw: string) {
      const attrs: Record<string, string> = {};
      const id = raw.match(/#([A-Za-z0-9_.:-]+)/)?.[1];
      if (id) attrs.id = id;
      for (const match of raw.matchAll(/\b(caption|float|fit|position)=["']([^"']+)["']/g)) {
        attrs[match[1]] = match[2];
      }
      return attrs;
    }

    function mediaFilesFromFigures(figures: Array<{ src: string | null; source: { source_file: string } | null }>) {
      const seen = new Set<string>();
      return figures.flatMap((figure) => {
        const src = figure.src || "";
        if (!src || src.startsWith("data:") || src.includes("://") || src.startsWith("#")) return [];
        const path = resolveRelativePath(figure.source?.source_file || "/workspace/market.md", src);
        if (seen.has(path)) return [];
        seen.add(path);
        return [{ path, hash: hash(src) }];
      });
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
      const compiled = injectGeneratedToc(expanded.compiled);
      const title = titleFromMarkdown(text);
      const status = statusFromMarkdown(text);
      const documentSet = frontMatterValue(text, "documentSet") || frontMatterValue(text, "document_set") || frontMatterValue(text, "set");
      const version = frontMatterValue(text, "version") || "1.0.0";
      const approvedBy = frontMatterValue(text, "approvedBy");
      const approvedAt = frontMatterValue(text, "approvedAt");
      const headings = headingsFromMarkdown(compiled);
      const citationReferences = citationReferencesFromMarkdown(compiled);
      const bibliography = bibliographyFromMarkdown(compiled, filePath);
      const duplicateKeys = duplicateBibliographyKeys(bibliography);
      const glossary = glossaryFromMarkdown(compiled);
      const indexTerms = indexTermsFromMarkdown(compiled);
      const aiSources = aiSourcesFromMarkdown(compiled);
      const aiAssistedSections = aiAssistedSectionsFromMarkdown(compiled);
      const comments = commentsFromMarkdown(compiled);
      const changeNotes = changeNotesFromMarkdown(compiled);
      const transformArtifacts = transformArtifactsFromMarkdown(compiled, filePath);
      const figureBlocks = figureBlocksFromMarkdown(compiled, filePath);
      const tableBlocks = tableBlocksFromMarkdown(compiled, filePath);
      const equationBlocks = equationBlocksFromMarkdown(compiled, filePath);
      const sourceHash = hash(text);
      const frontMatterScalars = frontMatterScalarMap(text);
      const projectVariables = projectVariableScalars(filePath);
      const metadata = {
        ...projectVariables,
        ...frontMatterScalars,
        title,
        status,
        version,
        ...(approvedBy ? { approvedBy } : {}),
        ...(approvedAt ? { approvedAt } : {}),
        ...(documentSet ? { documentSet, document_set: documentSet } : {}),
      };
      const diagnostics = expanded.diagnostics;
      const diagnosticLineIndex = compiled.split("\n").findIndex((line) => line.includes("DIAGNOSTIC_TARGET"));
      if (diagnosticLineIndex >= 0) {
        const line = compiled.split("\n")[diagnosticLineIndex] || "";
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
        document_version: version,
        status,
        exported_at: new Date(0).toISOString(),
        source_hash: sourceHash,
        output_path: null,
        output_hash: null,
        included_files: expanded.includedFiles,
        media_files: mediaFilesFromFigures(figureBlocks),
        layout_sections: [],
        export_target: "html",
        export_options: {},
        transform_artifacts: transformArtifacts,
        progress_steps: [
          {
            id: "compile",
            label: "Compile document model",
            state: "complete",
            detail: "Browser mock compile completed.",
            work_units: 1,
          },
        ],
        diagnostics,
        source_map: [],
        app_version: "e2e-mock",
      };
      return {
        compiled_markdown: compiled,
        html: `${htmlFromMarkdown(compiled)}${transformArtifacts.map((artifact) => artifact.html).join("\n")}`,
        semantic: {
          title,
          status,
          headings,
          outline: headings,
          tables: tableBlocks.length,
          table_summaries: tableBlocks.map((table) => ({
            line: table.line,
            id: table.id,
            caption: table.caption,
            rows: table.rows.length,
            columns: table.headers,
            formulas: [],
          })),
          figures: figureBlocks.length,
          equations: equationBlocks.length,
          citations: citationReferences,
          citation_references: citationReferences,
          duplicate_bibliography_keys: duplicateKeys,
          glossary,
          layout_directives: [],
          comments,
          change_notes: changeNotes,
          ai_sources: aiSources,
          ai_assisted_sections: aiAssistedSections,
          labels: [],
          cross_references: [],
        },
        document_ast: {
          metadata: { ...metadata, source_hash: sourceHash },
          blocks: [
            ...headings.map((heading) => ({
              kind: "heading",
              level: heading.level,
              text: heading.text,
              anchor: heading.anchor,
              line: heading.line,
              end_line: heading.line,
              source: null,
            })),
            ...tableBlocks,
            ...figureBlocks,
            ...equationBlocks,
          ],
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
        transform_artifacts: transformArtifacts,
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
      if (cmd === "pending_cli_open_paths") return [];
      if (cmd === "list_transform_handler_installers") {
        return [
          {
            id: "mock-transform-handlers",
            label: "Mock transform handlers",
            platform: "test",
            manager: "mock",
            summary: "Mock installer plan for browser workflow tests.",
            installable: false,
            requires_admin: false,
            estimatedDownloadSize: "0 MB",
            installLocation: "/workspace/.neditor/handlers",
            engine_names: ["calc", "sql", "xlsx"],
            commands: [],
            handlers: ["calc", "sql", "xlsx"],
            notes: [],
          },
        ];
      }
      if (cmd === "default_markdown_reader_plan") {
        return {
          supported: true,
          applied: false,
          platform: "test",
          appName: "NEditor",
          fileExtensions: ["md", "markdown", "mdown", "mkd"],
          message: "NEditor is available as a Markdown reader.",
          commands: [],
          manual_steps: [],
        };
      }
      if (cmd === "configure_default_markdown_reader") {
        const enabled = Boolean(args?.request && (args.request as { enabled?: boolean }).enabled);
        return {
          supported: true,
          applied: enabled,
          platform: "test",
          appName: "NEditor",
          fileExtensions: ["md", "markdown", "mdown", "mkd"],
          message: enabled ? "NEditor is configured as the default Markdown reader." : "Default Markdown reader integration is disabled.",
          commands: [],
          manual_steps: [],
        };
      }
      if (cmd === "create_support_bundle") {
        const output = typeof args?.request?.output === "string" ? args.request.output : undefined;
        return {
          schema: "neditor.ned-support-bundle.v1",
          workspace: args?.request?.workspace || ".",
          writtenTo: output,
          privacy: {
            documentContentIncluded: false,
            secretsIncluded: false,
            note: "This bundle includes setup status, command paths, report paths, and release evidence summaries only.",
          },
          doctor: { status: "ready", warnings: [] },
          releaseReadiness: {
            status: "current-host-ready-with-external-gaps",
            releaseReady: false,
            evidenceGaps: [{ id: "mock-gap", status: "pending" }],
            failures: [],
          },
          specCompletion: {
            status: "partial-with-release-risks",
            summary: { totalRows: 115, completeRows: 9, openRows: 106 },
            openRows: [{ specSection: "Mock", requirementArea: "Mock gap", status: "Partial" }],
          },
          engineProbe: {
            status: "complete",
            summary: { installed: 10, missingLocal: 0, incompatible: 0, invalidExternalEvidence: 0 },
            engines: [{ key: "pikchr", name: "Pikchr", status: "installed" }],
          },
          evidenceReports: [
            { id: "platform-evidence", status: "accepted", bucket: "ready" },
            { id: "google-docs-import", status: "pending-google-drive-authorization", bucket: "attention" },
          ],
          evidenceReportSummary: { total: 2, ready: 1, attention: 1, missing: 0, failed: 0 },
          recommendations: ["Mock support recommendation"],
        };
      }
      if (cmd === "write_desktop_ui_smoke_report") return null;
      if (cmd === "desktop_workflow_smoke_enabled") return false;
      if (cmd === "write_desktop_workflow_smoke_report") return null;
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
      if (cmd === "prepare_local_agent_handoff") {
        const request = args.request as { profile_id: string; prompt_markdown: string; workspace_path?: string | null };
        const command = request.profile_id === "claude-code-cli" ? "claude" : request.profile_id === "opencode-cli" ? "opencode" : "codex";
        const label = request.profile_id === "claude-code-cli" ? "Claude Code" : request.profile_id === "opencode-cli" ? "OpenCode" : "Codex";
        const workspacePath = normalizePath(request.workspace_path || "/workspace");
        const handoffPath = `${workspacePath}/.neditor/agent-handoffs/neditor-${request.profile_id}-e2e.md`;
        setFile(handoffPath, request.prompt_markdown);
        return {
          profile_id: request.profile_id,
          label,
          command,
          available: true,
          executable_path: `/usr/local/bin/${command}`,
          workspace_path: workspacePath,
          handoff_path: handoffPath,
          launch_command: [command],
          instructions: [`Start ${label} from the workspace path below.`],
          warnings: [],
        };
      }
      if (cmd === "compile_document_with_options") {
        const request = args.request as { text: string; file_path?: string | null };
        if (compileDelayMs > 0) await delay(compileDelayMs);
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
      if (cmd === "list_snapshots") {
        const request = args.request as { file_path?: string | null; storage?: string | null };
        const filePath = request.file_path ? normalizePath(request.file_path) : null;
        const root = snapshotRoot(filePath, request.storage);
        return Array.from(snapshots.entries())
          .filter(([path, item]) => path.startsWith(`${root}/`) && (!filePath || item.sourcePath === filePath))
          .map(([path, item]) => snapshotListItem(path, item))
          .sort((left, right) => String(right.created_at).localeCompare(String(left.created_at)));
      }
      if (cmd === "get_git_status") {
        const path = args.path ? normalizePath(args.path as string) : "";
        if (path.includes("/no-git/") || path.endsWith("/no-git.md")) {
          return {
            inside_repo: false,
            branch: null,
            dirty: false,
            summary: ["outside Git repository"],
          };
        }
        return {
          inside_repo: true,
          branch: "main",
          dirty: false,
          summary: releaseTags.length ? [`tag ${releaseTags[releaseTags.length - 1]}`] : ["clean working tree"],
        };
      }
      if (cmd === "git_history") {
        return [
          { revision: "abc123def456", subject: "Update market report", author: "NEditor", date: "2026-05-20T00:00:00Z" },
          { revision: "001122334455", subject: "Initial market report", author: "NEditor", date: "2026-05-19T00:00:00Z" },
        ];
      }
      if (cmd === "git_diff") return "diff --git a/market.md b/market.md\n+Mock diff for browser workflow\n";
      if (cmd === "commit_document_changes") return null;
      if (cmd === "tag_release") {
        const request = args.request as { tag: string };
        releaseTags.push(request.tag);
        persistE2eState();
        return request.tag;
      }
      if (cmd === "restore_git_revision") {
        const request = args.request as { path: string; revision: string };
        const text = `---\ntitle: Market Entry Report\nstatus: draft\n---\n\n# Market Entry Report\n\nRestored from ${request.revision}.\n`;
        setFile(request.path, text);
        return readMockFile(request.path);
      }
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
        const diagnostics: Array<{
          severity: "error" | "warning";
          message: string;
          source_file: string;
          line: number;
          column: number;
          end_line: number;
          end_column: number;
          suggestion: string;
          related: string[];
        }> = [];
        const approvedMetadata =
          /^status:\s*(approved|published)\s*$/m.test(request.text) &&
          /^approvedBy:\s*\S.+$/m.test(request.text) &&
          /^approvedAt:\s*\S.+$/m.test(request.text);
        if ((request.target || "html") === "pptx" && !approvedMetadata) {
          diagnostics.push({
            severity: "error",
            message: "PPTX export requires approved metadata before writing.",
            source_file: "/workspace/market.md",
            line: 3,
            column: 1,
            end_line: 3,
            end_column: 16,
            suggestion: "Set status to approved or published and add approvedBy plus approvedAt before exporting a presentation.",
            related: ["target:pptx"],
          });
        }
        const pendingAiLine =
          response.semantic.ai_sources.find((source) => source.status !== "human-reviewed")?.line ||
          response.semantic.ai_assisted_sections.find((section) => section.status !== "human-reviewed")?.line;
        if (request.options?.includeProvenance !== false && pendingAiLine) {
          diagnostics.push({
            severity: "warning",
            message: "Document has AI-assisted sections that are not human-reviewed.",
            source_file: "/workspace/market.md",
            line: pendingAiLine,
            column: 1,
            end_line: pendingAiLine,
            end_column: 2,
            suggestion: "Mark AI source blocks and AI-assisted section markers as human-reviewed after review.",
            related: ["ai-provenance"],
          });
        }
        const progressSteps = [
          ...response.export_manifest.progress_steps,
          {
            id: "render",
            label: `Render ${request.target || "html"} artifact`,
            state: "pending",
            detail: "Browser mock export readiness completed before writing.",
            work_units: 1,
          },
        ];
        return {
          ready: diagnostics.length === 0,
          error_count: diagnostics.filter((diagnostic) => diagnostic.severity === "error").length,
          warning_count: diagnostics.filter((diagnostic) => diagnostic.severity === "warning").length,
          info_count: request.target === "pptx" ? 1 : 0,
          source_map: [],
          paged_document: response.paged_document,
          diagnostics,
          manifest: {
            ...response.export_manifest,
            export_target: request.target || "html",
            export_options: request.options || {},
            progress_steps: progressSteps,
          },
          progress_steps: progressSteps,
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
      if (cmd === "create_snapshot") {
        const request = args.request as { text: string; file_path?: string | null; label?: string | null; storage?: string | null };
        const filePath = request.file_path ? normalizePath(request.file_path) : null;
        const snapshotText = filePath && files.has(filePath) ? files.get(filePath)!.text : request.text;
        const root = snapshotRoot(filePath, request.storage);
        const label = (request.label || "snapshot").replace(/[^a-z0-9_-]/gi, "") || "snapshot";
        const ordinal = String(snapshots.size + 1).padStart(3, "0");
        const snapshotPath = `${root}/20260520T0000${ordinal}Z-${label}.md`;
        const createdAt = `2026-05-20T00:00:${String(snapshots.size + 1).padStart(2, "0")}Z`;
        const compiled = compileMarkdown(snapshotText, filePath || "/workspace/market.md");
        snapshots.set(snapshotPath, {
          text: snapshotText,
          hash: hash(snapshotText),
          modified: createdAt,
          metadata_path: snapshotPath.replace(/\.md$/, ".json"),
          created_at: createdAt,
          label,
          sourcePath: filePath,
          documentVersion: String(compiled.metadata.version || "unversioned"),
          status: String(compiled.semantic.status || "draft"),
          author: frontMatterValue(snapshotText, "author") || null,
          includeGraphHash: hash(JSON.stringify(compiled.include_graph)),
        });
        persistE2eState();
        return { snapshot_path: snapshotPath, metadata_path: snapshotPath.replace(/\.md$/, ".json"), hash: hash(snapshotText) };
      }
      if (cmd === "restore_snapshot") {
        const request = args.request as { snapshot_path: string; file_path?: string | null; storage?: string | null };
        const snapshotPath = normalizePath(request.snapshot_path);
        const item = snapshots.get(snapshotPath);
        const filePath = request.file_path ? normalizePath(request.file_path) : null;
        if (!snapshotPath.endsWith(".md")) throw new Error("Snapshot restore requires a Markdown snapshot file.");
        if (!snapshotPath.startsWith(`${snapshotRoot(filePath, request.storage)}/`)) throw new Error("Snapshot restore path must stay inside the configured snapshot store.");
        if (!item) throw new Error("Snapshot restore requires matching snapshot metadata.");
        if (item.sourcePath !== filePath) throw new Error("Snapshot metadata does not match the active document.");
        return { path: snapshotPath, text: item.text, hash: item.hash, modified: item.modified };
      }
      if (cmd === "export_document") {
        const request = args.request as { output_path: string; target?: string; options?: { includeManifest?: boolean } };
        if (request.output_path.includes("fail")) throw new Error(`Mock export writer failed for ${request.output_path}`);
        const manifestPath = request.options?.includeManifest === false ? null : `${request.output_path}.manifest.json`;
        const progressSteps = [
          {
            id: "compile",
            label: "Compile document model",
            state: "complete",
            detail: "Browser mock compile completed.",
            work_units: 1,
          },
          {
            id: "render",
            label: `Render ${request.target || "html"} artifact`,
            state: "complete",
            detail: `Target artifact path: ${request.output_path}`,
            work_units: 1,
          },
        ];
        return {
          output_path: request.output_path,
          manifest_path: manifestPath,
          progress_steps: progressSteps,
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
      setCompileDelay(value: number) {
        compileDelayMs = Math.max(0, Number(value) || 0);
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
      setClipboardText(text: string, mime = "text/plain") {
        clipboardText = text;
        clipboardMime = mime;
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
    class MockSpeechRecognition {
      continuous = false;
      interimResults = false;
      lang = "";
      onresult: ((event: Event) => void) | null = null;
      onerror: ((event: Event) => void) | null = null;
      onend: (() => void) | null = null;

      start() {
        window.setTimeout(() => {
          const result = {
            isFinal: true,
            length: 1,
            0: {
              transcript:
                "Create a client proposal for Acme. The audience is the executive team. Focus on a fast first draft.",
            },
          };
          this.onresult?.({
            resultIndex: 0,
            results: { length: 1, 0: result },
          } as unknown as Event);
          this.onend?.();
        }, 10);
      }

      stop() {
        this.onend?.();
      }

      abort() {
        this.onend?.();
      }
    }
    const speechWindow = window as typeof window & {
      SpeechRecognition?: typeof MockSpeechRecognition;
      webkitSpeechRecognition?: typeof MockSpeechRecognition;
    };
    speechWindow.SpeechRecognition = MockSpeechRecognition;
    speechWindow.webkitSpeechRecognition = MockSpeechRecognition;
    Object.defineProperty(window, "isSecureContext", { value: true, configurable: true });
    Object.defineProperty(navigator, "permissions", {
      value: {
        query: async () => ({
          state: "granted",
          onchange: null,
          addEventListener() {},
          removeEventListener() {},
          dispatchEvent() {
            return true;
          },
        }),
      },
      configurable: true,
    });
    Object.defineProperty(navigator, "clipboard", {
      value: {
        read: async () =>
          clipboardText
            ? [
                {
                  types: clipboardMime === "text/html" ? ["text/html", "text/plain"] : ["text/plain"],
                  getType: async (type: string) => new Blob([type === "text/html" ? clipboardText : clipboardText.replace(/<[^>]+>/g, "")], { type }),
                },
              ]
            : [],
        readText: async () => clipboardText.replace(/<[^>]+>/g, ""),
        writeText: async (text: string) => {
          clipboardText = text;
          clipboardMime = "text/plain";
        },
      },
      configurable: true,
    });
    window.isTauri = true;
  });
}

async function editorText(page: Page) {
  return page.locator(".cm-content").innerText();
}

async function selectSidebarPanelOption(page: Page, panel: string) {
  const selector = page.getByLabel("Sidebar panel");
  if ((await selector.inputValue().catch(() => "")) === panel) {
    await selector.selectOption(panel === "outline" ? "files" : "outline");
  }
  await selector.selectOption(panel);
  await page.evaluate((panelId) => window.__NEDITOR_APP_E2E__?.setSidebar(panelId), panel);
  await expect(selector).toHaveValue(panel);
}

async function openSettingsSection(page: Page, section: string) {
  await selectSidebarPanelOption(page, "settings");
  await page.evaluate((sectionId) => window.__NEDITOR_APP_E2E__?.selectConfigurationSection(sectionId), section);
}

async function moveEditorCursorToEnd(page: Page) {
  if (process.platform === "darwin") {
    await page.keyboard.press("Meta+End");
    await page.keyboard.press("Meta+ArrowDown");
    return;
  }
  await page.keyboard.press("Control+End");
}

async function queueDialogSelection(page: Page, path: string | null) {
  await page.evaluate((selectedPath) => window.__NEDITOR_E2E__.queueDialogSelection(selectedPath), path);
}

async function queueConfirmResponse(page: Page, value: boolean) {
  await page.evaluate((response) => window.__NEDITOR_E2E__.queueConfirmResponse(response), value);
}

async function confirmTransformTrustPrompt(page: Page, engineName: string, accepted: boolean) {
  const prompt = page.getByRole("region", { name: "External transform trust prompts" }).filter({ hasText: `${engineName} transform` });
  await expect(prompt).toBeVisible();
  await queueConfirmResponse(page, accepted);
  await prompt.getByRole("button", { name: "Trust" }).click();
}

async function commitInputValue(inputLocator: Locator, value: string) {
  await inputLocator.evaluate(
    (node, nextValue) => {
      const input = node as HTMLInputElement;
      input.value = nextValue;
      input.dispatchEvent(new Event("input", { bubbles: true }));
      input.dispatchEvent(new Event("change", { bubbles: true }));
    },
    value,
  );
}

async function setEnginePath(enginePath: Locator, path: string) {
  await commitInputValue(enginePath, path);
  await expect(enginePath).toHaveValue(path);
}

async function setCompileDelay(page: Page, value: number) {
  await page.evaluate((delayMs) => window.__NEDITOR_E2E__.setCompileDelay(delayMs), value);
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

async function setMockClipboardText(page: Page, text: string, mime = "text/plain") {
  await page.evaluate(({ value, type }) => window.__NEDITOR_E2E__.setClipboardText(value, type), { value: text, type: mime });
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

test.beforeEach(async ({ page }, testInfo: TestInfo) => {
  const stateKey = `__neditor_e2e_state__:${testInfo.titlePath.join(" / ")}`;
  await installTauriMock(page, stateKey);
  await page.goto("/");
  await expect(page.getByRole("textbox", { name: "Markdown editor" })).toBeVisible();
  await expect.poll(() => page.evaluate(() => Boolean(window.__NEDITOR_APP_E2E__))).toBe(true);
});

test("boots the workbench and switches core view modes", async ({ page }) => {
  await expect(page.getByRole("heading", { name: "Market Entry Report" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeVisible();
  await expect(page.getByRole("textbox", { name: "Markdown editor" })).toBeVisible();
  await expect(page.getByRole("textbox", { name: "Markdown editor" })).toHaveAttribute("aria-multiline", "true");
  await expect(page.getByRole("document", { name: /Rendered preview for Market Entry Report, draft/ })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Application menus" })).toBeVisible();

  await page.getByRole("button", { name: "File menu" }).click();
  await expect(page.getByRole("menu", { name: "File menu" })).toContainText("New Document");
  await expect(page.getByRole("menu", { name: "File menu" })).toContainText("Save As");
  await page.keyboard.press("Escape");

  await page.getByRole("button", { name: "Writing Tools menu" }).click();
  await expect(page.getByRole("menu", { name: "Writing Tools menu" })).toContainText("Edit Table at Cursor");
  await expect(page.getByRole("menu", { name: "Writing Tools menu" })).toContainText("Go to Source Table");
  await expect(page.getByRole("menu", { name: "Writing Tools menu" })).toContainText("Lesson plan");
  await expect(page.getByRole("menu", { name: "Writing Tools menu" })).toContainText("Technical textbook");
  await page.keyboard.press("Escape");

  await page.getByRole("button", { name: "Quality menu" }).click();
  await page.getByRole("menuitem", { name: /Run QA Review/ }).click();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("review");
  await expect(page.getByRole("region", { name: "Quality improvement recommendations" })).toContainText("Quality");

  await page.getByLabel("View mode").selectOption("preview");
  await expect(page.getByRole("region", { name: "Markdown source" })).toBeHidden();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeVisible();
  await expect(page.getByRole("document", { name: /Rendered preview for Market Entry Report, draft/ })).toBeVisible();

  await page.getByLabel("View mode").selectOption("source");
  await expect(page.getByRole("region", { name: "Markdown source" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeHidden();

  await page.getByLabel("View mode").selectOption("split");
  await expect(page.getByRole("region", { name: "Markdown source" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeVisible();

  await page.getByLabel("View mode").selectOption("focus");
  await expect(page.getByRole("region", { name: "Markdown source" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeHidden();

  await page.getByLabel("View mode").selectOption("outline");
  await expect(page.locator("#outline-mode")).toBeVisible();
  await expect(page.getByRole("region", { name: "Markdown source" })).toBeHidden();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeHidden();
  await expect(page.getByLabel("Outline title Market Entry Report")).toBeVisible();

  await page.getByLabel("View mode").selectOption("export");
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("exports");
  await expect(page.getByRole("region", { name: "Markdown source" })).toBeHidden();
  await expect(page.getByRole("region", { name: "Live preview" })).toBeVisible();
  await expect(page.locator(".sidebar").getByRole("heading", { name: "Export", exact: true })).toBeVisible();
  await expect(page.locator(".sidebar").getByRole("heading", { name: "Manifest" })).toBeVisible();

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

test("offers searchable contextual help with workflow actions", async ({ page }) => {
  await selectSidebarPanelOption(page, "help");
  await expect(page.locator(".sidebar").getByRole("heading", { name: "Help Center" })).toBeVisible();
  await expect(page.locator(".help-quick-actions").getByRole("button", { name: "Docs Live" })).toBeVisible();
  await expect(page.getByLabel("Selected help topic").getByText("Create, open, save, and orient yourself in the writing workspace.")).toBeVisible();

  await page.getByLabel("Search help").fill("substack");
  await expect(page.getByRole("button", { name: /Export and publishing/ })).toBeVisible();
  await page.getByRole("button", { name: /Export and publishing/ }).click();
  await expect(page.getByLabel("Selected help topic").getByText(/Markdown bundles, blog packages, Substack/)).toBeVisible();

  await page.getByLabel("Search help").fill("roadmap");
  await expect(page.getByRole("button", { name: /AI-first platform roadmap/ })).toBeVisible();
  await page.getByRole("button", { name: /AI-first platform roadmap/ }).click();
  await expect(page.getByLabel("Selected help topic")).toContainText("50 product changes");
  await expect(page.getByLabel("Selected help topic")).toContainText("release evidence bundles");

  await page.getByLabel("Search help").fill("cache stale windows");
  await expect(page.getByRole("button", { name: /External transform troubleshooting/ })).toBeVisible();
  await page.getByRole("button", { name: /External transform troubleshooting/ }).click();
  await expect(page.getByLabel("Selected help topic")).toContainText("If permission is denied");
  await expect(page.getByLabel("Selected help topic")).toContainText("output is empty or stale");
  await expect(page.getByLabel("Selected help topic")).toContainText("full `.exe` path");
  await expect(page.getByLabel("Selected help topic")).toContainText("PlantUML usually works best in file mode");

  await page.getByRole("button", { name: "Engine settings" }).click();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("settings");
  await expect(page.locator(".sidebar").getByRole("heading", { name: "Settings" })).toBeVisible();

  await page.locator(".command-bar").getByRole("button", { name: "Help" }).click();
  await page.getByLabel("Search help").fill("substack");
  await page.getByRole("button", { name: /Export and publishing/ }).click();
  await page.getByRole("button", { name: "Export panel" }).click();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("exports");
  await expect(page.locator(".sidebar").getByRole("heading", { name: "Export", exact: true })).toBeVisible();

  await page.locator(".command-bar").getByRole("button", { name: "Help" }).click();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("help");
  await expect(page.locator(".sidebar").getByRole("heading", { name: "Help Center" })).toBeVisible();

  await page.locator(".command-bar").getByRole("button", { name: "AI Create" }).hover();
  await expect(page.getByRole("tooltip")).toContainText("agentic Docs Live composer");

  await page.locator(".command-bar").getByRole("button", { name: "Agent" }).click();
  const agent = page.getByRole("dialog", { name: "AI agent workspace" });
  await expect(agent).toBeVisible();
  await agent.getByLabel("What should NEditor do?").fill("Create a board memo, revise it for the CFO, review citations, and distribute as PDF and Google Docs. audience: executive team owner: Strategy deadline: June 1");
  await agent.getByRole("button", { name: "Plan agent workflow" }).click();
  await expect(agent.getByLabel("Agent workflow plan")).toContainText("create -> revise -> review -> distribute");
  await expect(agent.getByLabel("Agent workflow steps")).toContainText("Prepare distribution");
  await agent.getByRole("button", { name: "Generate agent packet" }).click();
  await expect(agent.getByLabel("Agent generated output")).toContainText("Agent run prepared");
  await expect(agent.getByLabel("Agent generated output")).toContainText("QA gates");
  const scheduler = agent.getByLabel("Agent automation scheduler");
  await expect(scheduler).toContainText("Automation Scheduler");
  await scheduler.getByRole("button", { name: "Run safe queue" }).click();
  await expect(scheduler).toContainText("complete");
  await expect(scheduler).toContainText("Evidence scan refreshed");
  await expect(agent.getByRole("region", { name: "Agent run history", exact: true })).toContainText(
    /Automation: \d+ complete, 0 running, 0 queued, \d+ blocked/,
  );
  await expect(agent.getByLabel("Agent distribution target runbooks")).toContainText("PDF controlled copy");
  await expect(agent.getByLabel("Agent distribution target runbooks")).toContainText("Google Docs collaboration package");
  await expect(agent.getByLabel("Agent generated Markdown")).toHaveValue(/NEditor Agent Workspace/);
  await expect(agent.getByLabel("Agent generated Markdown")).toHaveValue(/Target Runbooks/);
  await agent.getByLabel("Provider profile").selectOption("openai-compatible");
  await agent.getByRole("button", { name: "Build provider request" }).click();
  await expect(agent.getByLabel("AI provider request package")).toContainText("approves this provider");
  await expect(agent.getByLabel("AI provider request Markdown")).toHaveValue(/OPENAI_API_KEY/);
  await agent.getByRole("button", { name: "Send to Docs Live" }).click();
  await expect(page.getByRole("dialog", { name: "Docs Live voice drafting" })).toBeVisible();
  await page.getByRole("button", { name: "Close Docs Live" }).click();
  await expect(page.getByRole("dialog", { name: "Docs Live voice drafting" })).toBeHidden();

  await page.locator(".help-quick-actions").getByRole("button", { name: "Guided demo" }).click();
  const demo = page.getByRole("dialog", { name: "NEditor guided demo" });
  await expect(demo).toBeVisible();
  await expect(demo).toContainText("Create with AI");
  await demo.getByRole("button", { name: "Next" }).click();
  await expect(demo).toContainText("Plan the structure");
  await demo.getByRole("button", { name: "Previous" }).click();
  await demo.getByRole("button", { name: "Try this step" }).click();
  await expect(page.getByRole("dialog", { name: "Docs Live voice drafting" })).toBeVisible();
  await expect(page.getByLabel("Document type")).toBeVisible();
});

test("routes natural language command palette instructions to AI workflow surfaces", async ({ page }) => {
  await page.getByRole("button", { name: "Commands" }).click();
  await page
    .getByLabel("Search commands, headings, citations, glossary, index terms, or enter an AI instruction")
    .fill("Draft a customer renewal memo with Docs Live, review claims, export to Substack, and prepare provider handoff");
  const aiRoute = page.getByRole("region", { name: "AI command route", exact: true });
  await expect(aiRoute).toContainText("Generate with AI agent");
  const suggestions = page.getByRole("region", { name: "AI command route suggestions", exact: true });
  await expect(suggestions).toContainText("Docs Live");
  await expect(suggestions).toContainText("Export readiness");
  await expect(suggestions).toContainText("Provider handoff");
  await expect(suggestions).toContainText("Review governance");
  await suggestions.getByRole("button", { name: "Export readiness" }).click();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("exports");
  await expect(page.locator(".sidebar").getByRole("heading", { name: "Export", exact: true })).toBeVisible();

  await page.getByRole("button", { name: "Commands" }).click();
  await page
    .getByLabel("Search commands, headings, citations, glossary, index terms, or enter an AI instruction")
    .fill("Prepare provider handoff for Google Docs with OpenAI-compatible model");
  await page.getByRole("region", { name: "AI command route suggestions", exact: true }).getByRole("button", { name: "Provider handoff" }).click();
  const agent = page.getByRole("dialog", { name: "AI agent workspace" });
  await expect(agent).toBeVisible();
  await expect(agent.getByLabel("AI provider handoff")).toBeVisible();
  await expect(agent.getByLabel("AI provider request package")).toContainText("OpenAI-compatible");
  await expect(agent.getByLabel("AI provider request Markdown")).toHaveValue(/Google Docs/);

  await agent.getByLabel("Provider profile").selectOption("codex-cli");
  await agent.getByRole("button", { name: "Build provider request" }).click();
  await expect(agent.getByLabel("Local agent handoff")).toContainText("Codex workspace handoff");
  await agent.getByRole("button", { name: "Prepare local agent workspace" }).click();
  await expect(agent.getByLabel("Local agent handoff")).toContainText("/workspace/.neditor/agent-handoffs/neditor-codex-cli-e2e.md");
});

test("turns agent claim inventory findings into citation TODOs", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/claim-inventory.md",
    [
      "---",
      "title: Claim Inventory",
      "status: draft",
      "---",
      "",
      "# Claim Inventory",
      "",
      "Revenue increased 18% in 2026 because renewal expansion improved.",
      "The company will launch the premium plan in Q3 FY2026.",
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/claim-inventory.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await page.getByRole("button", { name: "Agent" }).click();
  const agent = page.getByRole("dialog", { name: "AI agent workspace" });
  await agent.getByLabel("What should NEditor do?").fill("Review claims and evidence, prepare PDF distribution, and create citation TODOs for unsupported facts.");
  await agent.getByRole("button", { name: "Generate agent packet" }).click();
  const claimInventory = agent.getByRole("region", { name: "Agent claim inventory" });
  await expect(claimInventory).toContainText("Revenue increased 18%");
  await expect(claimInventory).toContainText("launch the premium plan");
  await claimInventory.locator(".snapshot-row").filter({ hasText: "Revenue increased 18%" }).getByRole("button", { name: "Add citation TODO" }).click();
  await expect.poll(() => editorText(page)).toContain("citation-todo");
  await expect.poll(() => editorText(page)).toContain("Revenue increased 18%");
});

test("exposes keyboard skip links to primary workbench regions", async ({ page }) => {
  for (const [linkName, targetSelector] of [
    ["Skip to commands", "#main-commands"],
    ["Skip to workspace", "#document-workspace"],
    ["Skip to sidebar", "#document-sidebar"],
    ["Skip to source", "#markdown-source"],
    ["Skip to preview", "#live-preview"],
    ["Skip to status", "#document-status"],
  ] as const) {
    const skipLink = page.getByRole("link", { name: linkName });
    await skipLink.focus();
    await expect(skipLink).toBeFocused();
    await skipLink.press("Enter");
    await expect(page.locator(targetSelector)).toBeFocused();
  }
});

test("keeps primary workbench regions accessible across desktop and narrow viewports", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 820 });
  await selectSidebarPanelOption(page, "outline");
  const workspace = page.locator(".workspace");
  const sidebar = page.locator(".sidebar");
  const editor = page.getByRole("region", { name: "Markdown source" });
  const preview = page.getByRole("region", { name: "Live preview" });

  await expect(sidebar.getByRole("heading", { name: "Outline" })).toBeVisible();
  await expect(editor).toBeVisible();
  await expect(preview).toBeVisible();
  await expect.poll(() => workspace.evaluate((element) => getComputedStyle(element).gridTemplateColumns.split(" ").length)).toBeGreaterThan(1);

  await page.setViewportSize({ width: 390, height: 820 });
  await expect(sidebar.getByRole("heading", { name: "Outline" })).toBeVisible();
  await expect(editor).toBeVisible();
  await expect(preview).toBeVisible();
  await expect.poll(() => workspace.evaluate((element) => getComputedStyle(element).gridTemplateColumns.split(" ").length)).toBe(1);
  await expect.poll(() => sidebar.evaluate((element) => getComputedStyle(element).display)).toBe("block");
  await expect
    .poll(() =>
      page.evaluate(() => ({
        scrollWidth: document.documentElement.scrollWidth,
        viewportWidth: window.innerWidth,
      })),
    )
    .toEqual({ scrollWidth: 390, viewportWidth: 390 });

  for (const selector of ["#main-commands", "#document-sidebar", "#markdown-source", "#live-preview", "#document-status"]) {
    const box = await page.locator(selector).boundingBox();
    expect(box, `${selector} should have a rendered box`).not.toBeNull();
    expect(box!.x, `${selector} should not overflow left`).toBeGreaterThanOrEqual(0);
    expect(box!.x + box!.width, `${selector} should not overflow right`).toBeLessThanOrEqual(391);
  }
});

test("collapses and restores command toolbars to recover writing space", async ({ page }) => {
  const commandBar = page.locator("#main-commands");
  const workspace = page.locator("#document-workspace");
  const initialBox = await commandBar.boundingBox();
  const initialWorkspaceBox = await workspace.boundingBox();
  expect(initialBox).not.toBeNull();
  expect(initialWorkspaceBox).not.toBeNull();

  await commandBar.getByRole("button", { name: "Collapse File toolbar" }).click();
  await expect(page.getByRole("button", { name: "Expand File toolbar" })).toBeVisible();
  await expect(commandBar.getByRole("button", { name: "Expand File toolbar" })).toBeHidden();
  await expect(commandBar.getByRole("button", { name: "New" })).toBeHidden();

  await page.getByRole("button", { name: "Expand File toolbar" }).click();
  await expect(commandBar.getByRole("button", { name: "New" })).toBeVisible();

  await commandBar.getByRole("button", { name: "Collapse all" }).click();
  await expect(page.getByRole("button", { name: "Expand File toolbar" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Expand View toolbar" })).toBeVisible();
  await expect(page.getByLabel("View mode")).toBeHidden();
  const collapsedBox = await commandBar.boundingBox();
  const collapsedWorkspaceBox = await workspace.boundingBox();
  expect(collapsedBox).not.toBeNull();
  expect(collapsedWorkspaceBox).not.toBeNull();
  expect(collapsedBox!.height).toBeLessThan(initialBox!.height);
  expect(collapsedBox!.height).toBeLessThanOrEqual(2);
  expect(collapsedWorkspaceBox!.height).toBeGreaterThan(initialWorkspaceBox!.height + 80);
  await expect(page.getByLabel("Collapsed toolbars")).toBeVisible();

  await page.getByRole("button", { name: "Expand View toolbar" }).click();
  await commandBar.getByRole("button", { name: "Expand all" }).click();
  await expect(commandBar.getByRole("button", { name: "Collapse File toolbar" })).toBeVisible();
  await expect(commandBar.getByRole("button", { name: "New" })).toBeVisible();
  await expect(page.getByLabel("View mode")).toBeVisible();
});

test("manages modal focus and Escape return paths", async ({ page }) => {
  const aiPasteButton = page.getByRole("button", { name: "AI Paste" });
  await aiPasteButton.click();
  const aiDialog = page.getByRole("dialog", { name: "AI paste cleanup" });
  await expect(aiDialog).toBeVisible();
  await expect(page.getByRole("textbox", { name: "Original" })).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(aiDialog).toBeHidden();
  await expect(aiPasteButton).toBeFocused();

  const commandsButton = page.getByRole("button", { name: "Commands" });
  await commandsButton.click();
  const commandDialog = page.getByRole("dialog", { name: "Command palette" });
  await expect(commandDialog).toBeVisible();
  await expect(page.getByLabel("Search commands, headings, citations, glossary, index terms, or enter an AI instruction")).toBeFocused();
  await page.keyboard.press("Escape");
  await expect(commandDialog).toBeHidden();
  await expect(commandsButton).toBeFocused();
});

test("shows delegated button help on hover and focus", async ({ page }) => {
  const tooltip = page.getByRole("tooltip");
  const newButton = page.getByRole("button", { name: "New", exact: true });
  await newButton.hover();
  await expect(tooltip).toContainText("New document");

  await page.mouse.move(0, 0);
  await expect(tooltip).toBeHidden();

  const commandsButton = page.getByRole("button", { name: "Commands" });
  await commandsButton.focus();
  await expect(tooltip).toContainText("Open command palette");

  await page.keyboard.press("Tab");
  await expect(tooltip).toContainText("Open Help Center");

  await page.locator("#document-workspace").focus();
  await expect(tooltip).toBeHidden();

  await selectSidebarPanelOption(page, "tables");
  const disabledExport = page.getByRole("button", { name: "Export CSV" });
  await expect(disabledExport).toBeDisabled();
  await disabledExport.hover({ force: true });
  await expect(tooltip).toContainText("Export CSV");
  await expect(tooltip).toContainText("This action is unavailable until the required document state is ready.");
});

test("supports keyboard-only operation for deep workbench controls", async ({ page }) => {
  const newButton = page.getByRole("button", { name: "New", exact: true });
  await newButton.focus();
  await expect(newButton).toBeFocused();
  await newButton.press("Space");
  await expect(page.locator(".document-tabs .tab")).toHaveCount(2);
  await expect(page.locator(".document-tabs .tab.active").getByRole("button", { name: "Unsaved Market Entry Report" })).toBeVisible();

  const marketTab = page.locator(".document-tabs .tab").first().getByRole("button", { name: "Market Entry Report" });
  await marketTab.focus();
  await expect(marketTab).toBeFocused();
  await marketTab.press("Space");
  await expect(page.locator(".document-tabs .tab").first()).toHaveClass(/active/);

  const commandsButton = page.getByRole("button", { name: "Commands" });
  await commandsButton.focus();
  await expect(commandsButton).toBeFocused();
  await commandsButton.press("Space");
  const commandDialog = page.getByRole("dialog", { name: "Command palette" });
  await expect(commandDialog).toBeVisible();
  await expect(page.getByLabel("Search commands, headings, citations, glossary, index terms, or enter an AI instruction")).toBeFocused();
  await page.keyboard.type("Show document outline");
  await page.keyboard.press("Tab");
  await expect(commandDialog.getByRole("button", { name: /Show document outline Navigate/ })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(commandDialog).toBeHidden();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("outline");

  const primaryKey = process.platform === "darwin" ? "Meta" : "Control";
  await page.locator("#document-workspace").focus();
  await page.keyboard.press(`${primaryKey}+F`);
  await expect(page.locator(".cm-panel.cm-search")).toBeVisible();
  await page.keyboard.press("Escape");
  await page.keyboard.press(`${primaryKey}+Shift+R`);
  await expect(page.getByLabel("View mode")).toHaveValue("review");
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("review");
  await page.keyboard.press(`${primaryKey}+Shift+X`);
  await expect(page.getByLabel("View mode")).toHaveValue("export");
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("exports");
  await page.keyboard.press(`${primaryKey}+Shift+A`);
  const shortcutAgent = page.getByRole("dialog", { name: "AI agent workspace" });
  await expect(shortcutAgent).toBeVisible();
  await shortcutAgent.getByLabel("Close AI agent workspace").click();
  await expect(shortcutAgent).toBeHidden();
  await page.keyboard.press(`${primaryKey}+Shift+L`);
  const shortcutDocsLive = page.getByRole("dialog", { name: "Docs Live voice drafting" });
  await expect(shortcutDocsLive).toBeVisible();
  await shortcutDocsLive.getByLabel("Close Docs Live").click();
  await expect(shortcutDocsLive).toBeHidden();
  await page.keyboard.press(`${primaryKey}+Shift+H`);
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("help");
  await expect(page.getByLabel("Selected help topic")).toContainText("Keyboard shortcuts");
  await expect(page.getByLabel("Selected help topic")).toContainText("AI agent workspace");

  const diagnosticDocument = [
    "---",
    "title: Keyboard Diagnostics",
    "status: draft",
    "---",
    "",
    "# Keyboard Diagnostics",
    "",
    "This line contains DIAGNOSTIC_TARGET for keyboard navigation.",
  ].join("\n");
  await setMockFileText(page, "/workspace/keyboard-diagnostics.md", diagnosticDocument);
  await queueDialogSelection(page, "/workspace/keyboard-diagnostics.md");
  await page.getByRole("button", { name: "Open", exact: true }).focus();
  await page.keyboard.press("Enter");
  await selectSidebarPanelOption(page, "diagnostics");

  const inventory = page.getByLabel("Compiler output inventory");
  await expect(inventory).toContainText("Compiled Markdown");
  await expect(inventory).toContainText("HTML preview");
  await expect(inventory).toContainText("Semantic model");
  await expect(inventory).toContainText("Source map");
  await expect(inventory).toContainText("Export manifest");
  await expect(inventory.locator(".snapshot-row").filter({ hasText: "Transform artifacts" })).toContainText("artifacts");

  const diagnosticsList = page.getByRole("list", { name: "Compiler diagnostics" });
  const diagnostic = diagnosticsList.getByRole("listitem", { name: /warning diagnostic: Mock diagnostic target needs review/ });
  const diagnosticJump = diagnostic.getByRole("button", { name: "Go to source" });
  await diagnosticJump.focus();
  await page.keyboard.press("Enter");
  await expect(page.locator(".cm-line").filter({ hasText: "DIAGNOSTIC_TARGET" })).toBeVisible();

  await page.getByRole("document", { name: /Rendered preview for Keyboard Diagnostics, draft/ }).focus();
  await expect(page.getByRole("document", { name: /Rendered preview for Keyboard Diagnostics, draft/ })).toBeFocused();

  await selectSidebarPanelOption(page, "tables");
  const newTableButton = page.getByRole("button", { name: "New table" });
  await newTableButton.focus();
  await page.keyboard.press("Enter");
  await page.getByLabel("Caption").fill("Keyboard budget");
  await page.getByLabel("Item, row 1, column A").focus();
  await page.keyboard.press(process.platform === "darwin" ? "Meta+A" : "Control+A");
  await page.keyboard.type("Travel");
  await page.getByLabel("Value, row 1, column B").focus();
  await page.keyboard.press(process.platform === "darwin" ? "Meta+A" : "Control+A");
  await page.keyboard.type("450");
  const totalsButton = page.getByRole("button", { name: "Add totals row" });
  await totalsButton.focus();
  await page.keyboard.press("Enter");
  const insertTableButton = page.getByRole("button", { name: "Insert table" });
  await insertTableButton.focus();
  await page.keyboard.press("Enter");
  await expect.poll(() => editorText(page)).toContain("Table: Keyboard budget");
  await expect.poll(() => editorText(page)).toContain("Travel");

  await queueDialogSelection(page, "/workspace/keyboard-diagnostics.md");
  await page.getByRole("button", { name: "Open", exact: true }).focus();
  await page.keyboard.press("Space");
  await selectSidebarPanelOption(page, "review");
  await page.locator(".sidebar").getByLabel("Status").selectOption("in-review");
  await setMockFileText(page, "/workspace/keyboard-diagnostics.md", externalApprovedDocument());
  await page.getByRole("button", { name: "Save", exact: true }).focus();
  await page.keyboard.press("Space");

  const compareButton = page.locator(".status-bar .conflict-actions").getByRole("button", { name: "Compare" });
  await compareButton.focus();
  await page.keyboard.press("Space");
  const conflictDialog = page.getByRole("dialog", { name: "External file conflict" });
  await expect(conflictDialog).toBeVisible();
  await expect(conflictDialog.getByLabel("Close external file conflict")).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  await expect(conflictDialog.getByRole("button", { name: "Accept external" })).toBeFocused();
  await page.keyboard.press("Tab");
  await expect(conflictDialog.getByLabel("Close external file conflict")).toBeFocused();

  await conflictDialog.getByRole("button", { name: "Add external line 3 to merge" }).focus();
  await page.keyboard.press("Space");
  await conflictDialog.getByRole("button", { name: "Add external line 8 to merge" }).focus();
  await page.keyboard.press("Space");
  await expect(conflictDialog.getByLabel("Merged result")).toHaveValue("status: approved\nExternal disk edit.");
  await conflictDialog.getByRole("button", { name: "Move external line 8 up" }).focus();
  await page.keyboard.press("Space");
  await expect(conflictDialog.getByLabel("Merged result")).toHaveValue("External disk edit.\nstatus: approved");
  await conflictDialog.getByRole("button", { name: "Remove external line 8" }).focus();
  await page.keyboard.press("Space");
  await expect(conflictDialog.getByLabel("Merged result")).toHaveValue("status: approved");
  await conflictDialog.getByLabel("Close external file conflict").focus();
  await page.keyboard.press("Escape");
  await expect(conflictDialog).toBeHidden();
  await expect(compareButton).toBeFocused();
});

test("exposes status and progress messages as live regions", async ({ page }) => {
  const statusBar = page.locator("#document-status");
  await expect(statusBar).toHaveAttribute("aria-label", "Document status and progress");
  await expect(statusBar.locator(".status-message")).toHaveAttribute("role", "status");
  await expect(statusBar.locator(".status-message")).toHaveAttribute("aria-live", "polite");
  await expect(statusBar.locator(".status-message")).toHaveAttribute("aria-atomic", "true");
  await expect(statusBar.locator(".word-stats")).toHaveAttribute("aria-label", /Document statistics:/);
});

test("cancels a pending preview compile and resumes editing", async ({ page }) => {
  await setCompileDelay(page, 1200);

  const editorContent = page.locator(".cm-content");
  await editorContent.click();
  await moveEditorCursorToEnd(page);
  await page.keyboard.type("\n\n## Cancelled Compile Target\nThis preview update should be cancelled.");

  const compileStatus = page.getByRole("status", { name: /Compile progress: Compiling preview/ });
  await expect(compileStatus).toBeVisible();
  await compileStatus.getByRole("button", { name: "Cancel compile" }).click();
  await expect(page.locator(".status-bar")).toContainText("Cancelled preview compile");
  await expect(page.getByRole("button", { name: "Cancel compile" })).toBeHidden();

  await setCompileDelay(page, 0);
  await editorContent.click();
  await moveEditorCursorToEnd(page);
  await page.keyboard.type("\n\nCompile resumes after cancellation.");
  await expect(page.getByRole("region", { name: "Live preview" })).toContainText("Compile resumes after cancellation.");
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
    "![Preview figure](assets/preview-navigation.png){#fig:preview-navigation caption=\"Preview figure source\"}",
    "",
    "Table: Preview source map {#tbl:preview-navigation}",
    "| Metric | Value |",
    "| --- | ---: |",
    "| Click target | 1 |",
    "",
    "$$",
    "ROI = Gain / Cost",
    "$$ {#eq:preview-navigation caption=\"Preview equation source\"}",
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

  const targetFigure = previewPane.locator("figure#fig\\:preview-navigation figcaption");
  await targetFigure.scrollIntoViewIfNeeded();
  await targetFigure.click();

  await expect(page.locator(".cm-line").filter({ hasText: "Preview figure" })).toBeVisible();

  const targetTable = previewPane.locator("table#tbl\\:preview-navigation caption");
  await targetTable.scrollIntoViewIfNeeded();
  await targetTable.click();

  await expect(page.locator(".cm-line").filter({ hasText: "Table: Preview source map" })).toBeVisible();

  const targetEquation = previewPane.locator("figure#eq\\:preview-navigation figcaption");
  await targetEquation.scrollIntoViewIfNeeded();
  await targetEquation.click();

  await expect(page.locator(".cm-line").filter({ hasText: "ROI = Gain / Cost" })).toBeVisible();
});

test("renders generated table of contents in preview and links back to source", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/toc-preview.md",
    [
      "---",
      "title: TOC Preview Proof",
      "status: approved",
      "---",
      "",
      "# TOC Preview Proof",
      "",
      "[TOC]",
      "",
      "## Executive Summary",
      "",
      "Opening summary.",
      "",
      "## Findings",
      "",
      "Evidence section.",
      "",
      "### Detail Note",
      "",
      "Drill-down detail.",
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/toc-preview.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  const preview = page.getByRole("region", { name: "Live preview" });
  await expect(preview.getByRole("heading", { name: "Table of Contents" })).toBeVisible();
  await expect(preview.getByRole("link", { name: "Executive Summary" })).toHaveAttribute("href", "#executive-summary");
  await expect(preview.getByRole("link", { name: "Findings" })).toHaveAttribute("href", "#findings");
  await expect(preview.getByRole("link", { name: "Detail Note" })).toHaveAttribute("href", "#detail-note");

  await preview.getByRole("link", { name: "Findings" }).click();
  await expect(page.locator(".cm-line").filter({ hasText: "## Findings" })).toBeVisible();
});

test("updates the live preview after source edits", async ({ page }) => {
  const editorContent = page.locator(".cm-content");
  await editorContent.click();
  await moveEditorCursorToEnd(page);
  await page.keyboard.type("\n\n## Live Typing Target\nLive preview text from source editing.");

  await expect.poll(() => editorText(page)).toContain("## Live Typing Target");
  await expect(page.getByRole("heading", { name: "Live Typing Target" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toContainText("Live preview text from source editing.");
});

test("syncs split source panes through editing, preview, and primary scroll", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/split-source.md",
    [
      "---",
      "title: Split Source Proof",
      "status: draft",
      "---",
      "",
      "# Split Source Proof",
      "",
      "Opening context for dual-pane authoring.",
      "",
      ...Array.from({ length: 32 }, (_, index) => [`## Existing Section ${index + 1}`, "", `Business note ${index + 1}.`, ""]).flat(),
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/split-source.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  const splitGrid = page.locator(".editor-split-grid");
  const splitToggle = page.locator(".compact-check", { hasText: "Dual source" }).getByLabel("Split source editor panes");
  await splitToggle.check();
  await expect(splitGrid).toHaveAttribute("data-split-source", "true");
  await expect(page.locator(".editor-host .cm-content")).toHaveCount(2);

  const primaryEditor = page.getByRole("textbox", { name: "Primary Markdown editor" });
  const secondaryEditor = page.getByRole("textbox", { name: "Secondary Markdown editor" });
  await expect(primaryEditor).toBeVisible();
  await expect(secondaryEditor).toBeVisible();

  await secondaryEditor.click();
  await moveEditorCursorToEnd(page);
  await page.keyboard.insertText("\n\n## Secondary Pane Draft\nThis section was authored from the secondary pane.");

  await primaryEditor.click();
  await moveEditorCursorToEnd(page);
  await expect(page.locator(".editor-host-primary .cm-line").filter({ hasText: "## Secondary Pane Draft" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toContainText("This section was authored from the secondary pane.");

  await moveEditorCursorToEnd(page);
  await page.keyboard.insertText("\n\n## Primary Pane Revision\nThis section was authored from the primary pane.");

  await secondaryEditor.click();
  await moveEditorCursorToEnd(page);
  await expect(page.locator(".editor-host-secondary .cm-line").filter({ hasText: "## Primary Pane Revision" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toContainText("This section was authored from the primary pane.");

  const primaryScroller = page.locator(".editor-host-primary .cm-scroller");
  const secondaryScroller = page.locator(".editor-host-secondary .cm-scroller");
  const previewPane = page.locator(".preview-pane");
  await primaryScroller.evaluate((element) => {
    element.scrollTop = element.scrollHeight * 0.8;
    element.dispatchEvent(new Event("scroll", { bubbles: true }));
  });
  await expect.poll(() => previewPane.evaluate((element) => element.scrollTop)).toBeGreaterThan(20);

  const previewBeforeSecondaryScroll = await previewPane.evaluate((element) => element.scrollTop);
  await secondaryScroller.evaluate((element) => {
    element.scrollTop = element.scrollHeight * 0.2;
    element.dispatchEvent(new Event("scroll", { bubbles: true }));
  });
  await expect.poll(() => previewPane.evaluate((element) => element.scrollTop)).toBe(previewBeforeSecondaryScroll);
});

test("keeps large document editing and preview updates responsive", async ({ page }) => {
  const largeDocument = [
    "---",
    "title: Large Interaction Report",
    "status: draft",
    "---",
    "",
    "# Large Interaction Report",
    "",
    "Opening context for the large-document interaction path.",
    "",
    ...Array.from({ length: 120 }, (_, index) => [
      `## Large Section ${index + 1}`,
      "",
      `Narrative paragraph ${index + 1} with enough business-report text to keep the editor and preview panes scrollable during the interaction test.`,
      "",
      `| Metric | Value |`,
      `| --- | ---: |`,
      `| Revenue | ${1000 + index} |`,
      `| Cost | ${400 + index} |`,
      "",
    ]).flat(),
  ].join("\n");

  await setMockFileText(page, "/workspace/large-interaction.md", largeDocument);
  await queueDialogSelection(page, "/workspace/large-interaction.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  const editorContent = page.locator(".cm-content");
  const editorScroller = page.locator(".cm-scroller");
  const previewPane = page.locator(".preview-pane");
  await expect(page.getByRole("heading", { name: "Large Interaction Report" })).toBeVisible();

  await editorContent.click();
  await moveEditorCursorToEnd(page);
  const startedAt = await page.evaluate(() => performance.now());
  await page.keyboard.insertText("\n\n## Large Interaction Target\nLarge document edit landed.");

  await expect.poll(() => editorText(page)).toContain("Large document edit landed.");
  await expect(previewPane).toContainText("Large document edit landed.");
  const elapsedMs = await page.evaluate((start) => performance.now() - start, startedAt);
  expect(elapsedMs, "large document edit should reach preview promptly in the browser harness").toBeLessThan(3000);
  await expect(page.getByRole("status", { name: /Preview timing: Preview updated in \d+ ms for \d+ characters/ })).toBeVisible();
  await expect(page.locator(".status-bar")).toContainText("Preview updated in");

  await editorScroller.evaluate((element) => {
    element.scrollTop = element.scrollHeight * 0.7;
    element.dispatchEvent(new Event("scroll", { bubbles: true }));
  });
  await expect.poll(() => previewPane.evaluate((element) => element.scrollTop)).toBeGreaterThan(20);
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
  await openSettingsSection(page, "appearance");

  const appShell = page.locator(".app-shell");
  const previewPane = page.locator(".preview-pane");
  const previewDocument = page.locator(".preview-document");
  await page.getByRole("combobox", { name: "Theme", exact: true }).selectOption("dark");
  await page.getByLabel("Preview theme").selectOption("dark");
  await page.getByLabel("High contrast").check();
  await page.getByLabel("Reduced motion").check();
  await page.getByRole("textbox", { name: "Editor font", exact: true }).fill("Courier New, monospace");
  await page.getByRole("spinbutton", { name: "Editor font size" }).fill("18");
  await page.getByRole("spinbutton", { name: "Editor line height" }).fill("1.8");
  await page.getByRole("textbox", { name: "Preview font", exact: true }).fill("Georgia, serif");
  await page.getByRole("spinbutton", { name: "Preview font size" }).fill("19");
  await page.getByRole("spinbutton", { name: "Preview line height" }).fill("1.9");
  await expect(appShell).toHaveAttribute("data-theme", "dark");
  await expect(appShell).toHaveAttribute("data-high-contrast", "true");
  await expect(appShell).toHaveAttribute("data-reduced-motion", "true");
  await expect(previewPane).toHaveAttribute("data-preview-theme", "dark");
  await expect.poll(() => appShell.evaluate((element) => getComputedStyle(element).backgroundColor)).toBe("rgb(255, 255, 255)");
  await expect.poll(() => page.getByRole("button", { name: "Commands" }).evaluate((element) => getComputedStyle(element).borderTopColor)).toBe("rgb(0, 0, 0)");
  await expect.poll(() => editorContent.evaluate((element) => getComputedStyle(element).transitionDuration)).toBe("0s");
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
  await expect.poll(() => appShell.evaluate((element) => getComputedStyle(element).backgroundColor)).toBe("rgb(255, 255, 255)");
  await expect.poll(() => editorContent.evaluate((element) => getComputedStyle(element).transitionDuration)).toBe("0s");
  await expect.poll(() => editorContent.evaluate((element) => getComputedStyle(element).fontSize)).toBe("18px");
  await expect(page.getByLabel("Word wrap")).not.toBeChecked();
  await expect(page.getByLabel("Line numbers")).not.toBeChecked();
  await expect.poll(() => editorContent.evaluate((element) => element.classList.contains("cm-lineWrapping"))).toBe(false);
  await expect(page.locator(".cm-lineNumbers")).toHaveCount(0);

  await editorContent.click();
  await page.getByRole("button", { name: "Find" }).click();
  await page.getByRole("textbox", { name: "Find" }).fill("Acme");
  await page.getByRole("textbox", { name: "Replace" }).fill("Globex");
  await page.locator(".cm-search").getByRole("button", { name: "replace all" }).click();
  await expect.poll(() => editorText(page)).toContain("Find target Globex should be replaceable.");
  await expect.poll(() => editorText(page)).not.toContain("Acme");

  await editorContent.click();
  await moveEditorCursorToEnd(page);
  await page.keyboard.press("Enter");
  await page.keyboard.type("Second item");
  await expect.poll(() => editorText(page)).toContain("- Second item");

  await moveEditorCursorToEnd(page);
  await page.keyboard.press("Enter");
  await page.keyboard.type("(");
  await expect.poll(() => editorText(page)).toContain("()");

  await moveEditorCursorToEnd(page);
  await page.keyboard.press("Enter");
  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Bold selection");
  await page.getByRole("button", { name: /Bold selection Markdown/ }).click();
  await page.keyboard.type("bold shortcut");
  await expect.poll(() => editorText(page)).toContain("**bold shortcut**");

  await moveEditorCursorToEnd(page);
  await page.keyboard.press("Enter");
  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Italic selection");
  await page.getByRole("button", { name: /Italic selection Markdown/ }).click();
  await page.keyboard.type("italic shortcut");
  await expect.poll(() => editorText(page)).toContain("*italic shortcut*");

  await moveEditorCursorToEnd(page);
  await page.keyboard.press("Enter");
  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Inline code selection");
  await page.getByRole("button", { name: /Inline code selection Markdown/ }).click();
  await page.keyboard.type("code shortcut");
  await expect.poll(() => editorText(page)).toContain("`code shortcut`");

  await moveEditorCursorToEnd(page);
  await page.keyboard.press("Enter");
  await page.keyboard.type('"');
  await expect.poll(() => editorText(page)).toContain('""');

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Insert code fence");
  await page.getByRole("button", { name: /Insert code fence Snippet/ }).click();
  await expect.poll(() => editorText(page)).toContain("```markdown");

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Command Target");
  await page.getByRole("button", { name: /Command Target.*Heading line/ }).click();
  await expect(page.locator(".cm-line").filter({ hasText: "## Command Target" })).toBeVisible();
});

test("creates support bundle handoff from settings", async ({ page }) => {
  await openSettingsSection(page, "files");
  await expect(page.getByRole("heading", { name: "Support bundle" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Preview" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Save JSON" })).toBeVisible();
  await expect(page.getByText("not document content or secrets")).toBeVisible();

  await page.getByRole("button", { name: "Preview" }).click();
  await expect(page.getByText("Support bundle preview ready: current-host-ready-with-external-gaps, 1 evidence gaps, 106 open spec rows, engines complete (0 missing), 1 evidence reports need attention")).toBeVisible();
  await expect(page.getByText("Mock support recommendation")).toBeVisible();
  await expect(page.getByText("preview only")).toBeVisible();
  await expect(page.locator("dd").getByText("106 open", { exact: true })).toBeVisible();
  await expect(page.locator("dd").getByText("10 installed, 0 missing", { exact: true })).toBeVisible();
  await expect(page.locator("dd").getByText("1 ready, 1 attention, 0 missing", { exact: true })).toBeVisible();

  await queueDialogSelection(page, "/workspace/neditor-support-bundle.json");
  await page.getByRole("button", { name: "Save JSON" }).click();
  await expect(page.getByText("Wrote support bundle to /workspace/neditor-support-bundle.json")).toBeVisible();
  await expect(page.locator("dd").getByText("/workspace/neditor-support-bundle.json", { exact: true })).toBeVisible();
});

test("runs configurable Emacs and Vim-style editor keybinding modes", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/keybinding-modes.md",
    [
      "---",
      "title: Keybinding Modes",
      "status: draft",
      "---",
      "",
      "# Keybinding Modes",
      "",
      "Opening line for keybinding workflow proof.",
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/keybinding-modes.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await openSettingsSection(page, "appearance");

  const editor = page.getByRole("textbox", { name: "Primary Markdown editor" });
  const keybindings = page.getByLabel("Editor keybindings");
  const status = page.locator(".keymap-status");

  await keybindings.selectOption("emacs");
  await expect(status).toContainText("Emacs-style keys");
  await expect(editor).toHaveAttribute("data-keymap-mode", "emacs");
  await editor.click();
  await moveEditorCursorToEnd(page);
  await page.keyboard.insertText("\nEmacs target");
  await page.keyboard.press("Control+A");
  await page.keyboard.insertText("Start ");
  await page.keyboard.press("Control+E");
  await page.keyboard.insertText(" End");
  await expect.poll(() => editorText(page)).toContain("Start Emacs target End");

  await keybindings.selectOption("vim");
  await expect(status).toContainText("Vim insert mode");
  await expect(editor).toHaveAttribute("data-keymap-mode", "vim");
  await expect(editor).toHaveAttribute("data-vim-mode", "insert");
  await editor.click();
  await moveEditorCursorToEnd(page);
  await page.keyboard.insertText("\nVim target");
  await page.keyboard.press("Escape");
  await expect(status).toContainText("Vim normal mode");
  await expect(editor).toHaveAttribute("data-vim-mode", "normal");

  const beforeBlockedNormalText = await editorText(page);
  await page.keyboard.press("z");
  await expect.poll(() => editorText(page)).toBe(beforeBlockedNormalText);

  await page.keyboard.press("0");
  await page.keyboard.press("i");
  await expect(status).toContainText("Vim insert mode");
  await page.keyboard.insertText("VIM ");
  await expect.poll(() => editorText(page)).toContain("VIM Vim target");
  await page.keyboard.press("Escape");
  await page.keyboard.press("$");
  await page.keyboard.press("a");
  await page.keyboard.insertText(" done");
  await expect.poll(() => editorText(page)).toContain("VIM Vim target done");
  await page.keyboard.press("Escape");
  await page.keyboard.press("I");
  await expect(status).toContainText("Vim insert mode");
  await page.keyboard.insertText("LINESTART ");
  await expect.poll(() => editorText(page)).toContain("LINESTART VIM Vim target done");
  await page.keyboard.press("Escape");
  await page.keyboard.press("A");
  await page.keyboard.insertText(" LINEEND");
  await expect.poll(() => editorText(page)).toContain("LINESTART VIM Vim target done LINEEND");

  await page.keyboard.press("Escape");
  await page.keyboard.press("o");
  await expect(status).toContainText("Vim insert mode");
  await page.keyboard.insertText("Delete me with dd");
  await page.keyboard.press("Escape");
  const beforePendingDeleteText = await editorText(page);
  await page.keyboard.press("d");
  await expect.poll(() => editorText(page)).toBe(beforePendingDeleteText);
  await page.keyboard.press("d");
  await expect.poll(() => editorText(page)).not.toContain("Delete me with dd");

  await page.keyboard.press("G");
  await page.keyboard.press("I");
  await page.keyboard.insertText("\nword alpha beta");
  await page.keyboard.press("Escape");
  await page.keyboard.press("0");
  await page.keyboard.press("w");
  await page.keyboard.press("i");
  await page.keyboard.insertText("WORD-");
  await expect.poll(() => editorText(page)).toContain("word WORD-alpha beta");

  await page.keyboard.press("Escape");
  await page.keyboard.press("G");
  await page.keyboard.press("A");
  await page.keyboard.insertText("\ntrim this line");
  await page.keyboard.press("Escape");
  await page.keyboard.press("d");
  await page.keyboard.press("b");
  await expect.poll(() => editorText(page)).not.toContain("trim this line");
  await page.keyboard.press("0");
  await page.keyboard.press("C");
  await page.keyboard.insertText("replaced tail");
  await expect.poll(() => editorText(page)).toContain("replaced tail");
  await page.keyboard.press("Escape");
  await page.keyboard.press("O");
  await page.keyboard.insertText("join left");
  await page.keyboard.press("Escape");
  await page.keyboard.press("J");
  await expect.poll(() => editorText(page)).toContain("join left replaced tail");
  await page.keyboard.press("o");
  await page.keyboard.insertText("changeable word");
  await page.keyboard.press("Escape");
  await page.keyboard.press("0");
  await page.keyboard.press("c");
  await page.keyboard.press("w");
  await page.keyboard.insertText("changed");
  await expect.poll(() => editorText(page)).toContain("changed word");
  await expect.poll(() => editorText(page)).not.toContain("changeable word");

  await page.getByRole("button", { name: "Save Workspace" }).click();
  await page.reload();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("settings");
  await expect(page.getByLabel("Editor keybindings")).toHaveValue("vim");
  await expect(page.locator(".keymap-status")).toContainText("Vim insert mode");
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
  await selectSidebarPanelOption(page, "outline");

  await page.locator(".sidebar").getByRole("button", { name: "Outline Target" }).click();

  await expect(page.locator(".cm-line").filter({ hasText: "## Outline Target" })).toBeVisible();
  await expect.poll(() => page.locator(".cm-scroller").evaluate((element) => element.scrollTop)).toBeGreaterThan(20);
});

test("creates a document skeleton from an editable outline plan", async ({ page }) => {
  await selectSidebarPanelOption(page, "outline");
  const sidebar = page.locator(".sidebar");
  await expect(sidebar.getByRole("heading", { name: "Plan" })).toBeVisible();

  await sidebar.getByLabel("Document title").fill("Board Decision Memo");
  await sidebar
    .getByLabel("Editable document outline")
    .fill("- Executive Summary\n  - Decision Needed\n  - Key Risks\n- Financial Case\n- Next Steps");
  await sidebar.getByRole("button", { name: "Create document from outline" }).click();

  await expect(page.locator(".cm-line").filter({ hasText: "# Board Decision Memo" })).toBeVisible();
  await expect(page.locator(".cm-line").filter({ hasText: "## Executive Summary" })).toBeVisible();
  await expect(page.locator(".cm-line").filter({ hasText: "### Decision Needed" })).toBeVisible();
  await expect(page.locator(".cm-line").filter({ hasText: "## Financial Case" })).toBeVisible();
  await expect(page.getByRole("region", { name: "Live preview" })).toContainText("Board Decision Memo");
  await expect(sidebar.getByRole("button", { name: /Decision Needed/ })).toBeVisible();
  expect(await editorText(page)).toContain("<!-- Draft this section. -->");
});

test("generates a Docs Live draft from outline, context, and placeholders", async ({ page }) => {
  await page.getByRole("button", { name: "Docs Live", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "Docs Live voice drafting" });
  await expect(dialog).toBeVisible();

  await dialog.getByLabel("Document type").selectOption("proposal");
  await dialog.getByLabel("Document title").fill("Acme Renewal Proposal");
  await dialog.getByLabel("Outline").fill("- Executive Summary\n- Proposed Approach\n- Investment");
  await dialog.getByRole("button", { name: "Build questionnaire" }).click();
  await expect(dialog.getByLabel("AI-created questionnaire")).toHaveValue(/For "Executive Summary"/);
  await setMockClipboardText(page, "<p>Runtime clipboard proof</p>", "text/html");
  await dialog.getByRole("button", { name: "Check AI runtime" }).click();
  await expect(dialog.getByRole("region", { name: "AI runtime readiness" })).toContainText("Runtime readiness");
  await expect(dialog.getByLabel("AI runtime readiness report")).toHaveValue(/Speech recognition/);
  await expect(dialog.getByLabel("AI runtime readiness report")).toHaveValue(/Clipboard rich read succeeded/);
  await dialog.getByRole("button", { name: "Start dictation" }).click();
  await expect(dialog.getByLabel("Spoken direction")).toHaveValue(/Create a client proposal for Acme/);
  await dialog
    .getByLabel("Context and answers")
    .fill("The goal is to renew the platform contract. Include a clear recommendation and review notes.");
  await dialog
    .getByLabel("Questionnaire answers")
    .fill("The reader should approve renewal. Keep pricing assumptions visible for human review.");
  await dialog.getByLabel("Placeholder values").fill("client: Acme\nowner: Commercial team\ndeadline: June 1");

  await dialog.getByRole("button", { name: "Generate draft" }).click();
  await expect(dialog.getByText("3 drafted sections")).toBeVisible();
  await expect(dialog.getByLabel("Docs Live review preparation packet")).toContainText("Section runbook");
  await expect(dialog.getByLabel("Docs Live review preparation packet")).toContainText("Humanization checklist");
  await expect(dialog.getByLabel("Docs Live generated Markdown")).toHaveValue(/provider: NEditor Docs Live/);
  await expect(dialog.getByLabel("Docs Live generated Markdown")).toHaveValue(/Section-by-section Draft Runbook/);
  await dialog.getByRole("button", { name: "Apply draft" }).click();

  await expect(dialog).toBeHidden();
  const text = await editorText(page);
  expect(text).toContain("# Acme Renewal Proposal");
  expect(text).toContain("<!-- ai-assisted: status=needs-review");
  await expect(page.getByRole("region", { name: "Live preview" })).toContainText("Section-by-section Draft Runbook");
  await expect(page.getByRole("region", { name: "Live preview" })).toContainText("Review Preparation");
});

test("edits document structure from outline mode", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/outline-crud.md",
    [
      "---",
      "title: Outline CRUD",
      "status: draft",
      "---",
      "",
      "# Outline CRUD",
      "",
      "Confidential body text should not be shown as an outline row.",
      "",
      "## Market Analysis",
      "",
      "Market body.",
      "",
      "### Risks",
      "",
      "Risk body.",
      "",
      "#### Operational Detail",
      "",
      "Detail body.",
      "",
      "##### Implementation Note",
      "",
      "Deep body.",
      "",
      "## Methods",
      "",
      "Methods body.",
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/outline-crud.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await page.getByLabel("View mode").selectOption("outline");

  const outlineMode = page.locator("#outline-mode");
  await expect(outlineMode).toBeVisible();
  await expect(outlineMode).toContainText("Chapter");
  await expect(outlineMode).toContainText("Section");
  await expect(outlineMode).toContainText("Subsection");
  await expect(outlineMode).toContainText("Subsubsection");
  await expect(outlineMode).not.toContainText("Confidential body text");
  await expect(page.getByLabel("Outline title Implementation Note")).toHaveCount(0);

  const marketTitle = page.getByLabel("Outline title Market Analysis");
  await commitInputValue(marketTitle, "Market Findings");
  await expect(page.getByLabel("Outline title Market Findings")).toBeVisible();
  await page.getByLabel("Outline level Risks").selectOption("2");

  const marketRow = page.locator(".outline-mode-row").filter({ has: page.getByLabel("Outline title Market Findings") });
  await marketRow.getByRole("button", { name: "Add child" }).click();
  await expect(page.getByLabel("Outline title New subsection")).toBeVisible();

  await page.getByLabel("New outline heading title").fill("Appendix");
  await page.getByLabel("New outline heading level").selectOption("1");
  await page.getByRole("button", { name: "Add heading" }).click();
  await expect(page.getByLabel("Outline title Appendix")).toBeVisible();

  const methodsRow = page.locator(".outline-mode-row").filter({ has: page.getByLabel("Outline title Methods") });
  await methodsRow.getByRole("button", { name: "Delete" }).click();
  await expect(page.getByLabel("Outline title Methods")).toHaveCount(0);

  await page.getByLabel("View mode").selectOption("source");
  const source = await editorText(page);
  expect(source).toContain("## Market Findings");
  expect(source).toContain("## Risks");
  expect(source).toContain("### New subsection");
  expect(source).toContain("##### Implementation Note");
  expect(source).toContain("# Appendix");
  expect(source).not.toContain("## Methods");
});

test("folds and unfolds Markdown sections from toolbar and commands", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/folding-ergonomics.md",
    [
      "---",
      "title: Folding Ergonomics",
      "status: draft",
      "---",
      "",
      "# Folding Ergonomics",
      "",
      "## Section One",
      "",
      "First foldable section paragraph.",
      "",
      "```calc",
      "revenue = 125000",
      "cost = 74000",
      "profit = revenue - cost",
      "```",
      "",
      "## Section Two",
      "",
      "Second foldable section paragraph.",
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/folding-ergonomics.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  const foldedPlaceholders = page.locator(".cm-foldPlaceholder");
  await expect(foldedPlaceholders).toHaveCount(0);
  await page.getByRole("button", { name: "Fold", exact: true }).click();
  await expect.poll(async () => foldedPlaceholders.count()).toBeGreaterThan(0);

  await page.getByRole("button", { name: "Unfold", exact: true }).click();
  await expect(foldedPlaceholders).toHaveCount(0);
  await expect(page.locator(".cm-line").filter({ hasText: "profit = revenue - cost" })).toBeVisible();

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Fold all sections");
  await page.getByRole("button", { name: /Fold all sections Navigate/ }).click();
  await expect.poll(async () => foldedPlaceholders.count()).toBeGreaterThan(0);

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Unfold all sections");
  await page.getByRole("button", { name: /Unfold all sections Navigate/ }).click();
  await expect(foldedPlaceholders).toHaveCount(0);
  await expect(page.locator(".cm-line").filter({ hasText: "Second foldable section paragraph." })).toBeVisible();
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
    '![Architecture diagram](assets/architecture.png){#fig:architecture caption="Architecture diagram" fit="cover" position="top"}',
    "",
    "## Diagnostic Target",
    "",
    "This line contains DIAGNOSTIC_TARGET for source navigation.",
    "",
  ].join("\n");

  await setMockFileText(page, "/workspace/diagnostic-navigation.md", diagnosticDocument);
  await queueDialogSelection(page, "/workspace/diagnostic-navigation.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  const previewDiagnostic = page.locator(".preview-diagnostic").filter({ hasText: "Mock diagnostic target needs review." });
  await expect(previewDiagnostic).toContainText("Resolve the marked diagnostic target before publishing.");
  await expect(previewDiagnostic).toContainText("/workspace/diagnostic-navigation.md: line");
  await previewDiagnostic.getByRole("button", { name: "Go to source" }).click();
  await expect(page.locator(".cm-line").filter({ hasText: "DIAGNOSTIC_TARGET" })).toBeVisible();
  await expect.poll(() => page.locator(".cm-scroller").evaluate((element) => element.scrollTop)).toBeGreaterThan(20);

  await selectSidebarPanelOption(page, "diagnostics");

  const diagnosticInventory = page.getByLabel("Compiler output inventory");
  await expect(diagnosticInventory).toContainText("Compiled Markdown");
  await expect(diagnosticInventory).toContainText("HTML preview");
  await expect(diagnosticInventory).toContainText("Semantic model");
  await expect(diagnosticInventory).toContainText("Source map");
  await expect(diagnosticInventory).toContainText("Export manifest");
  await expect(diagnosticInventory.locator(".snapshot-row").filter({ hasText: "Transform artifacts" })).toContainText("artifacts");
  await expect(diagnosticInventory.locator(".snapshot-row").filter({ hasText: "Media map" })).toContainText("1 media files");
  await expect(diagnosticInventory.locator(".snapshot-row").filter({ hasText: "Figure media uses" })).toContainText("1 figure uses");

  const diagnosticsList = page.getByRole("list", { name: "Compiler diagnostics" });
  const diagnostic = diagnosticsList.getByRole("listitem", { name: /warning diagnostic: Mock diagnostic target needs review/ });
  await expect(diagnostic).toBeVisible();
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
  await moveEditorCursorToEnd(page);
  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("multi cursor");
  await expect(page.getByRole("button", { name: /Add cursor above.*line above.*Edit/ })).toBeVisible();
  await expect(page.getByRole("button", { name: /Add cursor below.*line below.*Edit/ })).toBeVisible();
  await expect(page.getByRole("button", { name: /Select next occurrence.*simultaneous editing.*Edit/ })).toBeVisible();
  await page.getByRole("button", { name: /Add cursor above.*Edit/ }).click();
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
  await page.getByRole("button", { name: /\[@risk2026.*Citation/ }).click();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("references");
  await expect(page.locator(".cm-line").filter({ hasText: "Citation target cites" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Resolved references" })).toBeVisible();
  await expect(page.locator(".sidebar").getByText("Risk Operating Model").first()).toBeVisible();
  await expect(page.getByRole("heading", { name: "Missing keys" })).toBeVisible();
  await expect(page.locator(".sidebar").getByText("@missing2026", { exact: true })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Duplicate keys" })).toBeVisible();
  await expect(page.locator(".sidebar").getByText("@dup2026", { exact: true }).first()).toBeVisible();
  await expect(page.getByText("/workspace/reference-navigation.md:17")).toBeVisible();
  await expect(page.getByText("/workspace/reference-navigation.md:22")).toBeVisible();

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("ARR");
  await page.getByRole("button", { name: /ARR.*Glossary/ }).click();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("references");
  await expect(page.locator(".cm-line").filter({ hasText: "ARR: Annual recurring revenue." })).toBeVisible();

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Working Capital");
  await page.getByRole("button", { name: /Working Capital.*Index/ }).click();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("references");
  await expect(page.locator(".cm-line").filter({ hasText: "Working capital" })).toBeVisible();

  const glossaryManager = page.getByRole("region", { name: "Glossary manager" });
  await expect(glossaryManager).toContainText("1 glossary terms");
  await expect(glossaryManager).toContainText("marker missing");
  await expect(glossaryManager).toContainText("ARR");
  await expect(glossaryManager).toContainText("Annual recurring revenue.");
  await glossaryManager.getByRole("button", { name: "Include glossary in exports" }).click();
  await expect(glossaryManager).toContainText("included in exports");
  await glossaryManager.getByRole("button", { name: "Add ARR to index" }).click();
  await expect.poll(() => editorText(page)).toContain("#index:ARR");
  await glossaryManager.getByRole("button", { name: "Insert glossary audit" }).click();
  await expect.poll(() => editorText(page)).toContain("## Glossary Audit");
  await expect.poll(() => editorText(page)).toContain("| ARR | Annual recurring revenue. | review |");

  const indexManager = page.getByRole("region", { name: "Index manager" });
  await expect(indexManager).toContainText("2 index terms");
  await expect(indexManager).toContainText("0 exclusions");
  await expect(indexManager).toContainText("front matter index: not set");
  await indexManager.getByRole("button", { name: "Insert generated index" }).click();
  await expect.poll(() => editorText(page)).toContain("[INDEX]");
  await expect(indexManager).toContainText("marker present");
  await indexManager.getByRole("button", { name: "Enable front matter index" }).click();
  await expect.poll(() => editorText(page)).toContain("index: true");
  await expect(indexManager).toContainText("front matter index: true");
  await indexManager.getByPlaceholder("Internal Draft, Secret Plan").fill("Internal Draft");
  await indexManager.getByRole("button", { name: "Exclude term" }).click();
  await expect(indexManager).toContainText("1 exclusions");
  await expect(indexManager).toContainText("Internal Draft");
  await indexManager.getByRole("button", { name: "Insert index audit" }).click();
  await expect.poll(() => editorText(page)).toContain("## Index Audit");
  await expect.poll(() => editorText(page)).toContain("| Working Capital | indexed | review |");
  await expect.poll(() => editorText(page)).toContain("| Internal Draft | excluded | confirm exclusion |");

  await glossaryManager.getByRole("button", { name: "Insert generated glossary" }).click();
  await expect.poll(() => editorText(page)).toContain("[GLOSSARY]");
  await expect(glossaryManager).toContainText("marker present");
});

test("manages front matter data sources from the references panel", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/.neditor/variables.yaml",
    ["variables:", "  projectLead: Strategy PMO", "  projectCapex: 450000"].join("\n"),
  );
  await setMockFileText(
    page,
    "/workspace/data-source-ui.md",
    [
      "---",
      "title: Data Source UI",
      "status: draft",
      "client: Example Corp",
      "account:",
      "  ownerName: Asha M.",
      "  renewalValue: 90000",
      "budget: 125000",
      "owner:",
      "dataSources:",
      "  - name: Revenue",
      "    path: data/revenue.csv",
      "    type: csv",
      "  - name: Escape",
      "    path: ../secret.json",
      "    type: json",
      "jsonFiles:",
      "  - data/accounts.json",
      "---",
      "",
      "# Data Source UI",
      "",
      "Data source manager proof.",
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/data-source-ui.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await selectSidebarPanelOption(page, "references");

  const dataSources = page.getByRole("region", { name: "Local data source manager" });
  await expect(dataSources).toContainText("3 local data sources | 2 ready | 1 need attention");
  await expect(dataSources.locator(".snapshot-row").filter({ hasText: "Revenue" })).toContainText("CSV | ready");
  await expect(dataSources.locator(".snapshot-row").filter({ hasText: "Escape" })).toContainText("blocked-path");
  await expect(dataSources).toContainText("data/accounts.json");

  await dataSources.getByPlaceholder("Revenue, Accounts, Settings").fill("Targets");
  await dataSources.getByPlaceholder("data/revenue.csv").fill("data/targets.tsv");
  await dataSources.getByLabel("Data source type").selectOption("tsv");
  await dataSources.getByRole("button", { name: "Add data source" }).click();
  await expect.poll(() => editorText(page)).toContain('name: "Targets"');
  await expect.poll(() => editorText(page)).toContain('path: "data/targets.tsv"');
  await expect.poll(() => editorText(page)).toContain("type: tsv");

  const variables = page.getByRole("region", { name: "Document variable manager" });
  await expect(variables).toContainText("front matter variables");
  await expect(variables).toContainText("project/merged variables");
  await expect(variables.locator(".snapshot-row").filter({ hasText: "client" })).toContainText("Example Corp");
  await expect(variables.locator(".snapshot-row").filter({ hasText: "account.ownerName" })).toContainText("Asha M.");
  await expect(variables.locator(".snapshot-row").filter({ hasText: "account.renewalValue" })).toContainText("90000");
  await expect(variables).toContainText("owner");
  await expect(variables).toContainText("empty");
  await expect(variables.locator(".snapshot-row").filter({ hasText: "projectLead" })).toContainText("Strategy PMO");
  await variables.getByLabel("Document variable insert filter").selectOption("currency");
  await variables.locator(".snapshot-row").filter({ hasText: "budget" }).getByRole("button", { name: "Insert variable" }).click();
  await expect.poll(() => editorText(page)).toContain("{{budget | currency}}");
  await variables.getByLabel("Document variable insert filter").selectOption("upper");
  await variables.locator(".snapshot-row").filter({ hasText: "projectLead" }).getByRole("button", { name: "Insert variable" }).click();
  await expect.poll(() => editorText(page)).toContain("{{projectLead | upper}}");
  await variables.getByLabel("Document variable insert filter").selectOption("title");
  await variables.locator(".snapshot-row").filter({ hasText: "account.ownerName" }).getByRole("button", { name: "Insert variable" }).click();
  await expect.poll(() => editorText(page)).toContain("{{account.ownerName | title}}");
  await variables.getByPlaceholder("client, owner, budget").fill("reviewer");
  await variables.getByPlaceholder("Example Corp, Strategy Office, 125000").fill("QA Lead");
  await variables.getByRole("button", { name: "Add variable" }).click();
  await expect.poll(() => editorText(page)).toContain('reviewer: "QA Lead"');
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
  await page.getByRole("button", { name: /Command First.*Open document/ }).click();
  await expect(page.locator(".document-tabs .tab.active")).toContainText("Command First");
  await expect.poll(() => editorText(page)).toContain("First command document body.");

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("workspace-target");
  await page.getByRole("button", { name: /reports\/workspace-target\.md.*Workspace file/ }).click();
  await expect(page.locator(".document-tabs .tab.active")).toContainText("Workspace Target");
  await expect.poll(() => editorText(page)).toContain("Workspace command body.");
  await selectSidebarPanelOption(page, "files");
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

  await openSettingsSection(page, "transforms");
  const engine = page.locator(".engine-row").filter({ has: page.getByRole("heading", { name: "d2" }) });
  const enginePath = engine.getByLabel("Engine path");
  const trusted = engine.getByLabel("Trusted");

  await expect(engine).toContainText("Runs only with explicit trust");
  await expect(engine).toContainText("Version probe: d2 --version");
  await expect(engine).toContainText("No external executable path is configured");
  await expect(engine).toContainText("compiler diagnostics will explain fallback rendering");

  await setEnginePath(enginePath, "/usr/local/bin/d2");
  await expect(trusted).not.toBeChecked();
  await expect(engine).toContainText("Probe required after engine path change.");
  await expect(engine).toContainText("Executable path is configured but not trusted yet");

  await confirmTransformTrustPrompt(page, "d2", true);
  await expect(trusted).toBeChecked();
  await expect(engine).toContainText("External executable path is trusted");

  await engine.getByLabel("Input").selectOption("file");
  await page.getByLabel("Timeout").fill("7750");
  await page.getByLabel("Timeout").dispatchEvent("change");
  await engine.getByRole("button", { name: "Probe" }).click();
  await expect(engine).toContainText("Probe passed");
  await expect(engine).toContainText("d2 probe ok via file with timeout 7750");
  await expect(engine).toContainText("Cache: d2:/usr/local/bin/d2:file:7750");

  await setEnginePath(enginePath, "/missing/d2");
  await expect(trusted).not.toBeChecked();

  await confirmTransformTrustPrompt(page, "d2", true);
  await expect(trusted).toBeChecked();
  await engine.getByRole("button", { name: "Probe" }).click();
  await expect(engine).toContainText("Probe failed");
  await expect(engine).toContainText("d2 executable not found at /missing/d2.");

  await setEnginePath(enginePath, "/opt/bin/d2");
  await expect(trusted).not.toBeChecked();
  await confirmTransformTrustPrompt(page, "d2", false);
  await expect(trusted).not.toBeChecked();
  await expect(engine).toContainText("Probe required after engine path change.");
  await engine.getByLabel("Disable external engine").check();
  await expect(engine).toContainText("External execution is disabled");
});

test("manages transform templates and inserts reusable workflows", async ({ page }) => {
  await page.getByRole("button", { name: "Templates" }).click();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("templates");

  const sidebar = page.locator(".sidebar");
  const templateFilters = page.getByRole("region", { name: "Transform template filters" });
  await expect(sidebar.getByRole("heading", { name: /Templates/ })).toBeVisible();
  await templateFilters.getByLabel("Category").selectOption("Science");
  await templateFilters.getByLabel("Transform").selectOption("calc");
  await templateFilters.getByLabel("Search").fill("dose");

  const doseTemplate = sidebar.locator("article.template-card").filter({ hasText: "Dose by weight" });
  await expect(doseTemplate).toContainText("Science | calc | builtin");
  await expect(doseTemplate.getByLabel("Template fill values")).toContainText("weight_kg");
  await expect(doseTemplate.getByLabel("Template fill values")).toContainText("tablet_strength_mg");
  await expect(doseTemplate).toContainText("clinical");
  await doseTemplate.getByText("Preview").click();
  await expect(doseTemplate.locator("pre")).toContainText("total_dose_mg");
  await doseTemplate.getByRole("button", { name: "Insert" }).click();

  await expect.poll(() => editorText(page)).toContain("weight_kg = 72");
  await expect.poll(() => editorText(page)).toContain("Total dose: {{=total_dose_mg}} mg");
  await expect(page.locator(".status-bar")).toContainText("Inserted Dose by weight template");

  await templateFilters.getByLabel("Category").selectOption("Charts");
  await templateFilters.getByLabel("Transform").selectOption("chart");
  await templateFilters.getByLabel("Search").fill("kpi");
  const chartTemplate = sidebar.locator("article.template-card").filter({ hasText: "KPI bar chart" });
  await expect(chartTemplate).toContainText("Charts | chart | builtin");
  await chartTemplate.getByRole("button", { name: "Insert" }).click();
  await expect.poll(() => editorText(page)).toContain("```chart");
  await expect.poll(() => editorText(page)).toContain("title: Quarterly KPI plan");

  await templateFilters.getByLabel("Category").selectOption("Business");
  await templateFilters.getByLabel("Transform").selectOption("timeline");
  await templateFilters.getByLabel("Search").fill("launch");
  const timelineTemplate = sidebar.locator("article.template-card").filter({ hasText: "Launch timeline" });
  await expect(timelineTemplate).toContainText("Business | timeline | builtin");
  await timelineTemplate.getByRole("button", { name: "Insert" }).click();
  await expect.poll(() => editorText(page)).toContain("2026-06-15: Pilot launch");

  await templateFilters.getByLabel("Transform").selectOption("roadmap");
  await templateFilters.getByLabel("Search").fill("quarterly");
  const roadmapTemplate = sidebar.locator("article.template-card").filter({ hasText: "Quarterly roadmap" });
  await expect(roadmapTemplate).toContainText("Business | roadmap | builtin");
  await roadmapTemplate.getByRole("button", { name: "Insert" }).click();
  await expect.poll(() => editorText(page)).toContain("Now: Harden editor ergonomics");

  await templateFilters.getByLabel("Transform").selectOption("adr");
  await templateFilters.getByLabel("Search").fill("architecture");
  const adrTemplate = sidebar.locator("article.template-card").filter({ hasText: "Architecture decision" });
  await expect(adrTemplate).toContainText("Business | adr | builtin");
  await adrTemplate.getByRole("button", { name: "Insert" }).click();
  await expect.poll(() => editorText(page)).toContain("Consequences: List expected benefits");

  await templateFilters.getByLabel("Transform").selectOption("qr");
  await templateFilters.getByLabel("Search").fill("release");
  const qrTemplate = sidebar.locator("article.template-card").filter({ hasText: "Release QR code" });
  await expect(qrTemplate).toContainText("Business | qr | builtin");
  await qrTemplate.getByRole("button", { name: "Insert" }).click();
  await expect.poll(() => editorText(page)).toContain("https://example.com/releases/neditor-report");

  await templateFilters.getByLabel("Category").selectOption("Data");
  await templateFilters.getByLabel("Transform").selectOption("openapi");
  await templateFilters.getByLabel("Search").fill("endpoint");
  const openApiTemplate = sidebar.locator("article.template-card").filter({ hasText: "OpenAPI endpoint" });
  await expect(openApiTemplate).toContainText("Data | openapi | builtin");
  await openApiTemplate.getByRole("button", { name: "Insert" }).click();
  await expect.poll(() => editorText(page)).toContain("openapi: 3.1.0");
  await expect.poll(() => editorText(page)).toContain("/reports:");

  await templateFilters.getByLabel("Transform").selectOption("json-schema");
  await templateFilters.getByLabel("Search").fill("schema");
  const schemaTemplate = sidebar.locator("article.template-card").filter({ hasText: "JSON Schema object" });
  await expect(schemaTemplate).toContainText("Data | json-schema | builtin");
  await schemaTemplate.getByRole("button", { name: "Insert" }).click();
  await expect.poll(() => editorText(page)).toContain('"title": "Customer"');
  await expect.poll(() => editorText(page)).toContain('"starter", "growth", "enterprise"');

  await page.getByLabel("View mode").selectOption("preview");
  const transformPreview = page.getByRole("region", { name: "Transform artifact preview" });
  await expect(transformPreview).toContainText("chart");
  await expect(transformPreview).toContainText("timeline");
  await expect(transformPreview).toContainText("roadmap");
  await expect(transformPreview).toContainText("adr");
  await expect(transformPreview).toContainText("qr");
  await expect(transformPreview).toContainText("openapi");
  await expect(transformPreview).toContainText("json-schema");
  await page.getByLabel("View mode").selectOption("split");

  const customEditor = page.getByRole("region", { name: "Custom transform template editor" });
  await customEditor.getByLabel("Name").fill("Custom Safety Margin");
  await customEditor.getByLabel("Category").fill("Business");
  await customEditor.getByLabel("Transform").selectOption("calc");
  await customEditor.getByLabel("Summary").fill("Reusable safety stock margin calculation.");
  await customEditor.getByLabel("Tags").fill("operations, margin");
  await customEditor.getByLabel("Body").fill(
    [
      "```calc",
      "required_units = 100",
      "available_units = 135",
      "margin_units = available_units - required_units",
      "```",
      "Margin: {{=margin_units}} units",
    ].join("\n"),
  );
  await customEditor.getByRole("button", { name: "Create custom" }).click();
  await expect(page.locator(".status-bar")).toContainText("Saved Custom Safety Margin template");

  await templateFilters.getByLabel("Category").selectOption("all");
  await templateFilters.getByLabel("Transform").selectOption("all");
  await templateFilters.getByLabel("Search").fill("safety margin");
  const customTemplate = sidebar.locator("article.template-card").filter({ hasText: "Custom Safety Margin" });
  await expect(customTemplate).toContainText("Business | calc | custom");
  await customTemplate.getByRole("button", { name: "Edit" }).click();
  await customEditor.getByLabel("Summary").fill("Reusable safety stock margin calculation with review note.");
  await customEditor.getByRole("button", { name: "Save custom" }).click();
  await expect(customTemplate).toContainText("review note");
  await customTemplate.getByRole("button", { name: "Duplicate" }).click();
  await customEditor.getByLabel("Name").fill("Temporary Safety Margin Template");
  await customEditor.getByRole("button", { name: "Create custom" }).click();
  const temporaryTemplate = sidebar.locator("article.template-card").filter({ hasText: "Temporary Safety Margin Template" });
  await expect(temporaryTemplate).toContainText("Business | calc | custom");
  await temporaryTemplate.getByRole("button", { name: "Delete" }).click();
  await expect(temporaryTemplate).toHaveCount(0);
  await customTemplate.getByRole("button", { name: "Insert" }).click();
  await expect.poll(() => editorText(page)).toContain("margin_units = available_units - required_units");

  await page.getByRole("button", { name: "Save Workspace" }).click();
  await page.reload();
  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Custom Safety Margin");
  await page.getByRole("button", { name: "Insert Custom Safety Margin template Template Business" }).click();
  await expect.poll(() => editorText(page)).toContain("Margin: {{=margin_units}} units");
});

test("builds business documents from saved identity snippets and local-agent handoff", async ({ page }) => {
  await page.getByRole("button", { name: "Templates" }).click();
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("templates");

  let identity = page.getByRole("region", { name: "Business document creation" });
  await expect(identity).toContainText("No company saved yet");
  await identity.getByRole("button", { name: "Business info" }).click();

  const profile = page.getByRole("dialog", { name: "Business identity setup" });
  await expect(profile).toBeVisible();
  await profile.getByLabel("Your name").fill("Jane Doe");
  await profile.getByLabel("Email address").fill("jane@example.com");
  await profile.getByLabel("Phone").fill("+1 555 0100");
  await profile.getByLabel("Role or title").fill("Managing Partner");
  await profile.getByLabel("Company name").fill("Acme Advisory");
  await profile.getByLabel("Company address").fill("123 Market Street");
  await profile.getByLabel("Website").fill("https://acme.example");
  await profile.getByLabel("Industry").fill("strategy consulting");
  await profile.getByLabel("Default client").fill("Globex");
  await profile.getByLabel("Brand voice").fill("clear and practical");
  await expect(profile.getByLabel("Business identity placeholder preview").locator("textarea")).toHaveValue(/companyName: Acme Advisory/);
  await expect(profile.getByLabel("Local agent integrations")).toContainText("Claude Code");
  await expect(profile.getByLabel("Local agent integrations")).toContainText("codex");
  await expect(profile.getByLabel("Local agent integrations")).toContainText("opencode");
  await profile.getByRole("button", { name: "Save business identity" }).click();
  await expect(profile).toBeHidden();
  await expect(identity).toContainText("Acme Advisory");
  await expect(identity).toContainText("jane@example.com");

  await page.reload();
  await page.getByRole("button", { name: "Templates" }).click();
  identity = page.getByRole("region", { name: "Business document creation" });
  await expect(identity).toContainText("Acme Advisory");
  await expect(identity).toContainText("https://acme.example");

  const snippets = page.getByRole("region", { name: "Reusable document parts" });
  await snippets.getByLabel("Find a part").fill("company overview");
  const overview = snippets.getByRole("listitem").filter({ hasText: "Company overview" });
  await overview.getByRole("button", { name: "Insert" }).click();
  await expect.poll(() => editorText(page)).toContain("Acme Advisory is a strategy consulting organization");
  await expect.poll(() => editorText(page)).toContain("clear and practical communication");

  const wizard = page.locator('section[aria-label="AI document creation wizard"]');
  await wizard.getByLabel("Find a document type").fill("lesson");
  await expect(wizard.getByRole("listitem").filter({ hasText: "Lesson plan" })).toContainText("Assessment");
  await expect(wizard.getByRole("listitem").filter({ hasText: "Lesson content" })).toContainText("Learner Handout");
  await wizard.getByLabel("Find a document type").fill("movie");
  await expect(wizard.getByRole("listitem").filter({ hasText: "Movie script" })).toContainText("Act III");
  await wizard.getByLabel("Find a document type").fill("rfp");
  const rfp = wizard.getByRole("listitem").filter({ hasText: "RFP response" });
  await expect(rfp).toContainText("Compliance Matrix");
  await expect(rfp).toContainText("Competitive bids");

  await rfp.getByRole("button", { name: "AI wizard" }).click();
  const docsLive = page.getByRole("dialog", { name: "Docs Live voice drafting" });
  await expect(docsLive).toBeVisible();
  await expect(docsLive.getByLabel("Document type")).toHaveValue("rfp-response");
  await expect(docsLive.getByLabel("Document title")).toHaveValue(/RFP response for Globex/);
  await expect(docsLive.getByLabel("Outline")).toHaveValue(/Compliance Matrix/);
  await expect(docsLive.getByLabel("Placeholder values")).toHaveValue(/companyName: Acme Advisory/);
  await expect(docsLive.getByLabel("Context and answers")).toHaveValue(/Agent handoff options/);
  await expect(docsLive.getByLabel("AI document creation wizard stages")).toContainText("Quality assurance");
  await docsLive.getByRole("button", { name: "Close Docs Live" }).click();
  await expect(docsLive).toBeHidden();

  await rfp.getByRole("button", { name: "Agent handoff" }).click();
  const agent = page.getByRole("dialog", { name: "AI agent workspace" });
  await expect(agent).toBeVisible();
  await expect(agent.getByLabel("AI provider handoff")).toContainText("Claude Code CLI handoff");
  await expect(agent.getByLabel("AI provider request Markdown")).toHaveValue(/Claude Code CLI handoff Request Package/);
  await expect(agent.getByLabel("AI provider request Markdown")).toHaveValue(/Local Agent Handoff/);
  await expect(agent.getByLabel("AI provider request Markdown")).toHaveValue(/```bash\nclaude\n```/);
  await agent.getByRole("button", { name: "Close AI agent workspace" }).click();
  await expect(agent).toBeHidden();

  const rfpWizard = page.getByRole("region", { name: "Native RFP response wizard" });
  await rfpWizard.getByLabel("RFP source text").fill([
    "Purpose: Globex seeks a partner to modernize customer support while reducing delivery risk.",
    "1. Vendor must provide a phased implementation plan within 90 days.",
    "2. Proposer shall include pricing, payment terms, and all assumptions.",
    "3. Vendor must demonstrate SOC 2 security controls and data protection practices.",
    "4. Submit signed insurance certificate and three relevant customer references.",
    "Evaluation criteria: technical merit 40 points, price 30 points, experience 30 points.",
  ].join("\n"));
  await rfpWizard.getByRole("button", { name: "Analyze RFP" }).click();
  await expect(rfpWizard.getByRole("region", { name: "RFP analysis results" })).toContainText("5 requirements");
  await expect(rfpWizard.getByText("Stated buyer intent", { exact: true })).toBeVisible();
  await expect(rfpWizard.getByText("Implied buyer intent", { exact: true })).toBeVisible();
  await expect(rfpWizard).toContainText("modernize customer support");
  await expect(rfpWizard).toContainText("easily scored response");
  await expect(rfpWizard).toContainText("RFP-REQ-001");
  await rfpWizard.getByRole("button", { name: "Create response" }).click();
  await expect.poll(() => editorText(page)).toContain("## Buyer Intent Analysis");
  await expect(page.getByRole("heading", { name: "Compliance Matrix" })).toBeVisible();
  await expect(page.getByText("RFP-REQ-003").first()).toBeVisible();
  await expect(page.getByText("Every RFP requirement appears in the compliance matrix").first()).toBeVisible();

  await page.getByRole("button", { name: "Equation" }).click();
  const equationEditor = page.getByRole("dialog", { name: "Equation editor" });
  await expect(equationEditor).toBeVisible();
  await equationEditor.getByLabel("Equation template category").selectOption("Business");
  await equationEditor.getByLabel("Search equation templates").fill("total cost");
  await expect(equationEditor.getByText("1 templates")).toBeVisible();
  await equationEditor.getByRole("button", { name: "Load Total cost equation template" }).click();
  await expect(equationEditor.getByLabel("Equation LaTeX")).toHaveValue(/C_\{implementation\}/);
  await expect(equationEditor.getByLabel("Equation Markdown preview")).toHaveValue(/#eq:total-cost/);
  await equationEditor.getByLabel("Equation template category").selectOption("Science");
  await equationEditor.getByLabel("Search equation templates").fill("molarity");
  await equationEditor.getByRole("button", { name: "Load Molarity equation template" }).click();
  await expect(equationEditor.getByLabel("Equation LaTeX")).toHaveValue("M=\\frac{n}{V}");
  await expect(equationEditor.getByLabel("Equation Markdown preview")).toHaveValue(/#eq:molarity/);
  await equationEditor.getByLabel("Equation mode").selectOption("inline");
  await equationEditor.getByLabel("Equation LaTeX").fill("a^2+b^2=c^2");
  await expect(equationEditor.getByLabel("Equation Markdown preview")).toHaveValue("$a^2+b^2=c^2$");
  await equationEditor.getByRole("button", { name: "Insert equation" }).click();
  await expect(equationEditor).toBeHidden();
  await expect.poll(() => editorText(page)).toContain("$a^2+b^2=c^2$");
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

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Insert glossary section");
  await page.getByRole("button", { name: "Insert glossary section Snippet" }).click();
  await expect.poll(() => editorText(page)).toContain("[GLOSSARY]");

  await selectSidebarPanelOption(page, "tables");
  await page.getByRole("button", { name: "New table" }).click();
  await page.getByLabel("Caption").fill("Workflow budget");
  await page.getByRole("button", { name: "Add totals row" }).click();
  await page.getByRole("button", { name: "Insert table" }).click();

  await expect.poll(() => editorText(page)).toContain("Table: Workflow budget");
  await expect.poll(() => editorText(page)).toContain("Total");
  await expect(page.getByLabel("Item, row 1, column A")).toHaveValue("Revenue");

  await page.getByRole("button", { name: "Find" }).click();
  await page.getByRole("textbox", { name: "Find" }).fill("Revenue");
  await page.getByRole("textbox", { name: "Replace" }).fill("Pipeline");
  await page.locator(".cm-search").getByRole("button", { name: "replace all" }).click();

  await expect.poll(() => editorText(page)).toContain("Pipeline");
  await expect(page.getByLabel("Item, row 1, column A")).toHaveValue("Pipeline");
});

test("opens, saves, duplicates, renames, reveals, and reverts mocked files", async ({ page }) => {
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  await expect.poll(() => editorText(page)).toContain("Original saved content.");
  await expect(page).toHaveTitle(/Market Entry Report - NEditor$/);
  await expect(page.getByText("/workspace")).toBeVisible();
  await expect(page.getByRole("button", { name: /market\.md/ }).first()).toBeVisible();

  await selectSidebarPanelOption(page, "review");
  await page.locator(".sidebar").getByLabel("Status").selectOption("in-review");
  await expect.poll(() => editorText(page)).toContain("status: in-review");
  await expect(page).toHaveTitle("* Market Entry Report - NEditor");
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("status: in-review");
  await expect(page).toHaveTitle("market.md - NEditor");

  await queueDialogSelection(page, "/workspace/market copy.md");
  await page.getByRole("button", { name: "Duplicate" }).click();
  await expect.poll(() => mockFileText(page, "/workspace/market copy.md")).toContain("status: in-review");
  await selectSidebarPanelOption(page, "files");
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

  await selectSidebarPanelOption(page, "review");
  await page.locator(".sidebar").getByLabel("Status").selectOption("approved");
  await expect.poll(() => editorText(page)).toContain("status: approved");

  await queueDialogSelection(page, "/workspace/market-approved.md");
  await page.getByRole("button", { name: "Save As" }).click();
  await expect.poll(() => mockFileText(page, "/workspace/market-approved.md")).toContain("status: approved");
  await expect(page.locator(".document-tabs .tab.active")).toContainText("market-approved.md");

  await page.locator(".document-tabs .tab.active").getByLabel("Close document").click();
  await expect(page.locator(".document-tabs .tab.active")).not.toContainText("market-approved.md");

  await openSettingsSection(page, "files");
  const recentlyClosed = page.getByLabel("Recently closed documents");
  await recentlyClosed.getByRole("button", { name: "/workspace/market-approved.md" }).click();

  await expect.poll(() => editorText(page)).toContain("status: approved");
  await expect(recentlyClosed.getByRole("button", { name: "/workspace/market-approved.md" })).toHaveCount(0);
  await selectSidebarPanelOption(page, "files");
  await expect.poll(() => activeFileRowText(page)).toContain("market-approved.md");
});

test("runs snapshot restore and release tagging workflows", async ({ page }) => {
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await expect(page.getByRole("status", { name: "Release status draft" })).toBeVisible();

  await selectSidebarPanelOption(page, "review");
  const sidebar = page.locator(".sidebar");
  const releaseChecklist = sidebar.getByRole("region", { name: "Release readiness checklist" });
  await expect(releaseChecklist).toContainText("Release state");
  await expect(releaseChecklist).toContainText("need review");
  await releaseChecklist.getByRole("button", { name: "Prepare release metadata" }).click();
  await expect.poll(() => editorText(page)).toContain("releaseTarget:");

  await sidebar.getByLabel("Status").selectOption("approved");
  await sidebar.getByLabel("Version").fill("2.0.0");
  await sidebar.getByLabel("Version").press("Tab");
  await sidebar.getByLabel("Approved by").fill("QA Lead");
  await sidebar.getByLabel("Approved by").press("Tab");
  await sidebar.getByRole("button", { name: "Set approval time" }).click();

  await expect.poll(() => editorText(page)).toContain("status: approved");
  await expect.poll(() => editorText(page)).toContain("version: 2.0.0");
  await expect.poll(() => editorText(page)).toContain("approvedBy: QA Lead");
  await expect.poll(() => editorText(page)).toContain("approvedAt:");
  await expect(page.getByRole("status", { name: "Release status approved" })).toBeVisible();
  await sidebar.getByLabel("Change note").fill("Approved release package for v2.0.0.");
  await sidebar.getByRole("button", { name: "Add change note" }).click();
  await expect.poll(() => editorText(page)).toContain("Approved release package for v2.0.0.");
  await expect(releaseChecklist).toContainText("6 complete, 0 missing, 0 need review");
  await releaseChecklist.getByRole("button", { name: "Insert release audit" }).click();
  await expect.poll(() => editorText(page)).toContain("## Release Readiness Audit");
  await expect.poll(() => editorText(page)).toContain("| Approval audit | complete | QA Lead");
  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("version: 2.0.0");
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("status: approved");
  await setMockFileText(page, "/workspace/market.md", await editorText(page));

  await selectSidebarPanelOption(page, "versioning");
  await expect(sidebar).toContainText("main | clean");
  await expect(sidebar).toContainText("Mock diff for browser workflow");

  await sidebar.getByRole("button", { name: "Create snapshot" }).click();
  await expect(page.locator(".status-bar")).toContainText("Snapshot saved to");
  await expect(sidebar.locator(".snapshot-row").filter({ hasText: "manual" })).toBeVisible();

  await page.locator(".cm-content").click();
  await moveEditorCursorToEnd(page);
  await page.keyboard.insertText("\n\nPost-snapshot change.");
  await expect.poll(() => editorText(page)).toContain("Post-snapshot change.");

  await sidebar.locator(".snapshot-row").filter({ hasText: "manual" }).getByRole("button", { name: "Restore snapshot" }).click();
  await expect.poll(() => editorText(page)).not.toContain("Post-snapshot change.");
  await expect.poll(() => editorText(page)).toContain("approvedBy: QA Lead");

  await sidebar.getByLabel("Release tag").fill("v2.0.0");
  await sidebar.getByRole("button", { name: "Tag release" }).click();
  await expect(page.locator(".status-bar")).toContainText("Tagged release v2.0.0");
});

test("guides Git-free users through snapshot-first versioning", async ({ page }) => {
  await setMockFileText(page, "/workspace/no-git/board-brief.md", "# Board Brief\n\nGit-free draft.");
  await queueDialogSelection(page, "/workspace/no-git/board-brief.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await selectSidebarPanelOption(page, "versioning");

  const sidebar = page.locator(".sidebar");
  await expect(sidebar.getByLabel("Git-free versioning guidance")).toContainText("Snapshot-first document history");
  await expect(sidebar.getByLabel("Git-free versioning guidance")).toContainText("Git-free recovery in private app data");
  await expect(sidebar.getByRole("button", { name: "Commit document" })).toHaveCount(0);
  await expect(sidebar.getByRole("button", { name: "Tag release" })).toHaveCount(0);
  await expect(sidebar).not.toContainText("Mock diff for browser workflow");

  await sidebar.getByLabel("Versioning snapshot storage").selectOption("project-local");
  await expect(sidebar.getByLabel("Git-free versioning guidance")).toContainText("Project-local snapshots travel with the folder");
  await sidebar.getByRole("button", { name: "Create recovery snapshot" }).click();
  await expect(page.locator(".status-bar")).toContainText("Snapshot saved to");
  await expect(sidebar.locator(".snapshot-row").filter({ hasText: "recovery" })).toBeVisible();
  await expect(sidebar.locator(".snapshot-row").filter({ hasText: ".neditor/snapshots" })).toBeVisible();
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
  await openSettingsSection(page, "files");
  await expect(page.getByLabel("Recent files").getByRole("button", { name: "/workspace/rename-source.md" })).toHaveCount(0);
  await expect(page.getByLabel("Recent files").getByRole("button", { name: "/workspace/rename-target.md" })).toBeVisible();
  await expect(page.getByLabel("Recently closed documents").getByRole("button", { name: "/workspace/rename-source.md" })).toHaveCount(0);
  await expect(page.getByLabel("Recently closed documents").getByRole("button", { name: "/workspace/rename-target.md" })).toBeVisible();

  await queueDialogSelection(page, "/workspace/delete-after-close.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await page.locator(".document-tabs .tab.active").getByLabel("Close document").click();
  await deleteMockFile(page, "/workspace/delete-after-close.md");
  await openSettingsSection(page, "files");
  await expect(page.getByLabel("Recently closed documents").getByRole("button", { name: "/workspace/delete-after-close.md" })).toBeVisible();
  await page.getByLabel("Recently closed documents").getByRole("button", { name: "/workspace/delete-after-close.md" }).click();
  await expect(page.getByLabel("Recently closed documents").getByRole("button", { name: "/workspace/delete-after-close.md" })).toHaveCount(0);
  await expect(page.getByText("Removed missing recent file delete-after-close.md")).toBeVisible();
});

test("reorders open document tabs and restores the chosen order", async ({ page }) => {
  await setMockFileText(page, "/workspace/order-a.md", "# Order A\n\nFirst ordered body.");
  await setMockFileText(page, "/workspace/order-b.md", "# Order B\n\nSecond ordered body.");

  await queueDialogSelection(page, "/workspace/order-a.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await queueDialogSelection(page, "/workspace/order-b.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  const workspaceTabs = page.locator('.tab-group[aria-label="Workspace tabs"] .tab');
  await expect(workspaceTabs).toHaveCount(2);
  await expect(workspaceTabs.nth(0)).toContainText(/Order A|order-a\.md/);
  await expect(workspaceTabs.nth(1)).toContainText(/Order B|order-b\.md/);

  await workspaceTabs.nth(1).getByLabel("Move tab left").click();
  await expect(workspaceTabs.nth(0)).toContainText(/Order B|order-b\.md/);
  await expect(workspaceTabs.nth(1)).toContainText(/Order A|order-a\.md/);
  await expect(page.locator(".status-bar")).toContainText("Moved Order B tab before target");

  await page.reload();
  const restoredWorkspaceTabs = page.locator('.tab-group[aria-label="Workspace tabs"] .tab');
  await expect(restoredWorkspaceTabs).toHaveCount(2);
  await expect(restoredWorkspaceTabs.nth(0)).toContainText(/Order B|order-b\.md/);
  await expect(restoredWorkspaceTabs.nth(1)).toContainText(/Order A|order-a\.md/);
  await expect(page.locator(".document-tabs .tab.active")).toContainText(/Order B|order-b\.md/);
});

test("reopens recent folders and prunes moved workspace paths", async ({ page }) => {
  await setMockFileText(page, "/client/project-a.md", "# Project A\n\nClient workspace body.");
  await setMockFileText(page, "/workspace/move-source.md", "# Move Source\n\nMove source body.");

  await queueDialogSelection(page, "/client");
  await page.getByRole("button", { name: "Open Folder", exact: true }).click();
  await selectSidebarPanelOption(page, "files");
  await expect(page.getByText("/client")).toBeVisible();
  await expect(page.getByRole("button", { name: /project-a\.md/ })).toBeVisible();

  await openSettingsSection(page, "files");
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

  await openSettingsSection(page, "files");
  const recentlyClosed = page.getByLabel("Recently closed documents");
  await expect(recentlyClosed.getByRole("button", { name: "/workspace/move-source.md" })).toBeVisible();
  await recentlyClosed.getByRole("button", { name: "/workspace/move-source.md" }).click();
  await expect(recentlyClosed.getByRole("button", { name: "/workspace/move-source.md" })).toHaveCount(0);
  await expect(page.getByText("Removed missing recent file move-source.md")).toBeVisible();

  await selectSidebarPanelOption(page, "files");
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
  await selectSidebarPanelOption(page, "settings");
  await page.locator(".cm-scroller").evaluate((element) => {
    element.scrollTop = element.scrollHeight;
    element.dispatchEvent(new Event("scroll", { bubbles: true }));
  });
  await expect.poll(() => page.locator(".cm-scroller").evaluate((element) => element.scrollTop)).toBeGreaterThan(20);
  await page.getByRole("button", { name: "Save Workspace" }).click();

  await page.reload();

  await expect(page.getByLabel("View mode")).toHaveValue("review");
  await expect(page.getByLabel("Sidebar panel")).toHaveValue("review");
  await expect(page.locator(".document-tabs .tab")).toHaveCount(2);
  await expect(page.getByLabel("Pinned tabs").getByRole("button", { name: /Pinned Brief|pinned-brief\.md/ })).toBeVisible();
  await expect(page.locator(".document-tabs .tab.active")).toContainText("Field Notes");
  await expect.poll(() => page.locator(".cm-scroller").evaluate((element) => element.scrollTop)).toBeGreaterThan(20);
  await expect.poll(() => editorText(page)).toContain("Observation 70");
  await expect.poll(() => page.locator(".preview-pane").evaluate((element) => element.scrollTop)).toBeGreaterThan(20);
  await openSettingsSection(page, "files");
  await expect(page.getByLabel("Recent files").getByRole("button", { name: "/workspace/field-notes.md" })).toBeVisible();

  await selectSidebarPanelOption(page, "files");
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
  await looseTab.evaluate((source) => {
    const target = document.querySelector('[aria-label="Board Pack tabs"]');
    if (!target) throw new Error("Board Pack tab group missing");
    const dataTransfer = new DataTransfer();
    source.dispatchEvent(new DragEvent("dragstart", { bubbles: true, cancelable: true, dataTransfer }));
    target.dispatchEvent(new DragEvent("dragover", { bubbles: true, cancelable: true, dataTransfer }));
    target.dispatchEvent(new DragEvent("drop", { bubbles: true, cancelable: true, dataTransfer }));
    source.dispatchEvent(new DragEvent("dragend", { bubbles: true, cancelable: true, dataTransfer }));
  });
  await expect(boardPack).toContainText("3");
  await expect(boardPack.getByRole("button", { name: /Loose Note/ })).toBeVisible();
  await expect.poll(() => editorText(page)).toContain("documentSet: Board Pack");

  await page.getByRole("button", { name: "Save", exact: true }).click();
  await expect.poll(() => mockFileText(page, "/workspace/loose-note.md")).toContain("documentSet: Board Pack");

  const documentSetManager = page.getByLabel("Document set manager");
  await expect(documentSetManager).toContainText("1 open sets");
  await documentSetManager.getByLabel("Rename active document set").fill("Board Packet");
  await documentSetManager.getByRole("button", { name: "Rename all open set tabs" }).click();
  const boardPacket = page.getByLabel("Board Packet tabs");
  await expect(boardPacket).toContainText("3");
  await expect(page.getByLabel("Board Pack tabs")).toHaveCount(0);
  await expect.poll(() => editorText(page)).toContain("documentSet: Board Packet");
  await documentSetManager.getByRole("button", { name: "Insert manifest" }).click();
  await expect.poll(() => editorText(page)).toContain("## Document Set Manifest: Board Packet");
  await expect.poll(() => editorText(page)).toContain("Risk Register");
  await expect.poll(() => editorText(page)).toContain("Review Handoff");

  await documentSetManager.getByRole("button", { name: "Remove active" }).click();
  await expect(boardPacket).toContainText("2");
  await expect.poll(() => editorText(page)).not.toContain("documentSet:");

  await boardPacket.getByLabel("Close tab group").click();
  await expect(page.getByLabel("Board Packet tabs")).toHaveCount(0);
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
  await openSettingsSection(page, "files");
  await expect(page.getByLabel("Recent files").getByRole("button", { name: "/workspace/missing-brief.md" })).toHaveCount(0);
});

test("blocks stale saves and preserves local conflict copies", async ({ page }) => {
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  await selectSidebarPanelOption(page, "review");
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
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("External disk edit.");
  await expect.poll(() => mockFileText(page, "/workspace/market.md")).toContain("status: approved");
  await expect.poll(() => mockFileText(page, "/workspace/market local copy.md")).toContain("status: in-review");
  await selectSidebarPanelOption(page, "files");
  await expect.poll(() => activeFileRowText(page)).toContain("market local copy.md");
});

test("merges external conflict text back into the original file", async ({ page }) => {
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  await selectSidebarPanelOption(page, "review");
  await page.locator(".sidebar").getByLabel("Status").selectOption("in-review");
  await setMockFileText(page, "/workspace/market.md", externalApprovedDocument());

  await page.getByRole("button", { name: "Save", exact: true }).click();
  await page.locator(".status-bar .conflict-actions").getByRole("button", { name: "Compare" }).click();

  const conflictDialog = page.getByRole("dialog", { name: "External file conflict" });
  const mergeComposition = conflictDialog.getByRole("region", { name: "Merge composition" });
  await expect(conflictDialog.getByRole("group", { name: /Local changed line \d+: status: in-review/ })).toBeVisible();
  await expect(conflictDialog.getByRole("group", { name: /External changed line \d+: status: approved/ })).toBeVisible();
  await expect(mergeComposition).toContainText("0 selected lines");
  await conflictDialog.getByRole("button", { name: "Add external line 3 to merge" }).click();
  await conflictDialog.getByRole("button", { name: "Add external line 8 to merge" }).click();
  await expect(mergeComposition).toContainText("2 selected lines");
  await expect(conflictDialog.getByLabel("Merged result")).toHaveValue("status: approved\nExternal disk edit.");
  await mergeComposition.getByRole("button", { name: "Move external line 8 up" }).click();
  await expect(conflictDialog.getByLabel("Merged result")).toHaveValue("External disk edit.\nstatus: approved");
  await mergeComposition.getByRole("button", { name: "Remove external line 8" }).click();
  await expect(conflictDialog.getByLabel("Merged result")).toHaveValue("status: approved");
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

  await selectSidebarPanelOption(page, "review");
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

  await selectSidebarPanelOption(page, "review");
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

test("detects external changes when switching back to an inactive clean tab", async ({ page }) => {
  await setMockFileText(page, "/workspace/watch-a.md", "# Watch A\n\nOriginal A body.");
  await setMockFileText(page, "/workspace/watch-b.md", "# Watch B\n\nOriginal B body.");

  await queueDialogSelection(page, "/workspace/watch-a.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await queueDialogSelection(page, "/workspace/watch-b.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await expect(page.locator(".document-tabs .tab.active")).toContainText(/Watch B|watch-b\.md/);
  await expect.poll(() => editorText(page)).toContain("Original B body.");
  await expect(page.locator(".status-bar")).toContainText("Native watch: 1 path");

  await setMockFileText(page, "/workspace/watch-a.md", "# Watch A\n\nExternal A update while inactive.");
  await emitMockFileWatch(page, "/workspace/watch-a.md");
  await expect.poll(() => editorText(page)).toContain("Original B body.");
  await expect.poll(() => editorText(page)).not.toContain("External A update while inactive.");

  await page.locator(".document-tabs .tab").filter({ hasText: /Watch A|watch-a\.md/ }).getByRole("button", { name: /Watch A|watch-a\.md/ }).click();
  await expect(page.locator(".document-tabs .tab.active")).toContainText(/Watch A|watch-a\.md/);
  await expect.poll(() => editorText(page)).toContain("External A update while inactive.");
  await expect(page.locator(".status-bar")).toContainText("Reloaded external changes");
  await expect(page.locator(".document-tabs .tab.active")).not.toContainText("*");
});

test("opens a root-file conflict when watcher events arrive during local edits", async ({ page }) => {
  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await expect(page.locator(".status-bar")).toContainText("Native watch: 1 path");

  await selectSidebarPanelOption(page, "review");
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
  await selectSidebarPanelOption(page, "references");
  const includeGraph = page.getByRole("region", { name: "Include graph" });
  await expect(includeGraph).toContainText("Depth 0");
  await expect(includeGraph).toContainText("market.md");
  await expect(includeGraph).toContainText("chapters/risk.md");

  await setMockFileText(page, "/workspace/chapters/risk.md", "## Risk Notes\n\nUpdated included risk note.");
  await emitMockFileWatch(page, "/workspace/chapters/risk.md");

  await expect(preview).toContainText("Updated included risk note.");
  await expect.poll(() => editorText(page)).toContain("!include chapters/risk.md");
  await expect.poll(() => editorText(page)).not.toContain("Updated included risk note.");
});

test("navigates include graph entries from references and commands", async ({ page }) => {
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

  await page.getByRole("button", { name: "Commands" }).click();
  await page.getByPlaceholder("Search commands, headings, citations, glossary, index terms").fill("Open include chapters/risk.md");
  await page.getByRole("button", { name: /Open include chapters\/risk\.md.*Include depth 0/ }).click();
  await expect.poll(() => editorText(page)).toContain("Original included risk note.");

  await queueDialogSelection(page, "/workspace/market.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();
  await selectSidebarPanelOption(page, "references");

  const includeGraph = page.getByRole("region", { name: "Include graph" });
  await expect(includeGraph).toContainText("market.md");
  await expect(includeGraph).toContainText("chapters/risk.md");
  await includeGraph.getByRole("button", { name: "Go to include directive for /workspace/chapters/risk.md" }).click();
  await expect.poll(() => editorText(page)).toContain("!include chapters/risk.md");

  await includeGraph.getByRole("button", { name: "Open include /workspace/chapters/risk.md" }).click();
  await expect.poll(() => editorText(page)).toContain("Original included risk note.");
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

  await selectSidebarPanelOption(page, "review");
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
  await selectSidebarPanelOption(page, "tables");
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

  const tableGrid = page.getByRole("group", { name: "Table editor grid" });
  const markdownPreview = page.getByLabel("Markdown preview");
  await expect(tableGrid.getByRole("group", { name: "Sort controls for column B" })).toBeVisible();
  await expect(tableGrid.getByRole("group", { name: "Row 1 controls" })).toBeVisible();
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

  await selectSidebarPanelOption(page, "tables");
  const markdownPreview = page.getByLabel("Markdown preview");
  await expect(page.getByLabel("Value, row 1, column B")).toHaveValue("125000");

  await page.getByLabel("Column B format").selectOption("currency");
  await expect(page.getByLabel("Total for Value, column B")).toHaveText("$125000");

  await page.getByRole("button", { name: "Add row" }).click();
  await page.getByLabel("Item, row 2, column A").fill("Cost");
  await page.getByLabel("Value, row 2, column B").fill("74000");
  await expect(markdownPreview).toHaveValue(/Cost\s+\|\s+\$74000/);

  await page.getByRole("button", { name: "Remove row 2" }).click();
  await expect(markdownPreview).not.toHaveValue(/Cost/);

  await page.getByRole("button", { name: "Add column" }).click();
  await page.getByLabel("Column C header").fill("Margin");
  await expect(page.getByRole("group", { name: "Move controls for column C" })).toBeVisible();
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
  const dialog = page.getByRole("dialog", { name: "AI paste cleanup" });
  await setMockClipboardText(page, "<p><strong>Assistant:</strong> Revenue grew 24%.</p>", "text/html");
  await dialog.getByRole("button", { name: "Load clipboard" }).click();
  await expect(dialog.getByRole("textbox", { name: "Original" })).toHaveValue(/<strong>Assistant/);
  await expect(page.locator(".status-bar")).toContainText("Loaded rich clipboard text for AI cleanup");
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

test("toggles AI review state and clears provenance readiness warnings", async ({ page }) => {
  await page.locator(".cm-content").click();
  await page.keyboard.press(process.platform === "darwin" ? "Meta+A" : "Control+A");
  await page.keyboard.insertText(
    [
      "---",
      "title: AI Review Workflow",
      "status: approved",
      "approvedBy: QA",
      "approvedAt: 2026-05-20T10:00:00Z",
      "---",
      "",
      "# AI Review Workflow",
      "",
      "<!-- ai-assisted: status=needs-review | source=OpenAI | promptSummary=Drafted risk language -->",
      "## AI Draft",
      "Drafted AI content.",
      "",
      "```ai-source",
      "provider: OpenAI",
      "model: ChatGPT",
      "date: 2026-05-20",
      "promptSummary: Drafted risk language",
      "reviewedBy: ",
      "reviewedAt: ",
      "status: needs-review",
      "```",
    ].join("\n"),
  );
  await expect.poll(() => editorText(page)).toContain("status: needs-review");

  await selectSidebarPanelOption(page, "exports");
  await page.getByRole("button", { name: "Prepare for export" }).click();
  await expect(page.locator("article.readiness").getByText("Needs attention", { exact: true })).toBeVisible();
  await expect(page.getByRole("list", { name: "Export readiness diagnostics" })).toContainText(
    "Document has AI-assisted sections that are not human-reviewed.",
  );

  await selectSidebarPanelOption(page, "review");
  await expect(page.locator(".sidebar")).toContainText("2 AI review pending");
  const sourceReview = page.locator(".sidebar article").filter({ hasText: "OpenAI / ChatGPT" });
  await sourceReview.getByLabel("Human reviewed").check();
  const sectionReview = page.locator(".sidebar article").filter({ hasText: "AI Draft" });
  await sectionReview.getByLabel("Human reviewed").check();

  await expect.poll(() => editorText(page)).toContain("status=human-reviewed");
  await expect.poll(() => editorText(page)).toContain("status: human-reviewed");
  await expect.poll(() => editorText(page)).toContain("reviewedBy: local");
  await expect(page.locator(".sidebar")).toContainText("0 AI review pending");
  await expect(page.locator(".sidebar")).toContainText("2 AI reviewed");

  await selectSidebarPanelOption(page, "exports");
  await page.getByRole("button", { name: "Prepare for export" }).click();
  await expect(page.locator("article.readiness").getByText("Ready", { exact: true })).toBeVisible();
  await expect(page.locator("article.readiness").getByText("0 errors, 0 warnings, 0 info", { exact: true })).toBeVisible();
  await expect(page.locator(".sidebar").getByText('"includeProvenance": true')).toBeVisible();
});

test("manages review comments and change notes from the review panel", async ({ page }) => {
  await setMockFileText(
    page,
    "/workspace/review-workflow.md",
    [
      "---",
      "title: Review Workflow",
      "status: draft",
      "---",
      "",
      "# Review Workflow",
      "",
      "Ready for reviewer notes.",
    ].join("\n"),
  );
  await queueDialogSelection(page, "/workspace/review-workflow.md");
  await page.getByRole("button", { name: "Open", exact: true }).click();

  await selectSidebarPanelOption(page, "review");
  await expect(page.locator(".snapshot-row").filter({ hasText: "draft | 0 unresolved | 0 resolved" })).toBeVisible();

  await page.getByLabel("New comment").fill("Confirm finance source.");
  await page.getByRole("button", { name: "Add comment" }).click();
  await expect.poll(() => editorText(page)).toContain("Confirm finance source.");
  await expect(page.locator(".snapshot-row").filter({ hasText: "Confirm finance source." })).toContainText("unresolved");
  await expect(page.locator(".snapshot-row").filter({ hasText: "draft | 1 unresolved | 0 resolved" })).toBeVisible();

  await page.getByLabel("Change note").fill("Updated finance assumption for review.");
  await page.getByRole("button", { name: "Add change note" }).click();
  await expect.poll(() => editorText(page)).toContain("Updated finance assumption for review.");
  await expect(page.locator(".snapshot-row").filter({ hasText: "1 change notes" })).toBeVisible();
  await expect(page.locator(".snapshot-row").filter({ hasText: "Updated finance assumption for review." })).toContainText("local");

  const commentRow = page.locator(".snapshot-row").filter({ hasText: "Confirm finance source." });
  await commentRow.getByRole("button", { name: "Resolve" }).click();
  await expect.poll(() => editorText(page)).toContain("<!-- comment: resolved | author: local");
  await expect(commentRow).toContainText("resolved");
  await expect(page.locator(".snapshot-row").filter({ hasText: "draft | 0 unresolved | 1 resolved" })).toBeVisible();
});

test("applies AI paste quote appendix and section merge modes", async ({ page }) => {
  await insertCleanedAiPaste(page, "Assistant: Quote this.", "quote");
  await expect.poll(() => editorText(page)).toContain("> Cleaned AI output");
  await expect.poll(() => editorText(page)).toContain("> Assistant: Quote this.");

  await insertCleanedAiPaste(page, "Assistant: Put this in an appendix.", "appendix");
  await expect.poll(() => editorText(page)).toContain("## AI Draft Appendix");
  await expect.poll(() => editorText(page)).toContain("Assistant: Put this in an appendix.");

  await page.locator(".cm-content").click();
  await moveEditorCursorToEnd(page);
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
  await page.locator(".cm-content").click();
  await page.keyboard.press(process.platform === "darwin" ? "Meta+A" : "Control+A");
  await page.keyboard.insertText(
    [
      "---",
      "title: Export Preview",
      "version: 1.0.0",
      "status: approved",
      "approvedBy: QA",
      "approvedAt: 2026-05-21",
      "---",
      "",
      "# Export Preview",
      "",
      "```d2",
      "a -> b",
      "```",
    ].join("\n"),
  );
  await expect.poll(() => editorText(page)).toContain("a -> b");

  await selectSidebarPanelOption(page, "exports");
  await page.getByLabel("View mode").selectOption("export");
  const targetSelect = page.getByLabel("Target");
  const exportPreview = page.getByRole("region", { name: "Export preview summary" });
  await expect(exportPreview).toContainText("HTML export preview");
  await queueDialogSelection(page, "/exports/market.html");
  await page.getByRole("button", { name: "Export HTML" }).click();
  const exportResult = page.getByRole("region", { name: "Export result" });
  await expect(targetSelect).toHaveValue("html");
  await expect(exportResult).toContainText("Output: /exports/market.html");
  await expect(exportResult).toContainText("Manifest: /exports/market.html.manifest.json");
  await expect(exportResult).toContainText("Mock html export wrote /exports/market.html");
  await expect(page.locator(".status-bar")).toContainText("Exported /exports/market.html with manifest /exports/market.html.manifest.json");

  await targetSelect.selectOption("pptx");
  await expect(exportPreview).toContainText("PPTX export preview");
  await expect(exportPreview).toContainText("readiness not run");
  await expect(exportPreview).toContainText("1 transform artifacts");
  const transformPreview = page.getByRole("region", { name: "Transform artifact preview" });
  await expect(transformPreview).toContainText("d2");
  await expect(transformPreview).toContainText("svg via external");
  await expect(transformPreview).toContainText("Cache d2:");
  await transformPreview.getByRole("button", { name: "Go to source" }).click();
  await expect(page.locator(".cm-line").filter({ hasText: "```d2" })).toBeVisible();
  await page.getByLabel("View mode").selectOption("export");

  await page.getByRole("button", { name: "Prepare for export" }).click();

  await expect(page.locator("article.readiness").getByText("Ready", { exact: true })).toBeVisible();
  await expect(page.locator("article.readiness").getByText("0 errors, 0 warnings, 1 info", { exact: true })).toBeVisible();
  await expect(page.locator(".sidebar").getByText('"export_target": "pptx"')).toBeVisible();
  await expect(exportPreview).toContainText("ready");

  await queueDialogSelection(page, "/exports/market.pptx");
  await page.getByRole("button", { name: "Export document" }).click();
  await expect(exportResult).toContainText("Output: /exports/market.pptx");
  await expect(exportResult).toContainText("Manifest: /exports/market.pptx.manifest.json");
  await expect(exportResult).toContainText("Mock pptx export wrote /exports/market.pptx");
  await expect(page.locator(".status-bar")).toContainText("Exported /exports/market.pptx with manifest /exports/market.pptx.manifest.json");

  await queueDialogSelection(page, "/exports/fail.pptx");
  await page.getByRole("button", { name: "Export document" }).click();
  await expect(exportResult).toContainText("Mock export writer failed for /exports/fail.pptx");
  await expect(exportResult).toContainText("Review export readiness diagnostics and target settings before retrying.");
  await expect(page.locator(".status-bar")).toContainText("Export failed: Mock export writer failed for /exports/fail.pptx");

  await page.getByLabel("View mode").selectOption("split");
  await page.locator(".cm-content").click();
  await page.keyboard.press(process.platform === "darwin" ? "Meta+A" : "Control+A");
  await page.keyboard.insertText(["---", "title: Blocked Export", "version: 1.0.0", "status: in-review", "---", "", "# Blocked Export"].join("\n"));
  await expect.poll(() => editorText(page)).toContain("status: in-review");
  await page.getByRole("button", { name: "Export document" }).click();
  await expect(page.locator("article.readiness").getByText("Needs attention", { exact: true })).toBeVisible();
  await expect(page.getByRole("list", { name: "Export readiness diagnostics" })).toContainText("PPTX export requires approved metadata before writing.");
  await expect(page.getByRole("list", { name: "Export readiness diagnostics" })).toContainText("target:pptx");
  await expect(page.locator(".status-bar")).toContainText("1 errors block export");
});

test("saves and reapplies reusable export profiles", async ({ page }) => {
  await openSettingsSection(page, "exports");
  await page.getByLabel("Brand name").fill("Acme Board");
  await page.getByLabel("Brand color").fill("#006699");
  await page.getByLabel("Footer template").fill("Confidential");
  await page.getByLabel("Citation style").selectOption("ieee");

  await selectSidebarPanelOption(page, "exports");
  const targetSelect = page.getByLabel("Target");
  const layoutSelect = page.getByLabel("Layout preset");
  await targetSelect.selectOption("pdf");
  await layoutSelect.selectOption("compact");
  await page.getByLabel("Export manifest").uncheck();
  await page.getByLabel("Cover page").uncheck();
  await page.getByLabel("Page numbers").uncheck();
  await page.getByLabel("Profile name").fill("Client PDF");
  await page.getByRole("button", { name: "Save profile" }).click();

  await expect(page.locator(".status-bar")).toContainText('Saved export profile "Client PDF"');
  await expect(page.getByLabel("Saved profile")).toContainText("Client PDF");

  await targetSelect.selectOption("html");
  await layoutSelect.selectOption("presentation");
  await page.getByLabel("Export manifest").check();
  await page.getByLabel("Cover page").check();
  await page.getByLabel("Page numbers").check();

  await page.getByLabel("Saved profile").selectOption({ label: "Client PDF" });
  await expect(targetSelect).toHaveValue("pdf");
  await expect(layoutSelect).toHaveValue("compact");
  await expect(page.getByLabel("Export manifest")).not.toBeChecked();
  await expect(page.getByLabel("Cover page")).not.toBeChecked();
  await expect(page.getByLabel("Page numbers")).not.toBeChecked();
  await expect(page.locator(".sidebar-hint").filter({ hasText: "PDF / compact / Acme Board" })).toBeVisible();

  await page.reload();
  await expect(page.getByRole("heading", { name: "Market Entry Report" })).toBeVisible();
  await selectSidebarPanelOption(page, "exports");
  await expect(page.getByLabel("Saved profile")).toContainText("Client PDF");
  await page.getByLabel("Saved profile").selectOption({ label: "Client PDF" });
  await expect(page.getByLabel("Target")).toHaveValue("pdf");
  await expect(page.getByLabel("Layout preset")).toHaveValue("compact");
});

test("publishes and hands off extended export targets", async ({ page }) => {
  await page.locator(".cm-content").click();
  await page.keyboard.press(process.platform === "darwin" ? "Meta+A" : "Control+A");
  await page.keyboard.insertText(
    [
      "---",
      "title: Publishing Handoff",
      "version: 1.0.0",
      "status: approved",
      "approvedBy: QA",
      "approvedAt: 2026-05-21",
      "---",
      "",
      "# Publishing Handoff",
      "",
      "A package-ready update with a table, link, and equation.",
      "",
      "| Metric | Value |",
      "| --- | ---: |",
      "| Revenue | 42 |",
      "",
      "See [the appendix](#appendix).",
      "",
      "$$x = y + z$$",
    ].join("\n"),
  );
  await expect.poll(() => editorText(page)).toContain("Publishing Handoff");

  await selectSidebarPanelOption(page, "exports");
  await page.getByLabel("View mode").selectOption("export");
  const targetSelect = page.getByLabel("Target");
  const exportPreview = page.getByRole("region", { name: "Export preview summary" });
  const exportResult = page.getByRole("region", { name: "Export result" });

  const targets = [
    { value: "blog", label: "Blog package", path: "/exports/publishing.blog.zip" },
    { value: "substack", label: "Substack package", path: "/exports/publishing.substack.zip" },
    { value: "latex", label: "LaTeX", path: "/exports/publishing.tex" },
    { value: "google-docs", label: "Google Docs package", path: "/exports/publishing.google-docs.zip" },
    { value: "epub", label: "EPUB ebook", path: "/exports/publishing.epub" },
  ];

  for (const target of targets) {
    await expect(targetSelect.locator(`option[value="${target.value}"]`)).toHaveText(target.label);
    await targetSelect.selectOption(target.value);
    await expect(targetSelect).toHaveValue(target.value);
    await expect(exportPreview).toContainText(`${target.value.toUpperCase()} export preview`);
    if (target.value === "blog") {
      await page.getByRole("region", { name: "Public export metadata options" }).getByLabel("Description").fill("");
      const checklist = page.getByRole("region", { name: "Distribution metadata checklist" });
      await expect(checklist).toContainText("Release approval");
      await expect(checklist).toContainText("Publishing preview");
      await expect(checklist).toContainText("missing");
      await checklist.getByRole("button", { name: "Add suggested metadata" }).click();
      await expect.poll(() => editorText(page)).toContain("releaseTarget: blog package");
      await expect.poll(() => editorText(page)).toContain("description: TODO public summary");
      await expect.poll(() => editorText(page)).toContain('"todo"');
      await expect(checklist).toContainText("Tags and keywords");
    }

    await page.getByRole("button", { name: "Prepare for export" }).click();
    await expect(page.locator("article.readiness").getByText("Ready", { exact: true })).toBeVisible();
    await expect(page.locator(".sidebar").getByText(`"export_target": "${target.value}"`)).toBeVisible();
    await expect(page.locator(".sidebar").getByText('"includeManifest": true')).toBeVisible();
    await expect(exportPreview).toContainText("ready");

    await queueDialogSelection(page, target.path);
    await page
      .getByRole("button", { name: target.value === "epub" ? "Export EPUB" : "Export document" })
      .click();
    await expect(exportResult).toContainText(`Output: ${target.path}`);
    await expect(exportResult).toContainText(`Manifest: ${target.path}.manifest.json`);
    await expect(exportResult).toContainText(`Mock ${target.value} export wrote ${target.path}`);
    await expect(page.locator(".status-bar")).toContainText(`Exported ${target.path} with manifest ${target.path}.manifest.json`);
  }
});
