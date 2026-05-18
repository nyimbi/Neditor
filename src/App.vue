<template>
  <div class="app-shell" :data-theme="store.theme">
    <header class="titlebar">
      <section class="document-tabs" aria-label="Open documents">
        <div
          v-for="document in store.documents"
          :key="document.id"
          class="tab"
          :class="{ active: document.id === store.activeId }"
        >
          <button class="tab-main" type="button" @click="activate(document.id)">
            <span>{{ document.dirty ? "*" : "" }}{{ document.title }}</span>
          </button>
          <button
            class="tab-close"
            type="button"
            :aria-label="document.pinned ? 'Unpin document' : 'Pin document'"
            @click="store.togglePin(document.id)"
          >
            {{ document.pinned ? "!" : "^" }}
          </button>
          <button class="tab-close" type="button" aria-label="Close document" @click="store.closeDocument(document.id)">x</button>
        </div>
      </section>

      <section class="window-meta" aria-label="Document status">
        <span>{{ active.compile?.semantic.status || "draft" }}</span>
        <span v-if="store.gitStatus?.inside_repo">{{ store.gitStatus.branch || "detached" }}{{ store.gitStatus.dirty ? " dirty" : " clean" }}</span>
      </section>
    </header>

    <nav class="command-bar" aria-label="Main commands">
      <button type="button" @click="store.newDocument">New</button>
      <button type="button" @click="openDocument">Open</button>
      <button type="button" @click="openFolder">Open Folder</button>
      <button type="button" @click="saveDocument">Save</button>
      <button type="button" @click="saveDocumentAs">Save As</button>
      <button type="button" @click="store.revertActive">Revert</button>
      <button type="button" @click="renameDocument">Rename</button>
      <button type="button" @click="duplicateDocument">Duplicate</button>
      <button type="button" @click="store.revealActive">Reveal</button>
      <button type="button" @click="store.snapshotActive()">Snapshot</button>
      <button type="button" @click="exportDocument">Export</button>
      <button type="button" @click="aiPasteOpen = true">AI Paste</button>
      <button type="button" @click="commandPaletteOpen = true">Commands</button>
      <span class="divider"></span>
      <button type="button" title="Bold" @click="wrapSelection('**')"><strong>B</strong></button>
      <button type="button" title="Italic" @click="wrapSelection('*')"><em>I</em></button>
      <button type="button" title="Code" @click="wrapSelection('`')">Code</button>
      <button type="button" title="Heading" @click="insertAtLineStart('## ')">H2</button>
      <button type="button" title="Link" @click="wrapSelection('[', '](https://)')">Link</button>
      <button type="button" title="Table" @click="insertBlock(tableSnippet)">Table</button>
      <button type="button" title="Calculation" @click="insertBlock(calcSnippet)">Calc</button>
      <button type="button" title="AI source" @click="insertBlock(aiSnippet)">AI</button>
      <span class="divider"></span>
      <select v-model="store.mode" aria-label="View mode">
        <option value="split">Split</option>
        <option value="source">Source</option>
        <option value="preview">Preview</option>
        <option value="focus">Focus</option>
        <option value="export">Export</option>
        <option value="review">Review</option>
        <option value="presentation">Presentation</option>
      </select>
      <select v-model="store.sidebar" aria-label="Sidebar panel">
        <option value="files">Files</option>
        <option value="outline">Outline</option>
        <option value="diagnostics">Diagnostics</option>
        <option value="tables">Tables</option>
        <option value="references">References</option>
        <option value="exports">Exports</option>
        <option value="versioning">Versioning</option>
        <option value="review">Review</option>
        <option value="settings">Settings</option>
      </select>
    </nav>

    <main class="workspace" :class="`mode-${store.mode}`">
      <aside class="sidebar" aria-label="Document workspace">
        <template v-if="store.sidebar === 'files'">
          <h2>Workspace</h2>
          <button type="button" @click="openFolder">Open folder</button>
          <button v-if="store.workspaceRoot" type="button" @click="store.refreshWorkspace">Refresh</button>
          <p v-if="store.workspaceRoot" class="workspace-root">{{ store.workspaceRoot }}</p>
          <p v-else>Open a folder to browse project files.</p>
          <button
            v-for="entry in store.workspaceFiles"
            :key="entry.path"
            class="file-row"
            :class="{ directory: entry.kind === 'directory', active: entry.path === active.path }"
            :style="{ paddingLeft: `${entry.depth * 12 + 8}px` }"
            type="button"
            @click="entry.kind === 'directory' ? undefined : store.openPath(entry.path)"
          >
            <span>{{ entry.kind === "directory" ? ">" : "-" }}</span>
            <span>{{ entry.name }}</span>
          </button>
        </template>

        <template v-else-if="store.sidebar === 'outline'">
          <h2>Outline</h2>
          <button
            v-for="heading in active.compile?.semantic.outline || []"
            :key="`${heading.line}-${heading.anchor}`"
            class="outline-row"
            :style="{ paddingLeft: `${heading.level * 10}px` }"
            type="button"
            @click="goToLine(heading.line)"
          >
            {{ heading.text }}
          </button>
        </template>

        <template v-else-if="store.sidebar === 'diagnostics'">
          <h2>Diagnostics</h2>
          <article
            v-for="diagnostic in active.compile?.diagnostics || []"
            :key="`${diagnostic.severity}-${diagnostic.message}-${diagnostic.line}`"
            class="diagnostic"
            :class="diagnostic.severity"
          >
            <strong>{{ diagnostic.severity }}</strong>
            <p>{{ diagnostic.message }}</p>
            <small v-if="diagnostic.suggestion">{{ diagnostic.suggestion }}</small>
          </article>
        </template>

        <template v-else-if="store.sidebar === 'tables'">
          <h2>Tables</h2>
          <label>
            Table
            <select v-model.number="selectedTableIndex" @change="loadSelectedTable">
              <option v-for="(table, index) in markdownTables" :key="`${table.startLine}-${index}`" :value="index">
                Line {{ table.startLine }} - {{ table.headers.join(", ") }}
              </option>
            </select>
          </label>
          <template v-if="tableDraft && selectedTable">
            <div class="table-actions">
              <button type="button" @click="applyTableDraft">Apply</button>
              <button type="button" @click="addTableRow">Add row</button>
              <button type="button" @click="addTableColumn">Add column</button>
              <button type="button" @click="addTableTotalsRow">Add totals row</button>
            </div>
            <label>
              CSV/TSV paste
              <textarea v-model="tablePasteText" rows="4"></textarea>
            </label>
            <button type="button" @click="replaceTableFromPaste">Replace from paste</button>
            <div class="table-editor-grid" :style="{ gridTemplateColumns: `110px repeat(${tableDraft.headers.length}, minmax(120px, 1fr)) 44px` }">
              <span></span>
              <input
                v-for="(_, columnIndex) in tableDraft.headers"
                :key="`header-${columnIndex}`"
                v-model="tableDraft.headers[columnIndex]"
                aria-label="Column header"
              />
              <span></span>
              <span>Align</span>
              <select v-for="(_, columnIndex) in tableDraft.headers" :key="`align-${columnIndex}`" v-model="tableDraft.alignments[columnIndex]">
                <option value="left">Left</option>
                <option value="center">Center</option>
                <option value="right">Right</option>
              </select>
              <span></span>
              <span>Format</span>
              <select v-for="(_, columnIndex) in tableDraft.headers" :key="`format-${columnIndex}`" v-model="tableDraft.formats[columnIndex]">
                <option value="text">Text</option>
                <option value="number">Number</option>
                <option value="currency">Currency</option>
                <option value="percent">Percent</option>
                <option value="date">Date</option>
              </select>
              <span></span>
              <span>Sort</span>
              <button v-for="(_, columnIndex) in tableDraft.headers" :key="`sort-${columnIndex}`" type="button" @click="sortTableRows(columnIndex)">Sort</button>
              <span></span>
              <template v-for="(row, rowIndex) in tableDraft.rows" :key="`row-${rowIndex}`">
                <button type="button" @click="removeTableRow(rowIndex)">Remove</button>
                <input
                  v-for="(_, columnIndex) in tableDraft.headers"
                  :key="`cell-${rowIndex}-${columnIndex}`"
                  v-model="row[columnIndex]"
                  aria-label="Table cell"
                />
                <span></span>
              </template>
              <span>Totals</span>
              <output v-for="(total, columnIndex) in tableColumnTotals" :key="`total-${columnIndex}`">
                {{ total || "-" }}
              </output>
              <span></span>
              <span>Remove column</span>
              <button v-for="(_, columnIndex) in tableDraft.headers" :key="`remove-col-${columnIndex}`" type="button" @click="removeTableColumn(columnIndex)">
                Remove
              </button>
              <span></span>
            </div>
          </template>
          <p v-else>No Markdown table selected.</p>
        </template>

        <template v-else-if="store.sidebar === 'references'">
          <h2>References</h2>
          <h3>Citations</h3>
          <p v-for="citation in active.compile?.semantic.citation_references || []" :key="`${citation.key}-${citation.locator || ''}`">
            [@{{ citation.key }}<template v-if="citation.locator">, {{ citation.locator }}</template>]
            <small>{{ bibliographyByKey.get(citation.key) || "Missing bibliography entry" }}</small>
          </p>
          <template v-if="active.compile?.semantic.duplicate_bibliography_keys.length">
            <h3>Duplicate keys</h3>
            <p v-for="key in active.compile.semantic.duplicate_bibliography_keys" :key="key" class="error">{{ key }}</p>
          </template>
          <h3>Glossary</h3>
          <dl>
            <template v-for="(definition, term) in active.compile?.semantic.glossary || {}" :key="term">
              <dt>{{ term }}</dt>
              <dd>{{ definition }}</dd>
            </template>
          </dl>
          <h3>Tables</h3>
          <article v-for="table in active.compile?.semantic.table_summaries || []" :key="table.line" class="snapshot-row">
            <p>{{ table.rows }} rows | {{ table.columns.join(", ") }}</p>
            <small v-for="(total, column) in table.numeric_columns" :key="column">{{ column }} total: {{ total }} </small>
          </article>
          <h3>Includes</h3>
          <p v-for="edge in active.compile?.include_graph || []" :key="`${edge.parent}-${edge.child}`">{{ edge.child }}</p>
          <h3>Cross references</h3>
          <p v-for="reference in active.compile?.semantic.cross_references || []" :key="reference.key">
            {{ reference.key }}: {{ reference.resolved ? "resolved" : "missing" }}
          </p>
          <h3>Labels</h3>
          <p v-for="label in active.compile?.semantic.labels || []" :key="label">{{ label }}</p>
        </template>

        <template v-else-if="store.sidebar === 'exports'">
          <h2>Export</h2>
          <label>
            Target
            <select v-model="store.exportTarget">
              <option value="html">HTML</option>
              <option value="pdf">PDF</option>
              <option value="docx">DOCX</option>
              <option value="pptx">PPTX</option>
              <option value="markdown-bundle">Markdown bundle</option>
            </select>
          </label>
          <button type="button" @click="store.prepareForExport">Prepare for export</button>
          <button type="button" @click="exportDocument">Export document</button>
          <article v-if="store.exportReadiness" class="readiness" :class="{ ready: store.exportReadiness.ready }">
            <strong>{{ store.exportReadiness.ready ? "Ready" : "Needs attention" }}</strong>
            <p>{{ store.exportReadiness.error_count }} errors, {{ store.exportReadiness.warning_count }} warnings, {{ store.exportReadiness.info_count }} info</p>
          </article>
          <h3>Manifest</h3>
          <pre>{{ manifestPreview }}</pre>
          <h3>Snapshots</h3>
          <button type="button" @click="store.listSnapshots">Refresh snapshots</button>
          <article v-for="snapshot in store.snapshots" :key="snapshot.snapshot_path" class="snapshot-row">
            <p>{{ snapshot.label || "snapshot" }}</p>
            <small>{{ snapshot.created_at || snapshot.snapshot_path }}</small>
            <button type="button" @click="store.restoreSnapshot(snapshot.snapshot_path)">Restore</button>
          </article>
        </template>

        <template v-else-if="store.sidebar === 'versioning'">
          <h2>Versioning</h2>
          <article v-if="store.gitStatus?.inside_repo" class="snapshot-row">
            <p>{{ store.gitStatus.branch || "detached" }} | {{ store.gitStatus.dirty ? "dirty" : "clean" }}</p>
            <small v-for="line in store.gitStatus.summary" :key="line">{{ line }}</small>
          </article>
          <p v-else>Current document is not inside a Git repository.</p>
          <label>
            Commit message
            <input v-model="store.commitMessage" placeholder="Update document" />
          </label>
          <button type="button" @click="store.commitActive()">Commit document</button>
          <label>
            Release tag
            <input v-model="store.releaseTag" placeholder="v1.0.0" />
          </label>
          <button type="button" @click="store.tagActiveRelease()">Tag release</button>
          <button type="button" @click="store.refreshGitDiff">Refresh diff</button>
          <h3>Diff</h3>
          <pre>{{ store.gitDiffText || "No uncommitted diff." }}</pre>
          <h3>History</h3>
          <article v-for="entry in store.gitHistory" :key="entry.revision" class="snapshot-row">
            <p>{{ entry.subject }}</p>
            <small>{{ entry.revision.slice(0, 12) }} | {{ entry.author }} | {{ entry.date }}</small>
            <button type="button" @click="store.restoreGitRevision(entry.revision)">Restore</button>
          </article>
        </template>

        <template v-else-if="store.sidebar === 'review'">
          <h2>Review</h2>
          <label>
            New comment
            <textarea v-model="reviewCommentText" rows="4" placeholder="Review note"></textarea>
          </label>
          <button type="button" @click="insertReviewComment">Add comment</button>
          <h3>Comments</h3>
          <article v-for="comment in active.compile?.semantic.comments || []" :key="String(comment.line)" class="snapshot-row">
            <p>{{ comment.text }}</p>
            <small>Line {{ comment.line }} | {{ comment.state }} | {{ comment.author || "local" }}{{ comment.created_at ? ` | ${comment.created_at}` : "" }}</small>
            <button v-if="comment.state !== 'resolved'" type="button" @click="store.resolveReviewComment(Number(comment.line))">Resolve</button>
          </article>
          <h3>AI provenance</h3>
          <article v-for="source in active.compile?.semantic.ai_sources || []" :key="`${source.provider}-${source.model}-${source.date}`" class="snapshot-row">
            <p>{{ source.provider || "unknown" }} / {{ source.model || "unknown" }}</p>
            <small>{{ source.status }} | {{ source.reviewed_by || "unreviewed" }}</small>
          </article>
        </template>

        <template v-else>
          <h2>Settings</h2>
          <label>
            Theme
            <select v-model="store.theme">
              <option value="system">System</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
            </select>
          </label>
          <label><input v-model="store.wordWrap" type="checkbox" /> Word wrap</label>
          <label><input v-model="store.lineNumbers" type="checkbox" /> Line numbers</label>
          <label><input v-model="store.autosave" type="checkbox" /> Autosave existing files</label>
          <label>
            Autosave delay
            <input v-model.number="store.autosaveDelayMs" type="number" min="500" max="30000" step="250" />
          </label>
          <label><input v-model="store.autoSnapshot" type="checkbox" /> Automatic snapshots</label>
          <label>
            Snapshot interval
            <input v-model.number="store.snapshotIntervalMs" type="number" min="30000" max="3600000" step="30000" />
          </label>
          <h3>Typography</h3>
          <label>
            Editor font
            <input v-model="store.editorFont" />
          </label>
          <label>
            Editor line height
            <input v-model.number="store.editorLineHeight" type="number" min="1" max="2.4" step="0.05" />
          </label>
          <label>
            Preview font
            <input v-model="store.previewFont" />
          </label>
          <label>
            Preview line height
            <input v-model.number="store.previewLineHeight" type="number" min="1" max="2.4" step="0.05" />
          </label>
          <h3>Recent files</h3>
          <button v-for="path in store.recentFiles" :key="path" class="outline-row" type="button" @click="store.openPath(path)">
            {{ path }}
          </button>
          <h3>Recent folders</h3>
          <button v-for="path in store.recentFolders" :key="path" class="outline-row" type="button" @click="store.openFolder(path)">
            {{ path }}
          </button>
          <h3>Recently closed</h3>
          <button v-for="path in store.recentlyClosed" :key="path" class="outline-row" type="button" @click="store.openPath(path)">
            {{ path }}
          </button>
          <h3>Transform engines</h3>
          <label>
            Timeout
            <input
              :value="store.transformTimeoutMs"
              type="number"
              min="1"
              max="30000"
              step="250"
              @change="store.setTransformTimeout(Number(eventValue($event)))"
            />
          </label>
          <article v-for="engine in store.externalTransformEngines" :key="engine.name" class="engine-row">
            <h4>{{ engine.name }}</h4>
            <small>{{ engine.execution }}</small>
            <label>
              Engine path
              <span class="path-picker">
                <input :value="store.transformEnginePaths[engine.name] || ''" @change="store.setTransformEnginePath(engine.name, eventValue($event))" />
                <button type="button" @click="chooseTransformEngine(engine.name)">Choose</button>
              </span>
            </label>
            <label><input :checked="Boolean(store.trustedTransformEngines[engine.name])" type="checkbox" @change="store.setTransformTrust(engine.name, eventChecked($event))" /> Trusted</label>
            <label>
              Input
              <select :value="store.transformInputModes[engine.name] || 'stdin'" @change="store.setTransformInputMode(engine.name, eventValue($event) === 'file' ? 'file' : 'stdin')">
                <option v-for="mode in engine.inputModes" :key="mode" :value="mode">{{ mode }}</option>
              </select>
            </label>
            <button type="button" @click="store.testExternalTransform(engine.name)">Probe</button>
          </article>
          <p v-for="engine in store.transformEngines.filter((candidate) => !candidate.requiresExecution)" :key="engine.name" class="engine-summary">
            {{ engine.name }}: {{ engine.execution }}
          </p>
        </template>
      </aside>

      <section v-show="store.mode !== 'preview' && store.mode !== 'export'" class="editor-pane" aria-label="Markdown source">
        <div ref="editorHost" class="editor-host"></div>
      </section>

      <section v-show="store.mode !== 'source' && store.mode !== 'focus'" class="preview-pane" aria-label="Live preview">
        <article class="preview-document" :style="previewDocumentStyle" @click="handlePreviewClick" v-html="active.compile?.html || ''"></article>
      </section>
    </main>

    <footer class="status-bar">
      <span>{{ store.statusMessage }}</span>
      <span v-if="store.externalConflict" class="conflict-actions">
        <button type="button" @click="conflictOpen = true">Compare</button>
        <button type="button" @click="store.acceptExternalChanges">Accept external</button>
        <button type="button" @click="store.keepLocalChanges">Keep local</button>
        <button type="button" @click="saveConflictCopy">Save copy</button>
      </span>
      <span>{{ wordStats }}</span>
      <span v-if="store.lastError" class="error">{{ store.lastError }}</span>
    </footer>

    <section v-if="aiPasteOpen" class="modal-backdrop" role="dialog" aria-modal="true" aria-label="AI paste cleanup">
      <form class="modal" @submit.prevent="cleanAiPaste">
        <header>
          <h2>Paste from AI Chat</h2>
          <button type="button" @click="closeAiPaste">x</button>
        </header>
        <section class="compare-grid ai-paste-grid">
          <label>
            Original
            <textarea v-model="aiPasteText" rows="12" placeholder="Paste AI chat output here"></textarea>
          </label>
          <label>
            Cleaned preview
            <textarea :value="store.aiCleanupPreview?.cleaned_markdown || ''" rows="12" readonly placeholder="Preview cleaned Markdown"></textarea>
          </label>
        </section>
        <label><input v-model="aiMarkAsDraft" type="checkbox" /> Mark as draft</label>
        <label><input v-model="aiAddProvenance" type="checkbox" /> Add provenance block</label>
        <label>
          Insert mode
          <select v-model="aiInsertMode">
            <option value="insert">Insert after document</option>
            <option value="appendix">Appendix</option>
            <option value="replace">Replace document</option>
          </select>
        </label>
        <section v-if="store.aiCleanupIssues.length" class="issue-list">
          <p v-for="issue in store.aiCleanupIssues" :key="issue">{{ issue }}</p>
        </section>
        <footer>
          <button type="button" @click="closeAiPaste">Cancel</button>
          <button type="button" :disabled="aiPreviewBusy || !aiPasteText.trim()" @click="previewAiPaste">
            {{ aiPreviewBusy ? "Cleaning" : "Preview cleanup" }}
          </button>
          <button type="submit" :disabled="aiPreviewBusy || !aiPasteText.trim()">Insert cleaned</button>
        </footer>
      </form>
    </section>

    <section v-if="commandPaletteOpen" class="modal-backdrop" role="dialog" aria-modal="true" aria-label="Command palette">
      <div class="modal command-modal">
        <header>
          <h2>Command Palette</h2>
          <button type="button" @click="commandPaletteOpen = false">x</button>
        </header>
        <input v-model="commandQuery" autofocus placeholder="Search commands, headings, citations, glossary terms" />
        <button
          v-for="command in filteredCommands"
          :key="command.name"
          class="command-row"
          type="button"
          @click="runCommand(command.run)"
        >
          <strong>{{ command.name }}</strong>
          <span>{{ command.group }}</span>
        </button>
      </div>
    </section>

    <section v-if="conflictOpen && store.externalConflict" class="modal-backdrop" role="dialog" aria-modal="true" aria-label="External file conflict">
      <div class="modal conflict-modal">
        <header>
          <h2>External Changes</h2>
          <button type="button" @click="conflictOpen = false">x</button>
        </header>
        <p>{{ store.externalConflict.message }}</p>
        <section class="compare-grid">
          <article>
            <h3>Local</h3>
            <pre>{{ active.text }}</pre>
          </article>
          <article>
            <h3>External</h3>
            <pre>{{ store.externalConflict.externalText || "Included file changed. Recompile to update the preview." }}</pre>
          </article>
        </section>
        <footer>
          <button type="button" @click="store.keepLocalChanges(); conflictOpen = false">Keep local</button>
          <button type="button" @click="saveConflictCopy">Save copy</button>
          <button type="button" @click="store.acceptExternalChanges(); conflictOpen = false">Accept external</button>
        </footer>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { open, save } from "@tauri-apps/plugin-dialog";
