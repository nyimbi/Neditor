<template>
  <aside
    v-show="store.mode !== 'outline' && !writingSpaceMaximized && !store.zenMode"
    id="document-sidebar"
    :key="store.sidebar"
    :data-sidebar="store.sidebar"
    :data-sidebar-layout="store.sidebarLayout"
    class="sidebar"
    aria-label="Document workspace"
    tabindex="-1"
  >
    <!-- ── Tab strip (tabs layout) ─────────────────────────────────────────── -->
    <nav
      v-if="store.sidebarLayout === 'tabs'"
      class="sidebar-tab-strip"
      aria-label="Sidebar panels"
    >
      <button
        v-for="tab in SIDEBAR_TABS"
        :key="tab.id"
        type="button"
        class="sidebar-tab"
        :class="{ 'sidebar-tab--active': effectiveTab === tab.id }"
        :aria-current="effectiveTab === tab.id ? 'true' : undefined"
        :aria-label="tab.label"
        @click="selectTab(tab.id)"
      >{{ tab.label }}</button>
    </nav>

    <!-- ── Activity-bar layout chrome ─────────────────────────────────────── -->
    <div
      v-if="store.uiMode === 'pilot' && store.sidebarLayout !== 'tabs'"
      class="sidebar-resize-handle"
      title="Drag to resize"
      @mousedown.prevent="onSidebarResizeStart"
    ></div>
    <button
      v-if="store.sidebarLayout !== 'tabs'"
      type="button"
      class="sidebar-collapse-btn"
      :title="sidebarCollapsed ? 'Expand sidebar (⌘B)' : 'Collapse sidebar (⌘B)'"
      :aria-label="sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'"
      @click="toggleSidebarCollapsed()"
    >{{ sidebarCollapsed ? '▶' : '◀' }}</button>
    <header v-if="store.uiMode === 'pilot' && store.sidebarLayout !== 'tabs'" class="sidebar-panel-header" aria-label="Current panel">
      <span class="sidebar-panel-name">{{ currentPanelLabel }}</span>
    </header>

    <!-- ── Tab-mode panel header ───────────────────────────────────────────── -->
    <div v-if="store.sidebarLayout === 'tabs' && effectiveTab" class="sidebar-tab-content-header">
      <span class="sidebar-tab-content-title">{{ PANEL_LABELS[effectiveTab] || effectiveTab }}</span>
      <button type="button" class="sidebar-tab-more" aria-label="More options" title="More options">···</button>
    </div>

    <FilesPanel v-if="store.sidebar === 'files'" />

    <OutlinePanel v-else-if="store.sidebar === 'outline'" />

    <template v-else-if="store.sidebar === 'diagnostics'">
      <h2>Diagnostics</h2>
      <section class="compiler-output-inventory" aria-label="Compiler output inventory">
        <article v-for="item in compilerOutputInventory" :key="item.label" class="snapshot-row" :data-status="item.status">
          <p>{{ item.label }}</p>
          <small>{{ item.status }} | {{ item.detail }}</small>
        </article>
      </section>
      <section role="list" aria-label="Compiler diagnostics">
        <article
          v-for="diagnostic in active.compile?.diagnostics || []"
          :key="`${diagnostic.severity}-${diagnostic.source_file || ''}-${diagnostic.line || ''}-${diagnostic.column || ''}-${diagnostic.message}`"
          class="diagnostic"
          :class="diagnostic.severity"
          role="listitem"
          :aria-label="diagnosticAnnouncementLabel(diagnostic)"
        >
          <strong>{{ diagnostic.severity }}</strong>
          <p>{{ diagnostic.message }}</p>
          <small v-if="diagnosticLocation(diagnostic)">{{ diagnosticLocation(diagnostic) }}</small>
          <small v-if="diagnostic.suggestion">{{ diagnostic.suggestion }}</small>
          <ul v-if="diagnostic.related.length" class="diagnostic-related">
            <li v-for="related in diagnostic.related" :key="related">{{ related }}</li>
          </ul>
          <button v-if="canNavigateDiagnostic(diagnostic)" type="button" @click="goToSourceTarget(diagnostic)">Go to source</button>
        </article>
      </section>
    </template>

    <LayoutPanel v-else-if="store.sidebar === 'layout'" />

    <TablesPanel v-else-if="store.sidebar === 'tables'" />

    <TemplatesPanel v-else-if="store.sidebar === 'templates'" />

    <ReferencesPanel v-else-if="store.sidebar === 'references'" />

    <ExportsPanel v-else-if="store.sidebar === 'exports'" />

    <VersioningPanel v-else-if="store.sidebar === 'versioning'" />

    <ReviewPanel v-else-if="store.sidebar === 'review'" />

    <!-- ── Backlinks panel ──────────────────────────────────────────────── -->
    <BacklinksPanel v-else-if="store.sidebar === 'backlinks'" />

    <!-- ── Tasks panel ──────────────────────────────────────────────────── -->
    <template v-else-if="store.sidebar === 'tasks'">
      <h2>Tasks</h2>
      <div class="sidebar-toolbar tasks-toolbar">
        <button type="button" @click="refreshWorkspaceTasks" :disabled="tasksLoading" title="Refresh tasks">↻</button>
        <button type="button" :class="{ active: tasksFilterStatus === 'all' }" @click="tasksFilterStatus = 'all'">All</button>
        <button type="button" :class="{ active: tasksFilterStatus === 'todo' }" @click="tasksFilterStatus = 'todo'">Todo</button>
        <button type="button" :class="{ active: tasksFilterStatus === 'done' }" @click="tasksFilterStatus = 'done'">Done</button>
      </div>
      <div class="tasks-tag-filter" v-if="allTaskTags.length">
        <select v-model="tasksFilterTag" aria-label="Filter by tag">
          <option value="">All tags</option>
          <option v-for="tag in allTaskTags" :key="tag" :value="tag">#{{ tag }}</option>
        </select>
      </div>
      <section class="tasks-panel" aria-label="Workspace tasks">
        <div v-if="tasksLoading" class="sidebar-loading">Scanning workspace…</div>
        <template v-else>
          <p v-if="!filteredTasks.length" class="sidebar-hint">
            {{ workspaceTasks.length ? 'No tasks match the current filter.' : 'No checkboxes found in workspace. Use - [ ] to create tasks.' }}
          </p>
          <div class="task-group" v-for="(groupTasks, groupKey) in _groupTasksByFile(filteredTasks)" :key="String(groupKey)">
            <h4 class="task-group-header">{{ String(groupKey).split('/').pop() }}</h4>
            <label
              v-for="task in groupTasks as typeof filteredTasks"
              :key="task.file_path + ':' + task.line"
              class="task-item"
              :class="{ done: task.done }"
            >
              <input type="checkbox" :checked="task.done" disabled />
              <span class="task-text">{{ task.text }}</span>
              <span v-if="task.due_date" class="task-due">{{ task.due_date }}</span>
              <button type="button" class="task-goto" @click="store.openPath(task.file_path)" title="Open file">→</button>
            </label>
          </div>
        </template>
      </section>
    </template>

    <!-- ── Daily Notes panel ────────────────────────────────────────────── -->
    <template v-else-if="store.sidebar === 'daily-notes'">
      <h2>Daily Notes</h2>
      <div class="sidebar-toolbar">
        <button type="button" @click="openTodayNote" class="primary" title="Open today's note (⌘⇧D)">Today</button>
      </div>
      <section class="daily-notes-panel" aria-label="Daily notes calendar">
        <div class="daily-notes-calendar-nav">
          <button type="button" @click="if (dailyNotesCalendarMonth === 1) { dailyNotesCalendarYear--; dailyNotesCalendarMonth = 12; } else dailyNotesCalendarMonth--">‹</button>
          <span>{{ ['Jan','Feb','Mar','Apr','May','Jun','Jul','Aug','Sep','Oct','Nov','Dec'][dailyNotesCalendarMonth - 1] }} {{ dailyNotesCalendarYear }}</span>
          <button type="button" @click="if (dailyNotesCalendarMonth === 12) { dailyNotesCalendarYear++; dailyNotesCalendarMonth = 1; } else dailyNotesCalendarMonth++">›</button>
        </div>
        <div class="daily-notes-calendar-grid" role="grid" aria-label="Daily notes calendar">
          <div class="cal-dow" v-for="d in ['Su','Mo','Tu','We','Th','Fr','Sa']" :key="d">{{ d }}</div>
          <button
            v-for="cell in dailyNotesCalendarGrid"
            :key="cell.date || 'empty-' + cell.day"
            type="button"
            :class="['cal-day', { 'cal-empty': cell.empty, 'cal-has-note': cell.hasNote, 'cal-today': cell.isToday }]"
            :disabled="cell.empty"
            :aria-label="cell.date ? `Open note for ${cell.date}` : undefined"
            @click="cell.date && openDailyNoteForDate(cell.date)"
          >{{ cell.day || '' }}</button>
        </div>
        <p class="sidebar-hint" v-if="!store.workspaceRoot">Open a workspace folder to enable daily notes.</p>
      </section>
    </template>
    <template v-if="store.sidebar === 'help'">
      <slot name="help-panel" />
    </template>
    <template v-else-if="store.sidebar === 'settings'">
      <slot name="settings-panel" />
    </template>
  </aside>
