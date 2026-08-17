<template>
  <!-- Item D: Toast notification host -->
  <aside
    v-if="toasts.visible.length || toasts.overflowCount > 0"
    class="toast-host"
    aria-label="Notifications"
    aria-live="polite"
    aria-atomic="false"
  >
    <p v-if="toasts.overflowCount > 0" class="toast-overflow">+{{ toasts.overflowCount }} more</p>
    <article
      v-for="toast in toasts.visible"
      :key="toast.id"
      class="toast-item"
      :class="`toast-${toast.kind}`"
      role="alert"
    >
      <div class="toast-content">
        <strong class="toast-title">{{ toast.title }}</strong>
        <span v-if="toast.body" class="toast-body">{{ toast.body }}</span>
      </div>
      <div class="toast-actions">
        <button
          v-if="toast.actionLabel && toast.onAction"
          type="button"
          class="toast-action-btn"
          @click="() => { toast.onAction?.(); toasts.dismiss(toast.id); }"
        >{{ toast.actionLabel }}</button>
        <button
          type="button"
          class="toast-dismiss-btn"
          :aria-label="`Dismiss: ${toast.title}`"
          @click="toasts.dismiss(toast.id)"
        >x</button>
      </div>
    </article>
  </aside>
</template>

<script setup lang="ts">
import { useToasts } from "../lib/toasts";

const toasts = useToasts();
</script>