import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap, history, historyKeymap } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { searchKeymap } from "@codemirror/search";
import { closeBrackets, closeBracketsKeymap } from "@codemirror/autocomplete";
import { forceLinting, linter, lintGutter, type Diagnostic as CodeMirrorDiagnostic } from "@codemirror/lint";
import { useDocumentsStore } from "./stores/documents";
import type { DocumentDiagnostic } from "./types";

const store = useDocumentsStore();
const editorHost = ref<HTMLElement | null>(null);
let editorView: EditorView | null = null;
let debounceHandle = 0;
let autosaveHandle = 0;
let autoSnapshotHandle = 0;
let lastAutoSnapshotSignature = "";
const aiPasteOpen = ref(false);
const aiPasteText = ref("");
const aiInsertMode = ref<"insert" | "replace" | "appendix">("insert");
const aiAddProvenance = ref(true);
const aiMarkAsDraft = ref(true);
const aiPreviewBusy = ref(false);
const aiPreviewSignature = ref("");
const commandPaletteOpen = ref(false);
const conflictOpen = ref(false);
const commandQuery = ref("");
const reviewCommentText = ref("");
const selectedTableIndex = ref(0);
const tablePasteText = ref("");
const tableDraft = ref<TableDraft | null>(null);

interface MarkdownTable {
  startLine: number;
  endLine: number;
  headers: string[];
  alignments: TableAlignment[];
  rows: string[][];
}

