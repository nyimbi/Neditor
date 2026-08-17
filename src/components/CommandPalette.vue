<template>
  <section
    ref="dialogEl"
    class="modal-backdrop"
    role="dialog"
    aria-modal="true"
    aria-label="Command palette"
    tabindex="-1"
    @keydown="handleKeydown"
  >
    <div class="modal command-modal">
      <header>
        <h2>Command Palette</h2>
        <button type="button" aria-label="Close command palette" @click="emit('close')">x</button>
      </header>
      <input
        :value="modelValue"
        autofocus
        data-initial-focus
        aria-label="Search commands, headings, citations, glossary, index terms, or enter an AI instruction"
        placeholder="Search commands, headings, citations, glossary, index terms"
        @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
        @keydown.enter.prevent="emit('agent-instruction')"
      />
      <button
        v-for="command in filteredCommands"
        :key="command.name"
        class="command-row"
        type="button"
        :aria-label="`${command.name} ${command.group}`"
        @click="emit('run-command', command.run)"
      >
        <span class="command-row-main">
          <strong>{{ command.name }}</strong>
          <small v-if="command.description">{{ command.description }}</small>
        </span>
        <span>{{ command.group }}</span>
      </button>
      <section v-if="commandAgentInstructionAvailable" class="command-agent-route" aria-label="AI command route">
        <div>
          <strong>Generate with AI agent</strong>
          <span>Plan the workflow, create a governed packet, and keep it ready for review or distribution.</span>
          <dl v-if="commandAgentPlanPreview" class="command-agent-preview" aria-label="AI command plan preview">
            <div>
              <dt>Lanes</dt>
              <dd>{{ commandAgentPlanPreview.lanes.join(", ") }}</dd>
            </div>
            <div>
              <dt>Targets</dt>
              <dd>{{ commandAgentPlanPreview.distributionTargets.length ? commandAgentPlanPreview.distributionTargets.join(", ") : "Review packet" }}</dd>
            </div>
            <div>
              <dt>Missing</dt>
              <dd>{{ commandAgentPlanPreview.missingInputs.length ? commandAgentPlanPreview.missingInputs.slice(0, 4).join(", ") : "Ready to draft" }}</dd>
            </div>
          </dl>
          <div v-if="commandAgentRouteSuggestions.length" class="command-agent-routes" role="region" aria-label="AI command route suggestions">
            <button
              v-for="route in commandAgentRouteSuggestions"
              :key="route.id"
              type="button"
              :title="route.detail"
              @click="emit('agent-route', route.id)"
            >
              {{ route.label }}
            </button>
          </div>
        </div>
        <div class="command-agent-actions">
          <button type="button" @click="emit('agent-plan')">Plan first</button>
          <button type="button" @click="emit('agent-instruction')">Generate Packet</button>
        </div>
      </section>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useDocumentsStore } from "../stores/documents";
import { commandSearchText, type CommandPaletteSearchable } from "../lib/commandPalette";
import { buildAgenticWorkflowPlan } from "../lib/agenticWorkflows";

interface CommandPaletteCommand extends CommandPaletteSearchable {
  run: () => unknown;
}

interface CommandAgentRouteSuggestion {
  id: string;
  label: string;
  detail: string;
}

const props = defineProps<{
  modelValue: string;
  commands: CommandPaletteCommand[];
  selectedText: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [query: string];
  close: [];
  "run-command": [run: () => unknown];
  "agent-instruction": [];
  "agent-plan": [];
  "agent-route": [routeId: string];
}>();

const store = useDocumentsStore();
const dialogEl = ref<HTMLElement | null>(null);
let returnFocusEl: HTMLElement | null = null;

const filteredCommands = computed(() => {
  const query = props.modelValue.trim().toLowerCase();
  if (!query) return props.commands;
  return props.commands.filter((command) => commandSearchText(command).includes(query));
});

const commandAgentInstructionAvailable = computed(() => {
  const query = props.modelValue.trim();
  if (query.length < 8) return false;
  return /\b(ai|agent|create|draft|write|revise|edit|review|summari[sz]e|publish|export|prepare|make|turn|improve|humanize|outline|compose|research|report|source|citation)\b/i.test(query);
});

const commandAgentPlanPreview = computed(() => {
  const instruction = props.modelValue.trim();
  if (!commandAgentInstructionAvailable.value) return null;
  const active = store.activeDocument;
  return buildAgenticWorkflowPlan({
    instruction,
    documentTitle: (active as any).compile?.semantic?.title || active.title,
    documentText: active.text,
    selectedText: props.selectedText,
  });
});

