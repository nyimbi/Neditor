<template>
  <h2>Outline <small>{{ outlineHeadings.length }}</small></h2>
  <p v-if="!outlineHeadings.length" class="sidebar-hint">Add headings directly or create a document from an outline plan.</p>
  <button
    v-for="heading in outlineHeadings"
    :key="`${heading.line}-${heading.anchor}`"
    class="outline-row"
    :style="{ paddingLeft: `${heading.level * 10}px` }"
    type="button"
    :aria-label="`Go to ${heading.text}, line ${heading.line}`"
    @click="goToSourceTarget(heading)"
  >
    <span>{{ heading.text }}</span>
    <small>Line {{ heading.line }}</small>
  </button>
  <CollapsibleAdvanced panel-id="outline" label="Advanced">
    <section class="outline-planner" aria-label="Outline planner">
      <h3>Plan</h3>
      <label>
        Document title
        <input v-model="outlineDraftTitle" placeholder="Board Brief" />
      </label>
      <label>
        Outline draft
        <textarea
          v-model="outlineDraftText"
          rows="9"
          aria-label="Editable document outline"
          placeholder="- Executive Summary&#10;  - Decision Needed&#10;  - Key Risks&#10;- Financial Case"
        ></textarea>
      </label>
      <label><input v-model="outlineDraftIncludeToc" type="checkbox" /> Include table of contents</label>
      <div class="outline-planner-actions">
        <button type="button" @click="loadOutlineDraftFromDocument">Load from document</button>
        <button type="button" :disabled="!outlineDraftItems.length" @click="openDocsLiveFromOutline">Flesh out with Docs Live</button>
        <button type="button" :disabled="!outlineDraftItems.length" @click="createDocumentFromOutline">Create document from outline</button>
        <button type="button" :disabled="!outlineDraftItems.length" @click="appendOutlineToDocument">Append outline</button>
      </div>
      <p class="sidebar-hint">{{ outlineDraftItems.length }} planned sections. Use indentation, bullets, numbers, or Markdown heading marks.</p>
    </section>
    <section class="document-map" aria-label="Document map">
      <header>
        <div>
          <h3>Document Map</h3>
          <small>{{ documentMapSummary }}</small>
        </div>
        <button
          type="button"
          :class="{ active: documentMapBlockersOnly }"
          title="Focus all unresolved review, citation, metadata, and diagnostic blockers"
          @click="focusDocumentMapBlockers"
        >
          {{ documentMapBlockersOnly ? "All items" : "Blockers" }}
        </button>
      </header>
      <div class="document-map-controls">
        <label>
          Search map
          <input v-model="documentMapQuery" type="search" placeholder="heading, comment, figure, warning" />
        </label>
        <label>
          Show
          <select v-model="documentMapFilter">
            <option v-for="filter in documentMapFilterOptions" :key="filter.id" :value="filter.id">{{ filter.label }}</option>
          </select>
        </label>
      </div>
      <div class="document-map-counts" aria-label="Document map counts">
        <button v-for="filter in documentMapFilterOptions" :key="filter.id" type="button" :class="{ active: documentMapFilter === filter.id }" @click="documentMapFilter = filter.id">
          <strong>{{ documentMapCountByFilter(filter.id) }}</strong>
          <span>{{ filter.shortLabel }}</span>
        </button>
      </div>
      <div v-if="filteredDocumentMapItems.length" class="document-map-list" role="list" aria-label="Navigable document map items">
        <button
          v-for="item in filteredDocumentMapItems"
          :key="item.id"
          type="button"
          role="listitem"
          class="document-map-row"
          :data-kind="item.kind"
          @click="goToDocumentMapItem(item)"
        >
          <span class="document-map-kind">{{ documentMapKindLabel(item.kind) }}</span>
          <span class="document-map-label">{{ item.label }}</span>
          <small>{{ item.detail }}</small>
          <span class="document-map-status">{{ item.status }}</span>
        </button>
      </div>
      <p v-else class="sidebar-hint">No document map items match the current filter.</p>
    </section>
    <section class="outline-library" aria-label="Document outline library">
      <header>
        <div>
          <h3>Outline Library</h3>
          <small>{{ filteredDocumentOutlineTemplates.length }} of {{ allDocumentOutlineTemplates.length }} outlines</small>
        </div>
      </header>
      <div class="outline-library-filters">
        <label>
          Search outlines
          <input v-model="outlineLibraryQuery" type="search" placeholder="RFP, board, textbook, policy" />
        </label>
        <label>
          Category
          <select v-model="outlineLibraryCategory">
            <option v-for="category in outlineLibraryCategories" :key="category" :value="category">{{ category === "all" ? "All categories" : category }}</option>
          </select>
        </label>
      </div>
      <div class="outline-library-actions">
        <button type="button" :disabled="!outlineDraftItems.length" @click="saveCurrentOutlineTemplate">Save planner outline</button>
        <button type="button" @click="resetCustomOutlineDraft">New custom outline</button>
        <button type="button" :disabled="!store.workspaceRoot || workspaceOutlineSyncBusy" @click="syncWorkspaceOutlines">
          {{ workspaceOutlineSyncBusy ? "Syncing..." : "Sync workspace outlines" }}
        </button>
      </div>
      <p class="sidebar-hint">{{ workspaceOutlineSyncStatus || (store.workspaceRoot ? "Workspace outlines sync with .neditor/outlines.json for CLI and app reuse." : "Open a workspace folder to sync outlines with .neditor/outlines.json.") }}</p>
      <label>
        Custom outline name
        <input v-model="customOutlineDraft.name" placeholder="Quarterly business review" />
      </label>
      <label>
        Custom category
        <input v-model="customOutlineDraft.category" placeholder="Executive" />
      </label>
      <label>
        Custom summary
        <input v-model="customOutlineDraft.summary" placeholder="Reusable outline for recurring documents" />
      </label>
      <label>
        Custom tags
        <input v-model="customOutlineTags" placeholder="board, decision, quarterly" />
      </label>
      <label>
        Best for
        <input v-model="customOutlineBestFor" placeholder="Board packs, client reviews, technical chapters" />
      </label>
      <label>
        Docs Live workflow
        <select v-model="customOutlineDraft.docsLiveType">
          <option v-for="type in docsLiveDocumentTypes" :key="type.id" :value="type.id">{{ type.label }}</option>
        </select>
      </label>
      <p class="sidebar-hint">Saving uses the current planner outline so users can adapt a built-in outline and keep it for future documents.</p>
      <div class="outline-template-list" role="list" aria-label="Selectable document outlines">
        <article v-for="template in filteredDocumentOutlineTemplates" :key="`${template.source}-${template.id}`" role="listitem" class="outline-template-card">
          <header>
            <div>
              <strong>{{ template.name }}</strong>
              <small>{{ template.category }} | {{ template.source }} | {{ outlineDocsLiveTypeLabel(template) }} | {{ template.outline.length }} sections</small>
            </div>
            <button type="button" title="Load this outline into the planner" @click="useDocumentOutlineTemplate(template)">Use</button>
          </header>
          <p>{{ template.summary }}</p>
          <pre>{{ documentOutlineTemplateToPlannerText(template) }}</pre>
          <div class="outline-template-actions">
            <button type="button" title="Send this outline to Docs Live" @click="sendOutlineTemplateToDocsLive(template)">Docs Live</button>
            <button type="button" title="Append this outline skeleton to the active document" @click="appendDocumentOutlineTemplate(template)">Append</button>
            <button v-if="template.source === 'custom'" type="button" title="Edit this custom outline's metadata using the current planner" @click="editCustomOutlineTemplate(template)">Edit</button>
            <button v-if="template.source === 'custom'" class="danger-action" type="button" title="Delete this custom outline" @click="store.deleteCustomDocumentOutlineTemplate(template.id)">Delete</button>
          </div>
        </article>
      </div>
    </section>
  </CollapsibleAdvanced>