type TableAlignment = "left" | "center" | "right";
type TableFormat = "text" | "number" | "currency" | "percent" | "date";

interface TableDraft {
  headers: string[];
  alignments: TableAlignment[];
  formats: TableFormat[];
  rows: string[][];
}

const tableSnippet = `| Item | Value |\n| --- | ---: |\n| Revenue | 125000 |\n`;
const calcSnippet = "```calc\nrevenue = 125000\ncost = 74000\nprofit = revenue - cost\n```\n";
const aiSnippet = "```ai-source\nprovider: OpenAI\nmodel: ChatGPT\ndate: 2026-05-18\nreviewedBy: \nstatus: human-reviewed\n```\n";

const active = computed(() => store.activeDocument);
const previewDocumentStyle = computed(() => ({
  fontFamily: store.previewFont,
  lineHeight: String(clampUiLineHeight(store.previewLineHeight)),
}));
const wordStats = computed(() => {
  const text = active.value?.text || "";
  const words = text.trim().split(/\s+/).filter(Boolean).length;
  const minutes = words ? Math.max(1, Math.ceil(words / 220)) : 0;
  return `${words} words | ${text.length} characters | ${minutes} min read`;
});
const manifestPreview = computed(() => JSON.stringify(active.value.compile?.export_manifest || {}, null, 2));
const bibliographyByKey = computed(() => new Map((active.value.compile?.bibliography || []).map((entry) => [entry.key, entry.title])));
const markdownTables = computed(() => parseMarkdownTables(active.value?.text || ""));
const selectedTable = computed(() => markdownTables.value[selectedTableIndex.value] || null);
const tableColumnTotals = computed(() => {
  const draft = tableDraft.value;
  if (!draft) return [];
  return draft.headers.map((_, columnIndex) => formatTableTotal(draft, columnIndex));
});
const diagnosticSignature = computed(() =>
  (active.value.compile?.diagnostics || [])
    .map((diagnostic) => [diagnostic.severity, diagnostic.source_file || "", diagnostic.line || "", diagnostic.message].join(":"))
    .join("\n"),
);
const commands = computed(() => [
  { name: "New document", group: "File", run: () => store.newDocument() },
  { name: "Open document", group: "File", run: () => void openDocument() },
  { name: "Open folder", group: "Workspace", run: () => void openFolder() },
  { name: "Save document", group: "File", run: () => void saveDocument() },
  { name: "Save as", group: "File", run: () => void saveDocumentAs() },
  { name: "Revert to saved", group: "File", run: () => void store.revertActive() },
  { name: "Rename document", group: "File", run: () => void renameDocument() },
  { name: "Duplicate document", group: "File", run: () => void duplicateDocument() },
  { name: "Prepare for export", group: "Export", run: () => void store.prepareForExport() },
  { name: "Export document", group: "Export", run: () => void exportDocument() },
  { name: "Create snapshot", group: "Versioning", run: () => void store.snapshotActive() },
  { name: "Refresh Git diff", group: "Versioning", run: () => void store.refreshGitDiff() },
  { name: "Commit document", group: "Versioning", run: () => void store.commitActive() },
  { name: "Tag release", group: "Versioning", run: () => void store.tagActiveRelease() },
  { name: "Paste from AI chat", group: "AI", run: () => (aiPasteOpen.value = true) },
  { name: "Add review comment", group: "Review", run: () => (store.sidebar = "review") },
  { name: "Open table editor", group: "Tables", run: () => openTableEditor() },
  { name: "Insert table", group: "Snippet", run: () => insertBlock(tableSnippet) },
  { name: "Insert calculation", group: "Snippet", run: () => insertBlock(calcSnippet) },
  { name: "Insert AI source", group: "Snippet", run: () => insertBlock(aiSnippet) },
  {
    name: active.value.pinned ? "Unpin active tab" : "Pin active tab",
    group: "Workspace",
    run: () => store.togglePin(active.value.id),
  },
  ...store.documents.map((document) => ({
    name: document.title,
    group: "Open document",
    run: () => {
      store.activeId = document.id;
    },
  })),
  ...store.workspaceFiles
    .filter((entry) => entry.kind !== "directory")
    .map((entry) => ({
      name: entry.relative_path,
      group: "Workspace file",
      run: () => void store.openPath(entry.path),
    })),
  ...((active.value.compile?.semantic.outline || []).map((heading) => ({
    name: heading.text,
    group: `Heading line ${heading.line}`,
    run: () => goToLine(heading.line),
  }))),
  ...((active.value.compile?.semantic.citations || []).map((citation) => ({
    name: `[@${citation}]`,
    group: "Citation",
    run: () => {
      store.sidebar = "references";
    },
  }))),
  ...Object.keys(active.value.compile?.semantic.glossary || {}).map((term) => ({
    name: term,
    group: "Glossary",
    run: () => {
      store.sidebar = "references";
    },
  })),
  ...((active.value.compile?.diagnostics || []).map((diagnostic) => ({
    name: diagnostic.message,
    group: `Diagnostic ${diagnostic.severity}`,
    run: () => {
      store.sidebar = "diagnostics";
      if (diagnostic.line) goToLine(diagnostic.line);
    },
  }))),
]);
const filteredCommands = computed(() => {
  const query = commandQuery.value.trim().toLowerCase();
  if (!query) return commands.value;
  return commands.value.filter((command) => `${command.name} ${command.group}`.toLowerCase().includes(query));
});

