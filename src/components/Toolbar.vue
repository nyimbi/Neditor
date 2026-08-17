<template>
  <nav v-show="!store.zenMode" id="main-commands" class="command-bar" aria-label="Main commands" tabindex="-1">
    <section
      v-for="row in commandToolbarRows"
      v-show="!isToolbarCollapsed(row.id)"
      :key="row.id"
      class="command-toolbar-row"
      :data-row-id="row.id"
      :aria-label="`${row.label} toolbar`"
    >
      <button
        class="command-toolbar-heading"
        type="button"
        :aria-label="`${isToolbarCollapsed(row.id) ? 'Expand' : 'Collapse'} ${row.label} toolbar`"
        :aria-expanded="!isToolbarCollapsed(row.id)"
        @click="$emit('toggle-toolbar-row', row.id)"
      >
        <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
          <path v-for="path in toolbarIconPaths(isToolbarCollapsed(row.id) ? 'expand' : 'collapse')" :key="path" :d="path"></path>
        </svg>
        <span>{{ row.label }}</span>
      </button>
      <section v-for="group in row.groups" v-show="!isToolbarCollapsed(row.id)" :key="group.id" class="command-group" :aria-label="`${group.label} commands`">
        <span class="command-group-label">{{ group.label }}</span>
        <div class="command-group-actions">
          <button
            v-for="action in group.actions"
            :key="action.id"
            type="button"
            class="icon-command"
            :class="{ primary: action.primary }"
            :disabled="action.disabled"
            :aria-label="action.label"
            :title="action.title || action.label"
            @click="runCommandBarAction(action)"
          >
            <span class="command-icon" aria-hidden="true">
              <svg viewBox="0 0 24 24" focusable="false">
                <path v-for="path in toolbarIconPaths(action.icon)" :key="path" :d="path"></path>
              </svg>
            </span>
            <span class="command-label">{{ action.label }}</span>
          </button>
        </div>
      </section>
    </section>
    <section v-show="!isToolbarCollapsed('view')" class="command-toolbar-row command-toolbar-row-view" aria-label="View toolbar">
      <button
        class="command-toolbar-heading"
        type="button"
        :aria-label="`${isToolbarCollapsed('view') ? 'Expand' : 'Collapse'} View toolbar`"
        :aria-expanded="!isToolbarCollapsed('view')"
        @click="$emit('toggle-toolbar-row', 'view')"
      >
        <svg viewBox="0 0 24 24" focusable="false" aria-hidden="true">
          <path v-for="path in toolbarIconPaths(isToolbarCollapsed('view') ? 'expand' : 'collapse')" :key="path" :d="path"></path>
        </svg>
        <span>View</span>
      </button>
      <label class="compact-field">
        <span>Mode</span>
        <select v-show="!isToolbarCollapsed('view')" v-model="store.mode" aria-label="View mode">
          <option value="split">Split</option>
          <option value="source">Source</option>
          <option value="preview">Preview</option>
          <option value="focus">Focus</option>
          <option value="outline">Outline</option>
          <option value="export">Export</option>
          <option value="review">Review</option>
          <option value="presentation">Presentation</option>
        </select>
      </label>
      <label class="compact-field">
        <span>Panel</span>
        <select
          v-show="!isToolbarCollapsed('view')"
          :value="store.sidebar"
          aria-label="Sidebar panel"
          @change="$emit('select-sidebar-panel', eventValue($event))"
          @input="$emit('select-sidebar-panel', eventValue($event))"
        >
          <option value="files">Files</option>
          <option value="outline">Outline</option>
          <option value="diagnostics">Diagnostics</option>
          <option value="tables">Tables</option>
          <option value="templates">Templates</option>
          <option value="layout">Layout</option>
          <option value="references">References</option>
          <option value="exports">Exports</option>
          <option value="versioning">Versioning</option>
          <option value="review">Review</option>
          <option value="help">Help</option>
          <option value="settings">Settings</option>
        </select>
      </label>
      <label class="compact-field">
        <span>Buttons</span>
        <select v-show="!isToolbarCollapsed('view')" v-model="store.toolbarDisplay" aria-label="Toolbar button display">
          <option value="both">Icons and text</option>
          <option value="icons">Icons only</option>
          <option value="text">Text only</option>
        </select>
      </label>
      <label class="compact-field compact-field-range">
        <span>Text</span>
        <input
          v-show="!isToolbarCollapsed('view')"
          v-model.number="store.toolbarTextSize"
          aria-label="Toolbar text size"
          type="range"
          min="9"
          max="15"
          step="1"
        />
        <output v-show="!isToolbarCollapsed('view')" aria-label="Current toolbar text size">{{ store.toolbarTextSize }}px</output>
      </label>
      <label v-show="!isToolbarCollapsed('view')" class="compact-check">
        <input v-model="store.splitSourcePanes" type="checkbox" aria-label="Split source editor panes" />
        <span>Dual source</span>
      </label>
      <button v-show="!isToolbarCollapsed('view')" class="compact-toolbar-toggle" type="button" @click="$emit('set-all-collapsed', !anyCommandToolbarsCollapsed)">
        <span class="command-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" focusable="false">
            <path v-for="path in toolbarIconPaths(anyCommandToolbarsCollapsed ? 'expand' : 'collapse')" :key="path" :d="path"></path>
          </svg>
        </span>
        <span>{{ anyCommandToolbarsCollapsed ? "Expand all" : "Collapse all" }}</span>
      </button>
      <button v-show="!isToolbarCollapsed('view')" class="compact-toolbar-toggle" type="button" @click="$emit('toggle-writing-space-maximized')">
        <span class="command-icon" aria-hidden="true">
          <svg viewBox="0 0 24 24" focusable="false">
            <path v-for="path in toolbarIconPaths(writingSpaceMaximized ? 'collapse' : 'expand')" :key="path" :d="path"></path>
          </svg>
        </span>
        <span>{{ writingSpaceMaximized ? "Restore writing" : "Maximize writing" }}</span>
      </button>
    </section>
  </nav>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useDocumentsStore } from "../stores/documents";

