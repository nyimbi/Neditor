<template>
  <div v-if="presenterViewOpen" class="presenter-modal" role="dialog" aria-modal="true" aria-label="Presenter view">
    <div class="presenter-main">
      <div class="presenter-slide-view" :style="{ background: activeThemeInfo.bg, color: activeThemeInfo.text }">
        <div v-if="currentPresenterSlide" class="presenter-slide-content">
          <h2 class="presenter-slide-h">{{ currentPresenterSlide.title }}</h2>
          <ul class="presenter-slide-bullets">
            <li v-for="line in currentPresenterSlide.lines.slice(0,7)" :key="line">{{ line }}</li>
          </ul>
        </div>
      </div>
      <div class="presenter-notes-area">
        <span class="presenter-label">Speaker notes</span>
        <textarea class="presenter-notes-ta" :value="currentPresenterSlide?.notes || ''" placeholder="Type notes for this slide…" @input="currentPresenterSlide && updateSlideNotes(currentPresenterSlide.title, ($event.target as HTMLTextAreaElement).value)"></textarea>
      </div>
    </div>
    <div class="presenter-panel">
      <div>
        <span class="presenter-label">Next</span>
        <div class="presenter-next-box" :style="{ background: activeThemeInfo.bg }">
          <span style="font-size:12px;opacity:0.7">{{ nextPresenterSlide?.title || '(end)' }}</span>
        </div>
      </div>
      <div class="presenter-slide-list">
        <div v-for="(slide, i) in presentationSlides" :key="i" class="presenter-list-item" :class="{ 'pli-active': i === presenterCurrentIdx }" @click="presenterCurrentIdx = i">
          <span class="pli-num">{{ i+1 }}</span>
          <span class="pli-title">{{ slide.title || '(untitled)' }}</span>
        </div>
      </div>
    </div>
    <div class="presenter-nav">
      <button type="button" :disabled="presenterCurrentIdx === 0" @click="presenterPrev">← Prev</button>
      <span>{{ presenterCurrentIdx + 1 }} / {{ presentationSlides.length }}</span>
      <button type="button" :disabled="presenterCurrentIdx >= presentationSlides.length - 1" @click="presenterNext">Next →</button>
      <button type="button" class="presenter-close" @click="presenterViewOpen = false">✕ Close</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { inject } from 'vue';

const _ctx = inject('presentationModeCtx') as Record<string, any>;
const {
  presenterViewOpen,
  presenterCurrentIdx,
  presentationSlides,
  currentPresenterSlide,
  nextPresenterSlide,
  activeThemeInfo,
  presenterPrev,
  presenterNext,
  updateSlideNotes,
} = _ctx;
</script>