onMounted(async () => {
  await store.boot();
  buildEditor();
  scheduleAutosave();
  scheduleAutoSnapshot();
  document.title = store.windowTitle;
  window.addEventListener("keydown", handleShortcut);
});

onBeforeUnmount(() => {
  editorView?.destroy();
  window.clearTimeout(autosaveHandle);
  window.clearTimeout(autoSnapshotHandle);
  window.removeEventListener("keydown", handleShortcut);
});

watch(
  () => active.value.id,
  async () => {
    await nextTick();
    selectedTableIndex.value = 0;
    loadSelectedTable();
    buildEditor();
  },
);

watch(
  () => active.value.text,
  (text) => {
    if (!editorView || editorView.state.doc.toString() === text) return;
    editorView.dispatch({
      changes: { from: 0, to: editorView.state.doc.length, insert: text },
    });
  },
);

watch(
  () => [active.value.id, active.value.text, active.value.path, active.value.dirty, store.autosave, store.autosaveDelayMs, store.externalConflict?.externalHash],
  () => {
    scheduleAutosave();
  },
);

watch(
  () => [active.value.id, active.value.text, active.value.dirty, store.autoSnapshot, store.snapshotIntervalMs, store.externalConflict?.externalHash],
  () => {
    scheduleAutoSnapshot();
  },
);

watch(
  () => [store.wordWrap, store.lineNumbers, store.theme, store.editorFont, store.editorLineHeight, store.previewFont, store.previewLineHeight],
  () => {
    buildEditor();
    void store.persistWorkspace();
  },
);

watch(
  () => [store.autosave, store.autosaveDelayMs, store.autoSnapshot, store.snapshotIntervalMs],
  () => {
    scheduleAutosave();
    scheduleAutoSnapshot();
    void store.persistWorkspace();
  },
);

watch(
  () => store.exportTarget,
  () => {
    void store.persistWorkspace();
  },
);

watch(
  () => store.windowTitle,
  (title) => {
    document.title = title;
  },
);

