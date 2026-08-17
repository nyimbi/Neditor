<template>
  <h2>Help Center</h2>
  <section class="help-center" aria-label="Help center">
    <div class="help-controls">
      <label>
        Search help
        <input :value="helpQuery" type="search" placeholder="export, outline, voice, shortcut" @input="$emit('update:help-query', ($event.target as HTMLInputElement).value)" />
      </label>
      <label>
        Area
        <select :value="helpCategory" @change="$emit('update:help-category', ($event.target as HTMLSelectElement).value)">
          <option value="all">All areas</option>
          <option v-for="category in helpCategoryOptions" :key="category.id" :value="category.id">{{ category.label }}</option>
        </select>
      </label>
    </div>
    <div class="help-quick-actions" aria-label="Popular help actions">
      <button type="button" @click="$emit('open-start-workspace')">Start Workspace</button>
      <button type="button" @click="$emit('open-help', 'docs-live')">Docs Live</button>
      <button type="button" @click="$emit('open-help', 'agent-lifecycle-governance')">AI Governance</button>
      <button type="button" @click="$emit('open-guided-demo')">Guided demo</button>
      <button type="button" @click="$emit('open-help', 'export-publishing')">Export</button>
      <button type="button" @click="$emit('open-help', 'keyboard-shortcuts')">Shortcuts</button>
    </div>
    <section class="start-workspace-cockpit" aria-label="Start Workspace cockpit">
      <header>
        <div>
          <strong>Start Workspace</strong>
          <span>{{ startWorkspaceSummary }}</span>
        </div>
        <button
          type="button"
          title="Insert the current onboarding checklist into the active document for review, delegation, or handoff"
          @click="$emit('insert-start-workspace-checklist')"
        >
          Insert checklist
        </button>
      </header>
      <ol class="start-workspace-steps" aria-label="Workspace setup and creation steps">
        <li v-for="item in startWorkspaceItems" :key="item.id" :class="{ complete: item.done }">
          <div>
            <strong>{{ item.label }}</strong>
            <span>{{ item.status }}</span>
            <small>{{ item.detail }}</small>
          </div>
          <button type="button" :disabled="item.disabled" :title="item.title" @click="$emit('run-start-workspace-action', item)">
            {{ item.actionLabel }}
          </button>
        </li>
      </ol>
    </section>
    <section class="help-topic-list" role="list" aria-label="Help topics">
      <div v-for="topic in filteredHelpTopics" :key="topic.id" role="listitem">
        <button
          class="help-topic-button"
          :class="{ active: topic.id === selectedHelpTopic?.id }"
          type="button"
          @click="$emit('select-help-topic', topic.id)"
        >
          <strong>{{ topic.title }}</strong>
          <small>{{ topic.summary }}</small>
        </button>
      </div>
    </section>
    <p v-if="!filteredHelpTopics.length" class="sidebar-hint">No help topics matched that search.</p>
    <article v-if="selectedHelpTopic" class="help-topic-detail" aria-label="Selected help topic">
      <div class="help-topic-header">
        <small>{{ helpCategoryLabel(selectedHelpTopic.category) }}</small>
        <h3>{{ selectedHelpTopic.title }}</h3>
        <p>{{ selectedHelpTopic.summary }}</p>
      </div>
      <p class="help-when">{{ selectedHelpTopic.when }}</p>
      <ol class="help-steps">
        <li v-for="step in selectedHelpTopic.steps" :key="step">{{ step }}</li>
      </ol>
      <ul class="help-tips">
        <li v-for="tip in selectedHelpTopic.tips" :key="tip">{{ tip }}</li>
      </ul>
      <div class="help-action-row">
        <button v-for="action in selectedHelpTopic.actions" :key="action.label" type="button" @click="$emit('run-help-action', action)">
          {{ action.label }}
        </button>
      </div>
      <div class="help-keywords" aria-label="Topic keywords">
        <span v-for="keyword in selectedHelpTopic.keywords" :key="keyword">{{ keyword }}</span>
      </div>
    </article>
  </section>
</template>

<script setup lang="ts">
type HelpCategory = "basics" | "writing" | "structure" | "content" | "review" | "export" | "settings";

interface HelpTopicAction {
  label: string;
  run: () => unknown;
}

interface HelpTopic {
  id: string;
  title: string;
  category: HelpCategory;
  summary: string;
  when: string;
  steps: string[];
  tips: string[];
  actions: HelpTopicAction[];
  keywords: string[];
}

type StartWorkspaceActionId = "identity" | "setup" | "wizard" | "docs-live" | "templates" | "demo" | "export" | "cli";

interface StartWorkspaceItem {
  id: StartWorkspaceActionId;
  label: string;
  status: string;
  detail: string;
  actionLabel: string;
  title: string;
  done: boolean;
  disabled?: boolean;
  run: () => unknown;
}

const helpCategoryOptions: { id: HelpCategory; label: string }[] = [
  { id: "basics", label: "Basics" },
  { id: "writing", label: "Writing" },
  { id: "structure", label: "Structure" },
  { id: "content", label: "Content blocks" },
  { id: "review", label: "Review" },
  { id: "export", label: "Export" },
  { id: "settings", label: "Settings" },
];

defineProps<{
  helpQuery: string;
  helpCategory: string;
  startWorkspaceSummary: string;
  startWorkspaceItems: StartWorkspaceItem[];
  filteredHelpTopics: HelpTopic[];
  selectedHelpTopic: HelpTopic | null;
}>();

defineEmits<{
  'update:help-query': [value: string];
  'update:help-category': [value: string];
  'open-start-workspace': [];
  'open-help': [topicId: string];
  'open-guided-demo': [];
  'insert-start-workspace-checklist': [];
  'run-start-workspace-action': [item: StartWorkspaceItem];
  'select-help-topic': [topicId: string];
  'run-help-action': [action: HelpTopicAction];
}>();

function helpCategoryLabel(category: HelpCategory): string {
  return helpCategoryOptions.find((option) => option.id === category)?.label || "Help";
}
</script>
