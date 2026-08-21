<template>
  <div class="collapsible-advanced">
    <button
      :id="headerId"
      type="button"
      class="collapsible-advanced-header"
      :aria-expanded="expanded ? 'true' : 'false'"
      :aria-controls="bodyId"
      @click="toggle"
    >
      <svg
        class="collapsible-chevron"
        :class="{ 'collapsible-chevron--open': expanded }"
        viewBox="0 0 24 24"
        focusable="false"
        aria-hidden="true"
      >
        <path d="M9 6l6 6-6 6" />
      </svg>
      <span class="collapsible-label">{{ label }}<template v-if="count !== undefined"> ({{ count }})</template></span>
    </button>
    <div
      :id="bodyId"
      class="collapsible-advanced-body"
      :class="{ 'collapsible-advanced-body--open': expanded }"
      role="region"
      :aria-labelledby="headerId"
    >
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted } from 'vue';
import { useDocumentsStore } from '../stores/documents';

const props = withDefaults(defineProps<{
  panelId: string;
  label?: string;
  count?: number;
}>(), {
  label: 'Advanced',
});

const store = useDocumentsStore();

const headerId = computed(() => `collapsible-adv-header-${props.panelId}`);
const bodyId = computed(() => `collapsible-adv-body-${props.panelId}`);

const expanded = computed(() => store.getAdvancedExpanded(props.panelId));

function toggle() {
  store.setAdvancedExpanded(props.panelId, !expanded.value);
}
</script>

<style scoped>
.collapsible-advanced {
  margin-top: 0.5rem;
}

.collapsible-advanced-header {
  display: flex;
  align-items: center;
  gap: 0.35rem;
  width: 100%;
  background: none;
  border: none;
  border-top: 1px solid var(--sidebar-border, #e0e0e0);
  padding: 0.45rem 0.5rem;
  cursor: pointer;
  font: inherit;
  font-size: 0.8rem;
  font-weight: 600;
  color: var(--sidebar-fg, inherit);
  text-align: left;
  letter-spacing: 0.03em;
  text-transform: uppercase;
}

.collapsible-advanced-header:focus-visible {
  outline: 2px solid var(--focus-ring, #005fcc);
  outline-offset: -2px;
}

.collapsible-chevron {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
  stroke: currentColor;
  fill: none;
  stroke-width: 2;
  stroke-linecap: round;
  stroke-linejoin: round;
  transition: transform 0.18s ease;
  transform: rotate(0deg);
}

.collapsible-chevron--open {
  transform: rotate(90deg);
}

@media (prefers-reduced-motion: reduce) {
  .collapsible-chevron {
    transition: none;
  }
}

.collapsible-advanced-body {
  display: none;
  overflow: hidden;
}

.collapsible-advanced-body--open {
  display: block;
  animation: collapsible-expand 0.18s ease;
}

@keyframes collapsible-expand {
  from { opacity: 0; }
  to   { opacity: 1; }
}

@media (prefers-reduced-motion: reduce) {
  .collapsible-advanced-body--open {
    animation: none;
  }
}
</style>