watch(diagnosticSignature, () => {
  if (editorView) forceLinting(editorView);
});

watch(
  markdownTables,
  (tables) => {
    if (!tables.length) {
      tableDraft.value = null;
      selectedTableIndex.value = 0;
      return;
    }
    if (selectedTableIndex.value >= tables.length) {
      selectedTableIndex.value = tables.length - 1;
    }
    if (!tableDraft.value) loadSelectedTable();
  },
  { immediate: true },
);

function editorExtensions() {
  return [
    ...(store.lineNumbers ? [lineNumbers()] : []),
    lintGutter(),
    history(),
    markdown(),
    linter(editorDiagnostics, { delay: 150 }),
    closeBrackets(),
    EditorView.contentAttributes.of({ spellcheck: "true", autocapitalize: "sentences" }),
    keymap.of([{ key: "Enter", run: continueMarkdownList }, ...closeBracketsKeymap, ...defaultKeymap, ...historyKeymap, ...searchKeymap]),
    ...(store.wordWrap ? [EditorView.lineWrapping] : []),
    EditorView.updateListener.of((update) => {
      if (!update.docChanged) return;
      window.clearTimeout(debounceHandle);
      debounceHandle = window.setTimeout(() => {
        store.updateText(update.state.doc.toString());
      }, 120);
    }),
    EditorView.theme({
      "&": {
        height: "100%",
        fontSize: "14px",
      },
      ".cm-scroller": {
        fontFamily: store.editorFont,
        lineHeight: String(clampUiLineHeight(store.editorLineHeight)),
      },
    }),
  ];
}

function continueMarkdownList(view: EditorView) {
  const selection = view.state.selection.main;
  if (!selection.empty) return false;
  const line = view.state.doc.lineAt(selection.head);
  const beforeCursor = line.text.slice(0, selection.head - line.from);
  const bullet = beforeCursor.match(/^(\s*)([-+*])\s+(.*)$/);
  const numbered = beforeCursor.match(/^(\s*)(\d+)([.)])\s+(.*)$/);
  if (!bullet && !numbered) return false;

  const indent = (bullet?.[1] || numbered?.[1] || "");
  const marker = bullet ? bullet[2] : `${Number(numbered?.[2] || "0") + 1}${numbered?.[3] || "."}`;
  const content = (bullet?.[3] || numbered?.[4] || "").trim();
  if (!content) {
    view.dispatch({
      changes: { from: line.from, to: selection.head, insert: indent },
      selection: { anchor: line.from + indent.length },
      scrollIntoView: true,
    });
    return true;
  }

  const insert = `\n${indent}${marker} `;
  view.dispatch({
    changes: { from: selection.head, insert },
    selection: { anchor: selection.head + insert.length },
    scrollIntoView: true,
  });
  return true;
}

function editorDiagnostics(view: EditorView): CodeMirrorDiagnostic[] {
  return (active.value.compile?.diagnostics || []).flatMap((diagnostic) => codeMirrorDiagnostic(view, diagnostic));
}

function codeMirrorDiagnostic(view: EditorView, diagnostic: DocumentDiagnostic): CodeMirrorDiagnostic[] {
  if (!diagnostic.line || diagnosticAppliesToIncludedFile(diagnostic)) return [];
  const line = view.state.doc.line(Math.max(1, Math.min(diagnostic.line, view.state.doc.lines)));
  const message = diagnostic.suggestion ? `${diagnostic.message}\n${diagnostic.suggestion}` : diagnostic.message;
  return [
    {
      from: line.from,
      to: Math.max(line.from, line.to),
      severity: diagnostic.severity,
      message,
      source: diagnostic.source_file || "compiler",
    },
  ];
}

function diagnosticAppliesToIncludedFile(diagnostic: DocumentDiagnostic) {
  const sourceFile = diagnostic.source_file;
  const activePath = active.value.path;
  return Boolean(sourceFile && activePath && sourceFile !== activePath);
}

function buildEditor() {
  if (!editorHost.value) return;
  editorView?.destroy();
  editorView = new EditorView({
    state: EditorState.create({
      doc: active.value.text,
      extensions: editorExtensions(),
    }),
    parent: editorHost.value,
  });
}

function activate(id: string) {
  store.activeId = id;
}

async function openDocument() {
  const selected = await open({
    multiple: false,
    filters: [{ name: "Markdown", extensions: ["md", "markdown", "mdown", "txt"] }],
  });
  if (typeof selected === "string") await store.openPath(selected);
}

async function openFolder() {
  const selected = await open({
    directory: true,
    multiple: false,
  });
  if (typeof selected === "string") await store.openFolder(selected);
}

function eventValue(event: Event) {
  return event.target instanceof HTMLInputElement || event.target instanceof HTMLSelectElement ? event.target.value : "";
}

function eventChecked(event: Event) {
  return event.target instanceof HTMLInputElement ? event.target.checked : false;
}

function clampUiLineHeight(value: number) {
  return Math.min(Math.max(Number(value) || 1.55, 1), 2.4);
}

function clampAutosaveDelay(value: number) {
  return Math.min(Math.max(Number(value) || 1500, 500), 30000);
}

function clampSnapshotInterval(value: number) {
  return Math.min(Math.max(Number(value) || 300000, 30000), 3600000);
}

function scheduleAutosave() {
  window.clearTimeout(autosaveHandle);
  const doc = active.value;
  if (!store.autosave || !doc.path || !doc.dirty || store.externalConflict) return;
  autosaveHandle = window.setTimeout(() => {
    void store.saveActive().catch((error) => {
      store.lastError = error instanceof Error ? error.message : String(error);
      store.statusMessage = "Autosave failed";
    });
  }, clampAutosaveDelay(store.autosaveDelayMs));
}

function scheduleAutoSnapshot() {
  window.clearTimeout(autoSnapshotHandle);
  const doc = active.value;
  const signature = `${doc.id}:${doc.text}`;
  if (!store.autoSnapshot || !doc.dirty || store.externalConflict || lastAutoSnapshotSignature === signature) return;
  autoSnapshotHandle = window.setTimeout(() => {
    const current = active.value;
    const currentSignature = `${current.id}:${current.text}`;
    if (lastAutoSnapshotSignature === currentSignature || store.externalConflict) return;
    void store
      .createSnapshot("auto")
      .then(() => {
        lastAutoSnapshotSignature = currentSignature;
        void store.listSnapshots();
      })
      .catch((error) => {
        store.lastError = error instanceof Error ? error.message : String(error);
        store.statusMessage = "Automatic snapshot failed";
      });
  }, clampSnapshotInterval(store.snapshotIntervalMs));
}

async function chooseTransformEngine(name: string) {
  const selected = await open({
    multiple: false,
  });
  if (typeof selected === "string") await store.setTransformEnginePath(name, selected);
}

async function saveDocument() {
  if (!active.value.path) {
    await saveDocumentAs();
    return;
  }
  await store.saveActive();
}

async function saveDocumentAs() {
  const path = await save({
    filters: [{ name: "Markdown", extensions: ["md"] }],
    defaultPath: active.value.title.endsWith(".md") ? active.value.title : `${active.value.title}.md`,
  });
  if (path) await store.saveActive(path);
}

async function renameDocument() {
  const path = await save({
    filters: [{ name: "Markdown", extensions: ["md"] }],
    defaultPath: active.value.title.endsWith(".md") ? active.value.title : `${active.value.title}.md`,
  });
  if (path) await store.renameActive(path);
}

async function duplicateDocument() {
  const path = await save({
    filters: [{ name: "Markdown", extensions: ["md"] }],
    defaultPath: `${active.value.title.replace(/\.[^.]+$/, "")} copy.md`,
  });
  if (path) await store.duplicateActive(path);
}