interface CommandBarAction {
  id: string;
  label: string;
  title?: string;
  icon: string;
  primary?: boolean;
  disabled?: boolean;
  run: () => unknown;
}

interface CommandBarGroup {
  id: string;
  label: string;
  actions: CommandBarAction[];
}

interface CommandToolbarRow {
  id: string;
  label: string;
  groups: CommandBarGroup[];
}

const props = defineProps<{
  commandToolbarRows: CommandToolbarRow[];
  writingSpaceMaximized: boolean;
  toolbarIconPaths: (icon: string) => string[];
}>();

defineEmits<{
  "toggle-toolbar-row": [id: string];
  "set-all-collapsed": [collapsed: boolean];
  "select-sidebar-panel": [panelId: string];
  "toggle-writing-space-maximized": [];
}>();

const store = useDocumentsStore();

const toolbarCollapseRowIds = computed(() => [...props.commandToolbarRows.map((r) => r.id), "view"]);

const anyCommandToolbarsCollapsed = computed(() => toolbarCollapseRowIds.value.some((id) => store.toolbarCollapsedRows.includes(id)));

function isToolbarCollapsed(id: string): boolean {
  return store.toolbarCollapsedRows.includes(id);
}

async function runCommandBarAction(action: CommandBarAction): Promise<void> {
  if (action.disabled) return;
  try {
    await action.run();
  } catch (error) {
    store.lastError = error instanceof Error ? error.message : String(error);
    store.statusMessage = `${action.label} failed`;
  }
}

function eventValue(event: Event): string {
  return (event.target as HTMLSelectElement | HTMLInputElement)?.value ?? "";
}
</script>