</template>

<script setup lang="ts">
import { computed, inject } from 'vue';
import FilesPanel from '../panels/sidebar/FilesPanel.vue';
import OutlinePanel from '../panels/sidebar/OutlinePanel.vue';
import LayoutPanel from '../panels/sidebar/LayoutPanel.vue';
import TablesPanel from '../panels/sidebar/TablesPanel.vue';
import TemplatesPanel from '../panels/sidebar/TemplatesPanel.vue';
import ReferencesPanel from '../panels/sidebar/ReferencesPanel.vue';
import ExportsPanel from '../panels/sidebar/ExportsPanel.vue';
import VersioningPanel from '../panels/sidebar/VersioningPanel.vue';
import ReviewPanel from '../panels/sidebar/ReviewPanel.vue';
import BacklinksPanel from '../panels/sidebar/BacklinksPanel.vue';
import { useDocumentsStore } from '../stores/documents';
import type { SidebarPanel } from '../lib/workspacePersistence';

const props = defineProps<{
  writingSpaceMaximized: boolean;
  sidebarCollapsed: boolean;
  sidebarWidth: number;
}>();

const emit = defineEmits<{
  'toggle-sidebar-collapsed': [];
  'update:sidebarWidth': [val: number];
}>();

const store = useDocumentsStore();