async function saveConflictCopy() {
  const path = await save({
    filters: [{ name: "Markdown", extensions: ["md"] }],
    defaultPath: `${active.value.title.replace(/\.[^.]+$/, "")} local copy.md`,
  });
  if (path) {
    await store.saveLocalConflictCopy(path);
    conflictOpen.value = false;
  }
}

async function exportDocument() {
  const extensions: Record<typeof store.exportTarget, string> = {
    html: "html",
    pdf: "pdf",
    docx: "docx",
    pptx: "pptx",
    "markdown-bundle": "zip",
  };
  const extension = extensions[store.exportTarget];
  const path = await save({
    filters: [{ name: store.exportTarget.toUpperCase(), extensions: [extension] }],
    defaultPath: `${active.value.title.replace(/\.[^.]+$/, "")}.${extension}`,
  });
  if (path) await store.exportActive(path);
}

async function cleanAiPaste() {
  if (!aiPasteText.value.trim()) return;
  if (aiPreviewSignature.value !== aiCleanupSignature() || !store.aiCleanupPreview) {
    await previewAiPaste();
  }
  if (!store.aiCleanupPreview) return;
  store.insertAiPaste(store.aiCleanupPreview, aiInsertMode.value);
  closeAiPaste();
}

async function previewAiPaste() {
  if (!aiPasteText.value.trim()) return;
  aiPreviewBusy.value = true;
  try {
    await store.previewAiPaste(aiPasteText.value, {
      addProvenance: aiAddProvenance.value,
      markAsDraft: aiMarkAsDraft.value,
    });
    aiPreviewSignature.value = aiCleanupSignature();
  } finally {
    aiPreviewBusy.value = false;
  }
}

function closeAiPaste() {
  aiPasteText.value = "";
  aiPreviewSignature.value = "";
  store.aiCleanupPreview = null;
  store.aiCleanupIssues = [];
  aiPasteOpen.value = false;
}

function aiCleanupSignature() {
  return JSON.stringify({
    text: aiPasteText.value,
    addProvenance: aiAddProvenance.value,
    markAsDraft: aiMarkAsDraft.value,
  });
}

function runCommand(run: () => unknown) {
  run();
  commandPaletteOpen.value = false;
  commandQuery.value = "";
}

function insertReviewComment() {
  store.insertReviewComment(reviewCommentText.value);
  reviewCommentText.value = "";
}

function wrapSelection(prefix: string, suffix = prefix) {
  if (!editorView) return;
  const range = editorView.state.selection.main;
  const selected = editorView.state.sliceDoc(range.from, range.to);
  editorView.dispatch({
    changes: { from: range.from, to: range.to, insert: `${prefix}${selected}${suffix}` },
    selection: { anchor: range.from + prefix.length, head: range.to + prefix.length },
  });
  editorView.focus();
}

function insertAtLineStart(prefix: string) {
  if (!editorView) return;
  const line = editorView.state.doc.lineAt(editorView.state.selection.main.from);
  editorView.dispatch({ changes: { from: line.from, insert: prefix } });
  editorView.focus();
}

function insertBlock(block: string) {
  if (!editorView) return;
  const position = editorView.state.selection.main.to;
  editorView.dispatch({ changes: { from: position, insert: `\n${block}\n` } });
  editorView.focus();
}

function openTableEditor() {
  store.sidebar = "tables";
  if (markdownTables.value.length && !tableDraft.value) loadSelectedTable();
}

function loadSelectedTable() {
  const table = selectedTable.value;
  if (!table) {
    tableDraft.value = null;
    return;
  }
  tableDraft.value = {
    headers: [...table.headers],
    alignments: [...table.alignments],
    formats: table.headers.map((_, columnIndex) => inferTableFormat(table.rows.map((row) => row[columnIndex] || ""))),
    rows: table.rows.map((row) => padTableRow(row, table.headers.length)),
  };
}

function applyTableDraft() {
  const table = selectedTable.value;
  const draft = tableDraft.value;
  if (!table || !draft) return;
  const normalizedDraft = normalizeTableDraft(draft);
  const lines = active.value.text.split("\n");
  lines.splice(table.startLine - 1, table.endLine - table.startLine + 1, ...serializeMarkdownTable(normalizedDraft));
  store.updateText(lines.join("\n"));
  tableDraft.value = normalizedDraft;
}

function addTableRow() {
  if (!tableDraft.value) return;
  tableDraft.value.rows.push(tableDraft.value.headers.map(() => ""));
}

function removeTableRow(rowIndex: number) {
  if (!tableDraft.value) return;
  tableDraft.value.rows.splice(rowIndex, 1);
}

function addTableColumn() {
  if (!tableDraft.value) return;
  const nextColumn = tableDraft.value.headers.length + 1;
  tableDraft.value.headers.push(`Column ${nextColumn}`);
  tableDraft.value.alignments.push("left");
  tableDraft.value.formats.push("text");
  for (const row of tableDraft.value.rows) row.push("");
}

function removeTableColumn(columnIndex: number) {
  if (!tableDraft.value || tableDraft.value.headers.length <= 1) return;
  tableDraft.value.headers.splice(columnIndex, 1);
  tableDraft.value.alignments.splice(columnIndex, 1);
  tableDraft.value.formats.splice(columnIndex, 1);
  for (const row of tableDraft.value.rows) row.splice(columnIndex, 1);
}

function addTableTotalsRow() {
  const draft = tableDraft.value;
  if (!draft) return;
  const totals = draft.headers.map((_, columnIndex) => {
    if (columnIndex === 0) return "Total";
    const values = numericColumnValues(draft, columnIndex);
    if (!values.length) return "";
    return `=SUM(${values.map(formatFormulaNumber).join(",")})`;
  });
  draft.rows.push(totals);
}

function replaceTableFromPaste() {
  const rows = parseDelimitedRows(tablePasteText.value);
  if (!rows.length) return;
  const headers = rows[0].map((cell, index) => cell.trim() || `Column ${index + 1}`);
  const bodyRows = rows.slice(1).map((row) => padTableRow(row, headers.length));
  tableDraft.value = {
    headers,
    alignments: headers.map(() => "left"),
    formats: headers.map((_, columnIndex) => inferTableFormat(bodyRows.map((row) => row[columnIndex] || ""))),
    rows: bodyRows.length ? bodyRows : [headers.map(() => "")],
  };
}

function sortTableRows(columnIndex: number) {
  if (!tableDraft.value) return;
  const format = tableDraft.value.formats[columnIndex];
  tableDraft.value.rows.sort((left, right) => compareTableCells(left[columnIndex] || "", right[columnIndex] || "", format));
}

function parseMarkdownTables(text: string): MarkdownTable[] {
  const lines = text.split("\n");
  const tables: MarkdownTable[] = [];
  let index = 0;
  while (index + 1 < lines.length) {
    const header = lines[index].trim();
    const separator = lines[index + 1].trim();
    if (!isMarkdownTableRow(header) || !isMarkdownTableSeparator(separator)) {
      index += 1;
      continue;
    }
    const headers = splitMarkdownTableRow(header);
    const alignments = splitMarkdownTableRow(separator).map(alignmentFromSeparator);
    const rows: string[][] = [];
    let nextIndex = index + 2;
    while (nextIndex < lines.length && isMarkdownTableRow(lines[nextIndex].trim())) {
      rows.push(padTableRow(splitMarkdownTableRow(lines[nextIndex].trim()), headers.length));
      nextIndex += 1;
    }
    tables.push({
      startLine: index + 1,
      endLine: nextIndex,
      headers,
      alignments: padAlignments(alignments, headers.length),
      rows,
    });
    index = nextIndex;
  }
  return tables;
}

function isMarkdownTableRow(line: string) {
  return line.startsWith("|") && line.endsWith("|") && (line.match(/\|/g) || []).length >= 2;
}

function isMarkdownTableSeparator(line: string) {
  return isMarkdownTableRow(line) && splitMarkdownTableRow(line).every((cell) => /^:?-{3,}:?$/.test(cell.replace(/\s/g, "")));
}

