<template>
  <section v-if="externalTransformTrustPrompts.length" class="trust-prompt" aria-label="External transform trust prompts">
    <article v-for="prompt in externalTransformTrustPrompts" :key="prompt.name" class="trust-prompt-item">
      <div>
        <strong>{{ prompt.name }} transform</strong>
        <span>{{ prompt.path }}</span>
        <small>{{ prompt.inputMode }} | {{ prompt.securitySummary }}</small>
      </div>
      <div class="trust-prompt-actions">
        <button type="button" @click="trustTransformEngine(prompt.name)">Trust</button>
        <button type="button" @click="reviewTransformEngineSettings(prompt.name)">Settings</button>
      </div>
    </article>
  </section>
</template>

<script setup lang="ts">
import { inject } from 'vue';

const _ctx = inject('trustPromptCtx') as Record<string, any>;
const {
  externalTransformTrustPrompts,
  trustTransformEngine,
  reviewTransformEngineSettings,
} = _ctx;
</script>