const commandAgentRouteSuggestions = computed<CommandAgentRouteSuggestion[]>(() => {
  const instruction = props.modelValue.trim().toLowerCase();
  if (!commandAgentInstructionAvailable.value) return [];
  const candidates: Array<CommandAgentRouteSuggestion & { rank: number }> = [
    {
      id: "docs-live",
      label: "Docs Live",
      detail: "Open voice/context drafting with the current instruction as the starting brief.",
      rank: /\b(create|draft|write|compose|section|voice|dictate|first draft)\b/.test(instruction) ? 0 : 3,
    },
    {
      id: "ai-paste",
      label: "AI Paste cleanup",
      detail: "Open cleanup for pasted chat output, provenance, citations, and insertion mode.",
      rank: /\b(paste|cleanup|clean up|chat output|clipboard|ai text)\b/.test(instruction) ? 0 : 5,
    },
    {
      id: "deep-research",
      label: "Deep Research",
      detail: "Open source search and iterative report generation with a selected page target.",
      rank: /\b(deep research|research report|source search|citation search|local source library|duckduckgo|searxng|tavily|200 pages?)\b/.test(instruction) ? 0 : 3,
    },
    {
      id: "review",
      label: "Review governance",
      detail: "Open review, provenance, comments, AI markers, and readiness blockers.",
      rank: /\b(review|qa|quality|citation|claim|humanize|governance|approve|risk)\b/.test(instruction) ? 0 : 4,
    },
    {
      id: "export",
      label: "Export readiness",
      detail: "Open target-aware export readiness, manifests, publishing packages, and distribution evidence.",
      rank: /\b(export|publish|distribut|blog|substack|google docs|latex|epub|ebook|html|pdf|docx|pptx)\b/.test(instruction) ? 0 : 4,
    },
    {
      id: "outline",
      label: "Outline mode",
      detail: "Open outline-first planning for chapters, sections, subsections, and drafting queues.",
      rank: /\b(outline|structure|plan|chapter|section|toc)\b/.test(instruction) ? 0 : 4,
    },
    {
      id: "provider",
      label: "Provider handoff",
      detail: "Open the Agent Workspace, generate a governed packet, and build a redacted provider request.",
      rank: /\b(provider|model|openai|anthropic|gemini|antigravity|ollama|local gateway|handoff|run ai)\b/.test(instruction) ? 0 : 4,
    },
  ];
  return candidates
    .sort((left, right) => left.rank - right.rank || left.label.localeCompare(right.label))
    .slice(0, 4)
    .map(({ rank: _rank, ...route }) => route);
});

function focusableElements(): HTMLElement[] {
  if (!dialogEl.value) return [];
  return Array.from(
    dialogEl.value.querySelectorAll<HTMLElement>(
      ["a[href]", "button:not([disabled])", "input:not([disabled])", "select:not([disabled])", "textarea:not([disabled])", "[tabindex]:not([tabindex='-1'])"].join(","),
    ),
  ).filter((el) => !el.hasAttribute("disabled") && el.offsetParent !== null);
}

function handleKeydown(event: KeyboardEvent): void {
  if (event.key === "Escape") {
    event.preventDefault();
    emit("close");
    return;
  }
  if (event.key !== "Tab") return;
  const focusable = focusableElements();
  if (!focusable.length) {
    event.preventDefault();
    dialogEl.value?.focus({ preventScroll: true });
    return;
  }
  const first = focusable[0];
  const last = focusable[focusable.length - 1];
  const activeEl = document.activeElement;
  if (event.shiftKey && activeEl === first) {
    event.preventDefault();
    last.focus({ preventScroll: true });
  } else if (!event.shiftKey && activeEl === last) {
    event.preventDefault();
    first.focus({ preventScroll: true });
  }
}

onMounted(() => {
  returnFocusEl = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const initial = dialogEl.value?.querySelector<HTMLElement>("[data-initial-focus]");
  const target = initial || focusableElements()[0] || dialogEl.value;
  target?.focus({ preventScroll: true });
});

onBeforeUnmount(() => {
  if (returnFocusEl?.isConnected) {
    returnFocusEl.focus({ preventScroll: true });
  }
  returnFocusEl = null;
});
</script>