function splitMarkdownTableRow(line: string) {
  return line
    .trim()
    .slice(1, -1)
    .split("|")
    .map((cell) => cell.replace(/\\\|/g, "|").trim());
}

function alignmentFromSeparator(cell: string): TableAlignment {
  const compact = cell.replace(/\s/g, "");
  if (compact.startsWith(":") && compact.endsWith(":")) return "center";
  if (compact.endsWith(":")) return "right";
  return "left";
}

function padAlignments(alignments: TableAlignment[], length: number) {
  return Array.from({ length }, (_, index) => alignments[index] || "left");
}

function padTableRow(row: string[], length: number) {
  return Array.from({ length }, (_, index) => row[index] || "");
}

function normalizeTableDraft(draft: TableDraft): TableDraft {
  const headers = draft.headers.map((header, index) => header.trim() || `Column ${index + 1}`);
  return {
    headers,
    alignments: padAlignments(draft.alignments, headers.length),
    formats: Array.from({ length: headers.length }, (_, index) => draft.formats[index] || "text"),
    rows: draft.rows.map((row) => padTableRow(row, headers.length)),
  };
}

function serializeMarkdownTable(draft: TableDraft) {
  const headers = draft.headers.map(escapeTableCell);
  const separator = draft.alignments.map(separatorForAlignment);
  const rows = draft.rows.map((row) =>
    row.map((cell, columnIndex) => escapeTableCell(formatTableCell(cell, draft.formats[columnIndex]))),
  );
  return [`| ${headers.join(" | ")} |`, `| ${separator.join(" | ")} |`, ...rows.map((row) => `| ${row.join(" | ")} |`)];
}

function separatorForAlignment(alignment: TableAlignment) {
  if (alignment === "center") return ":---:";
  if (alignment === "right") return "---:";
  return "---";
}

function escapeTableCell(cell: string) {
  return cell.replace(/\r?\n/g, " ").replace(/\|/g, "\\|").trim();
}

function parseDelimitedRows(text: string) {
  const rows = text
    .trim()
    .split(/\r?\n/)
    .filter((line) => line.trim().length)
    .map((line) => (line.includes("\t") ? line.split("\t").map((cell) => cell.trim()) : parseCsvLine(line)));
  const width = Math.max(0, ...rows.map((row) => row.length));
  return rows.map((row) => padTableRow(row, width));
}

function parseCsvLine(line: string) {
  const cells: string[] = [];
  let cell = "";
  let quoted = false;
  for (let index = 0; index < line.length; index += 1) {
    const char = line[index];
    const next = line[index + 1];
    if (char === '"' && quoted && next === '"') {
      cell += '"';
      index += 1;
    } else if (char === '"') {
      quoted = !quoted;
    } else if (char === "," && !quoted) {
      cells.push(cell.trim());
      cell = "";
    } else {
      cell += char;
    }
  }
  cells.push(cell.trim());
  return cells;
}

function inferTableFormat(values: string[]): TableFormat {
  const filled = values.map((value) => value.trim()).filter(Boolean);
  if (!filled.length) return "text";
  if (filled.every((value) => /^\$?-?\d[\d,]*(\.\d+)?$/.test(value))) {
    return filled.some((value) => value.startsWith("$")) ? "currency" : "number";
  }
  if (filled.every((value) => /^-?\d+(\.\d+)?%$/.test(value))) return "percent";
  if (filled.every((value) => !Number.isNaN(Date.parse(value)))) return "date";
  return "text";
}

function compareTableCells(left: string, right: string, format: TableFormat) {
  if (format === "number" || format === "currency" || format === "percent") {
    return parseCellNumber(left) - parseCellNumber(right);
  }
  if (format === "date") {
    return Date.parse(left) - Date.parse(right);
  }
  return left.localeCompare(right);
}

function formatTableCell(value: string, format: TableFormat) {
  const trimmed = value.trim();
  if (!trimmed || format === "text") return trimmed;
  if (format === "date") {
    const time = Date.parse(trimmed);
    return Number.isNaN(time) ? trimmed : new Date(time).toISOString().slice(0, 10);
  }
  const number = parseCellNumber(trimmed);
  if (Number.isNaN(number)) return trimmed;
  if (format === "currency") return `$${trimFixed(number, 2)}`;
  if (format === "percent") {
    const percent = trimmed.includes("%") || Math.abs(number) > 1 ? number : number * 100;
    return `${trimFixed(percent, 2)}%`;
  }
  return trimFixed(number, 2);
}

function formatTableTotal(draft: TableDraft, columnIndex: number) {
  const values = numericColumnValues(draft, columnIndex);
  if (!values.length) return "";
  const total = values.reduce((sum, value) => sum + value, 0);
  return formatTableCell(String(total), draft.formats[columnIndex]);
}

function numericColumnValues(draft: TableDraft, columnIndex: number) {
  return draft.rows
    .map((row) => parseEditableTableNumber(row[columnIndex] || ""))
    .filter((value): value is number => Number.isFinite(value));
}

function parseEditableTableNumber(value: string) {
  const trimmed = value.trim();
  if (!trimmed || trimmed.startsWith("=")) return Number.NaN;
  return parseCellNumber(trimmed);
}

function formatFormulaNumber(value: number) {
  return Number.isInteger(value) ? String(value) : trimFixed(value, 6);
}

function parseCellNumber(value: string) {
  return Number(value.replace(/[$,%]/g, ""));
}

function trimFixed(value: number, places: number) {
  return value.toFixed(places).replace(/\.?0+$/, "");
}

function goToLine(lineNumber: number) {
  if (!editorView) return;
  const line = editorView.state.doc.line(Math.max(1, Math.min(lineNumber, editorView.state.doc.lines)));
  editorView.dispatch({ selection: { anchor: line.from }, effects: EditorView.scrollIntoView(line.from, { y: "center" }) });
  editorView.focus();
}

function handlePreviewClick(event: MouseEvent) {
  const target = event.target;
  if (!(target instanceof Element)) return;
  const link = target.closest("a[href^='#']");
  const heading = target.closest("h1[id], h2[id], h3[id], h4[id], h5[id], h6[id]");
  const anchor = heading?.id || link?.getAttribute("href")?.slice(1) || "";
  if (!anchor) return;
  const headingEntry = active.value.compile?.semantic.outline.find((item) => item.anchor === anchor);
  if (!headingEntry) return;
  event.preventDefault();
  goToLine(headingEntry.line);
}

function handleShortcut(event: KeyboardEvent) {
  if (!(event.metaKey || event.ctrlKey)) return;
  if (event.key === "s") {
    event.preventDefault();
    void saveDocument();
  } else if (event.key === "o") {
    event.preventDefault();
    void openDocument();
  } else if (event.key === "n") {
    event.preventDefault();
    store.newDocument();
  } else if (event.key === "b") {
    event.preventDefault();
    wrapSelection("**");
  } else if (event.key === "i") {
    event.preventDefault();
    wrapSelection("*");
  } else if (event.key.toLowerCase() === "p" && event.shiftKey) {
    event.preventDefault();
    commandPaletteOpen.value = true;
  }
}
</script>

<style>
:root {
  color: #18212f;
  background: #edf1f5;
  font-family:
    Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  font-size: 14px;
  font-synthesis: none;
  line-height: 1.45;
  text-rendering: optimizeLegibility;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
}

button,
select,
input {
  font: inherit;
}

button,
select {
  border: 1px solid #bac4d1;
  border-radius: 6px;
  background: #f9fbfc;
  color: #18212f;
  min-height: 30px;
}

button {
  padding: 4px 10px;
  cursor: pointer;
}

button:hover,
select:hover {
  border-color: #6386b4;
}

.app-shell {
  display: grid;
  grid-template-rows: 38px 42px minmax(0, 1fr) 28px;
  width: 100vw;
  height: 100vh;
  color: #18212f;
  background: #edf1f5;
}