</template>

<script setup lang="ts">
import { inject } from 'vue';
import { useDocumentsStore } from '../../stores/documents';
import CollapsibleAdvanced from '../../components/CollapsibleAdvanced.vue';

const store = useDocumentsStore();
const _ctx = inject('sidebarCtx') as Record<string, any>;
const {
  allDocumentOutlineTemplates,
  appendDocumentOutlineTemplate,
  appendOutlineToDocument,
  createDocumentFromOutline,
  customOutlineBestFor,
  customOutlineDraft,
  customOutlineTags,
  docsLiveDocumentTypes,
  documentMapBlockersOnly,
  documentMapCountByFilter,
  documentMapFilter,
  documentMapFilterOptions,
  documentMapKindLabel,
  documentMapQuery,
  documentMapSummary,
  documentOutlineTemplateToPlannerText,
  editCustomOutlineTemplate,
  filteredDocumentMapItems,
  filteredDocumentOutlineTemplates,
  focusDocumentMapBlockers,
  goToDocumentMapItem,
  goToSourceTarget,
  loadOutlineDraftFromDocument,
  openDocsLiveFromOutline,
  outlineDocsLiveTypeLabel,
  outlineDraftIncludeToc,
  outlineDraftItems,
  outlineDraftText,
  outlineDraftTitle,
  outlineHeadings,
  outlineLibraryCategories,
  outlineLibraryCategory,
  outlineLibraryQuery,
  resetCustomOutlineDraft,
  saveCurrentOutlineTemplate,
  sendOutlineTemplateToDocsLive,
  syncWorkspaceOutlines,
  useDocumentOutlineTemplate,
  workspaceOutlineSyncBusy,
  workspaceOutlineSyncStatus,
} = _ctx;
</script>