const _ctx = inject('sidebarCtx') as Record<string, any>;
const {
  active,
  allTaskTags,
  canNavigateDiagnostic,
  compilerOutputInventory,
  dailyNotesCalendarGrid,
  dailyNotesCalendarMonth,
  dailyNotesCalendarYear,
  diagnosticAnnouncementLabel,
  diagnosticLocation,
  filteredTasks,
  goToSourceTarget,
  openDailyNoteForDate,
  openTodayNote,
  refreshWorkspaceTasks,
  sidebarCollapsed,
  tasksFilterStatus,
  tasksFilterTag,
  tasksLoading,
  workspaceTasks,
  writingSpaceMaximized,
} = _ctx;

const PANEL_LABELS: Record<string, string> = {
  files: 'Files', outline: 'Outline', diagnostics: 'Diagnostics', layout: 'Layout',
  tables: 'Tables', templates: 'Templates', references: 'References',
  exports: 'Export', versioning: 'Versioning', review: 'Review',
  backlinks: 'Backlinks', tasks: 'Tasks', 'daily-notes': 'Daily Notes',
  help: 'Help', settings: 'Settings',
};

const SIDEBAR_TABS = Object.entries(PANEL_LABELS).map(([id, label]) => ({ id, label }));

const currentPanelLabel = computed(() => PANEL_LABELS[store.sidebar as string] || String(store.sidebar));

/** In tabs mode: the active tab; falls back to store.sidebar if activeSidebarTab not yet set. */
const effectiveTab = computed(() =>
  store.sidebarLayout === 'tabs' ? (store.activeSidebarTab || store.sidebar) : store.sidebar
);

function selectTab(panelId: string): void {
  store.activeSidebarTab = panelId;
  store.sidebar = panelId as SidebarPanel;
}

function onSidebarResizeStart(event: MouseEvent): void {
  const startX = event.clientX;
  const startW = props.sidebarWidth;
  const onMove = (e: MouseEvent) => {
    emit('update:sidebarWidth', Math.max(160, Math.min(480, startW + (e.clientX - startX))));
  };
  const onUp = () => {
    window.removeEventListener('mousemove', onMove);
    window.removeEventListener('mouseup', onUp);
  };
  window.addEventListener('mousemove', onMove);
  window.addEventListener('mouseup', onUp);
}

function toggleSidebarCollapsed(): void {
  emit('toggle-sidebar-collapsed');
}

type _WorkspaceTask = { file_path: string; line: number; text: string; done: boolean; tags: string[]; due_date: string | null; heading_context: string };
function _groupTasksByFile(tasks: _WorkspaceTask[]): Record<string, _WorkspaceTask[]> {
  return tasks.reduce((acc: Record<string, _WorkspaceTask[]>, t) => {
    (acc[t.file_path] = acc[t.file_path] || []).push(t);
    return acc;
  }, {});
}

</script>