.app-shell[data-theme="dark"] {
  color: #e6edf5;
  background: #111821;
}

@media (prefers-color-scheme: dark) {
  .app-shell[data-theme="system"] {
    color: #e6edf5;
    background: #111821;
  }
}

.titlebar,
.command-bar,
.status-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 10px;
  border-bottom: 1px solid #c9d2dc;
  background: #f7f9fb;
}

.document-tabs {
  display: flex;
  min-width: 0;
  flex: 1;
  gap: 4px;
  overflow: hidden;
}

.tab {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  max-width: 220px;
  min-height: 30px;
  border: 1px solid #bac4d1;
  border-radius: 6px;
  background: #edf1f5;
}

.tab.active {
  border-color: #275da8;
  background: #ffffff;
}

.tab-main {
  min-width: 0;
  flex: 1;
  border: 0;
  background: transparent;
  text-align: left;
}

.tab-main span {
  display: block;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tab-close {
  min-width: 20px;
  min-height: 20px;
  padding: 0;
  border: 0;
  background: transparent;
}

.window-meta,
.status-bar {
  color: #526171;
  font-size: 12px;
}

.command-bar {
  overflow-x: auto;
}

.divider {
  width: 1px;
  height: 22px;
  background: #c9d2dc;
}

.workspace {
  display: grid;
  grid-template-columns: 260px minmax(0, 1fr) minmax(0, 1fr);
  min-height: 0;
}

.workspace.mode-source,
.workspace.mode-focus {
  grid-template-columns: 260px minmax(0, 1fr);
}

.workspace.mode-preview,
.workspace.mode-export {
  grid-template-columns: 260px minmax(0, 1fr);
}

.sidebar,
.editor-pane,
.preview-pane {
  min-height: 0;
  overflow: auto;
  border-right: 1px solid #c9d2dc;
}

.sidebar {
  padding: 12px;
  background: #f7f9fb;
}

.sidebar h2 {
  margin: 0 0 12px;
  font-size: 13px;
  text-transform: uppercase;
  color: #526171;
}

.sidebar h3 {
  margin: 16px 0 8px;
  font-size: 13px;
}

.sidebar label {
  display: grid;
  gap: 6px;
  margin-bottom: 12px;
}

.outline-row {
  display: block;
  width: 100%;
  margin-bottom: 2px;
  border: 0;
  background: transparent;
  text-align: left;
}

.file-row {
  display: grid;
  grid-template-columns: 18px minmax(0, 1fr);
  width: 100%;
  margin: 1px 0;
  border: 0;
  background: transparent;
  text-align: left;
}

.file-row span:last-child,
.workspace-root {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-row.directory {
  color: #526171;
  font-weight: 600;
}

.file-row.active {
  background: #e5edf7;
  color: #174a88;
}

.workspace-root {
  margin: 8px 0;
  color: #526171;
  font-size: 12px;
}

.diagnostic {
  margin-bottom: 8px;
  padding: 8px;
  border-left: 3px solid #6386b4;
  background: #ffffff;
}

.diagnostic.warning {
  border-color: #c68a1a;
}

.diagnostic.error {
  border-color: #c24141;
}

.diagnostic p {
  margin: 4px 0;
}

.readiness,
.snapshot-row,
.issue-list {
  margin: 8px 0;
  padding: 8px;
  border: 1px solid #c9d2dc;
  background: #ffffff;
}

.readiness {
  border-left: 3px solid #c68a1a;
}

.readiness.ready {
  border-left-color: #2f855a;
}

.snapshot-row p {
  margin: 0 0 4px;
}

.engine-row {
  display: grid;
  gap: 8px;
  margin: 8px 0;
  padding: 8px;
  border: 1px solid #c9d2dc;
  background: #ffffff;
}

.engine-row h4 {
  margin: 0;
  font-size: 13px;
}

.path-picker {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 6px;
}

.path-picker input {
  min-width: 0;
}

.engine-summary {
  margin: 4px 0;
  color: #526171;
  font-size: 12px;
}

.table-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-bottom: 12px;
}

.table-editor-grid {
  display: grid;
  gap: 4px;
  min-width: 680px;
  align-items: center;
}

.table-editor-grid input,
.table-editor-grid select,
.table-editor-grid button,
.table-editor-grid output {
  width: 100%;
  min-width: 0;
}

.table-editor-grid output {
  min-height: 30px;
  padding: 6px 8px;
  border: 1px solid #d8e0e8;
  background: #f5f7fa;
  color: #1b2733;
  font-variant-numeric: tabular-nums;
}

.table-editor-grid span {
  color: #526171;
  font-size: 12px;
}

.editor-pane {
  background: #ffffff;
}

.editor-host {
  height: 100%;
}

.preview-pane {
  background: #ffffff;
}

.preview-document {
  max-width: 900px;
  margin: 0 auto;
  padding: 48px;
  color: #1f2937;
  line-height: 1.65;
}

.preview-document h1,
.preview-document h2,
.preview-document h3 {
  color: #111827;
  line-height: 1.2;
}

.preview-document table,
.transform-table {
  width: 100%;
  border-collapse: collapse;
}

.preview-document td,
.preview-document th,
.transform-table td,
.transform-table th {
  border: 1px solid #c9d2dc;
  padding: 6px 8px;
}

.preview-document .glossary-term {
  border-bottom: 1px dotted #275da8;
  color: #174a8c;
  cursor: help;
  text-decoration: none;
}

.preview-document .glossary-term:focus {
  outline: 2px solid #275da8;
  outline-offset: 2px;
}

.preview-document .citation {
  color: #174a8c;
  cursor: help;
  font-weight: 600;
}

.preview-document .citation:focus {
  outline: 2px solid #275da8;
  outline-offset: 2px;
}

.preview-document pre,
.sidebar pre {
  overflow: auto;
  padding: 10px;
  background: #edf1f5;
}

.transform-pending,
.transform-calc {
  padding: 10px;
  border: 1px solid #c9d2dc;
  background: #f7f9fb;
}

.status-bar {
  justify-content: space-between;
  border-top: 1px solid #c9d2dc;
  border-bottom: 0;
}

.conflict-actions {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.error {
  color: #c24141;
}

.modal-backdrop {
  position: fixed;
  inset: 0;
  display: grid;
  place-items: center;
  padding: 24px;
  background: rgba(15, 23, 42, 0.38);
  z-index: 20;
}

.modal {
  display: grid;
  gap: 12px;
  width: min(720px, 100%);
  max-height: min(760px, 92vh);
  overflow: auto;
  padding: 16px;
  border: 1px solid #bac4d1;
  border-radius: 8px;
  background: #f7f9fb;
  box-shadow: 0 20px 60px rgba(15, 23, 42, 0.28);
}

.modal header,
.modal footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.modal h2 {
  margin: 0;
  font-size: 18px;
}

.modal textarea,
.modal input {
  width: 100%;
  border: 1px solid #bac4d1;
  border-radius: 6px;
  padding: 10px;
  color: #18212f;
  background: #ffffff;
}

.command-modal {
  align-content: start;
}

.command-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 16px;
  width: 100%;
  padding: 8px 10px;
  text-align: left;
}

.command-row span {
  color: #526171;
  font-size: 12px;
}

.conflict-modal {
  width: min(1100px, 100%);
}

.compare-grid {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 12px;
  min-height: 0;
}

.compare-grid pre {
  max-height: 420px;
  overflow: auto;
  padding: 10px;
  background: #edf1f5;
}

@media (max-width: 900px) {
  .workspace,
  .workspace.mode-source,
  .workspace.mode-focus,
  .workspace.mode-preview,
  .workspace.mode-export {
    grid-template-columns: 1fr;
  }

  .sidebar {
    display: none;
  }

  .preview-document {
    padding: 24px;
  }

  .compare-grid {
    grid-template-columns: 1fr;
  }
}
</style>
