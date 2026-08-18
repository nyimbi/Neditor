<template>
    <section
      v-if="docsLiveOpen"
      ref="docsLiveDialog"
      class="modal-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="Docs Live voice drafting"
      tabindex="-1"
      @keydown="handleModalKeydown('docs-live', $event)"
    >
      <form class="modal docs-live-modal" @submit.prevent="generateDocsLiveDraft">
        <header>
          <h2>Docs Live</h2>
          <button type="button" aria-label="Close Docs Live" @click="closeDocsLive">x</button>
        </header>

        <section class="docs-live-wizard-shell" aria-label="Docs Live creation wizard">
          <nav class="docs-live-stepper" aria-label="Docs Live wizard steps">
            <button
              v-for="(step, index) in docsLiveWizardSteps"
              :key="step.id"
              type="button"
              :class="{ active: docsLiveWizardStep === step.id, complete: index < docsLiveWizardStepIndex }"
              :aria-current="docsLiveWizardStep === step.id ? 'step' : undefined"
              @click="setDocsLiveWizardStep(step.id)"
            >
              <span>{{ index + 1 }}</span>
              <strong>{{ step.label }}</strong>
              <small>{{ step.summary }}</small>
            </button>
          </nav>

          <section v-if="docsLiveWizardStep === 'brief'" class="docs-live-wizard-panel" aria-label="Docs Live brief and decisions">
            <div class="docs-live-grid docs-live-grid-compact">
              <label>
                Document type
                <select v-model="docsLiveDocumentType" data-initial-focus @change="handleDocsLiveDocumentTypeChange">
                  <option v-for="type in docsLiveDocumentTypes" :key="type.id" :value="type.id">{{ type.label }}</option>
                </select>
              </label>
              <label>
                Document title
                <input v-model="docsLiveTitle" placeholder="Board brief, proposal, report" @change="refreshDocsLiveQuestionnaire" />
              </label>
              <label>
                Drafting depth
                <select v-model="docsLiveDraftingDepth">
                  <option v-for="depth in docsLiveDraftingDepthOptions" :key="depth.value" :value="depth.value">{{ depth.label }}</option>
                </select>
              </label>
            </div>

            <section class="docs-live-type-card" aria-label="Selected Docs Live document profile">
              <header>
                <div>
                  <strong>{{ docsLiveProfile.label }}</strong>
                  <span>{{ docsLiveProfile.planningLabel }} -> {{ docsLiveProfile.sequencingLabel }} -> {{ docsLiveProfile.qualityLabel }}</span>
                </div>
                <small>{{ docsLiveProfile.unitLabel }} workflow</small>
              </header>
              <div class="docs-live-profile-lists">
                <section>
                  <strong>Planning decisions</strong>
                  <ul>
                    <li v-for="artifact in docsLiveProfile.planningArtifacts.slice(0, 5)" :key="artifact">{{ artifact }}</li>
                  </ul>
                </section>
                <section>
                  <strong>Quality checks</strong>
                  <ul>
                    <li v-for="check in docsLiveProfile.qualityChecks.slice(0, 4)" :key="check">{{ check }}</li>
                  </ul>
                </section>
              </div>
            </section>

            <section class="docs-live-intent-brief" aria-label="AI Create intent brief">
              <header>
                <div>
                  <strong>Decisions to make before drafting</strong>
                  <span>These fields change with the document type and feed the outline, section queue, and QA pass.</span>
                </div>
                <small>{{ docsLiveIntentCompletion }}</small>
              </header>
              <div class="docs-live-intent-grid">
                <label v-for="field in docsLiveIntentFields" :key="field.key" :title="field.help">
                  {{ field.label }}
                  <input
                    :value="docsLivePlaceholderValue(field.key)"
                    :placeholder="field.placeholder"
                    @change="updateDocsLiveIntentField(field.key, inputValue($event))"
                  />
                  <small>{{ field.help }}</small>
                </label>
              </div>
            </section>
          </section>

          <section v-else-if="docsLiveWizardStep === 'outline'" class="docs-live-wizard-panel" aria-label="Docs Live outline planner">
            <section class="docs-live-outline-tools">
              <label>
                Outline
                <textarea v-model="docsLiveOutlineText" rows="12" placeholder="- Executive Summary&#10;- Recommendation&#10;- Next Steps" @input="handleDocsLiveOutlineInput"></textarea>
              </label>
              <aside class="docs-live-suggested-outline" aria-label="Suggested outline for selected document type">
                <header>
                  <div>
                    <strong>{{ docsLiveProfile.label }} outline</strong>
                    <span>Generated from the selected document type and refreshed when the type changes.</span>
                  </div>
                  <button type="button" @click="applyDocsLiveSelectedTypeOutline">Use this outline</button>
                </header>
                <label class="docs-live-outline-link">
                  <input v-model="docsLiveAutoUpdateOutline" type="checkbox" />
                  Keep editable outline linked to document type
                </label>
                <textarea :value="docsLiveSelectedTypeOutline" rows="12" readonly aria-label="Selected document type outline"></textarea>
                <div class="docs-live-outline-actions">
                  <button type="button" @click="loadDocsLiveOutlineFromDocument">Use document outline</button>
                  <button type="button" @click="refreshDocsLiveQuestionnaire">Refresh questions</button>
                </div>
              </aside>
            </section>
          </section>

          <section v-else-if="docsLiveWizardStep === 'context'" class="docs-live-wizard-panel" aria-label="Docs Live voice and context">
            <section class="docs-live-voice" aria-label="Voice dictation">
              <div class="docs-live-voice-actions">
                <button type="button" :disabled="!docsLiveSpeechAvailable" @click="toggleDocsLiveDictation">
                  {{ docsLiveListening ? "Stop dictation" : "Start dictation" }}
                </button>
                <button type="button" :disabled="docsLiveRuntimeChecking" @click="checkDocsLiveRuntime">
                  {{ docsLiveRuntimeChecking ? "Checking runtime..." : "Check AI runtime" }}
                </button>
                <span role="status">{{ docsLiveSpeechStatus }}</span>
              </div>
              <section v-if="docsLiveRuntimeReport" class="docs-live-runtime" aria-label="AI runtime readiness">
                <header>
                  <strong>Runtime readiness</strong>
                  <span>{{ docsLiveRuntimeReport.issues.length }} issues</span>
                </header>
                <ul>
                  <li>Speech: {{ docsLiveRuntimeReport.speechRecognition.state }} - {{ docsLiveRuntimeReport.speechRecognition.detail }}</li>
                  <li>Microphone: {{ docsLiveRuntimeReport.microphonePermission.state }} - {{ docsLiveRuntimeReport.microphonePermission.detail }}</li>
                  <li>Clipboard read: {{ docsLiveRuntimeReport.clipboardRead.state }} - {{ docsLiveRuntimeReport.clipboardRead.detail }}</li>
                  <li>Clipboard write: {{ docsLiveRuntimeReport.clipboardWrite.state }} - {{ docsLiveRuntimeReport.clipboardWrite.detail }}</li>
                </ul>
                <textarea :value="docsLiveRuntimeReport.markdown" rows="6" readonly aria-label="AI runtime readiness report"></textarea>
              </section>
              <label>
                Spoken direction
                <textarea v-model="docsLiveTranscript" rows="5" placeholder="Dictate what should change, who it is for, and the outcome you need." @input="refreshDocsLiveQuestionnaire"></textarea>
              </label>
              <p v-if="docsLiveInterimTranscript" class="sidebar-hint">{{ docsLiveInterimTranscript }}</p>
            </section>

            <section class="docs-live-voice-command-plan" aria-label="Docs Live voice command plan">
              <header>
                <div>
                  <strong>Voice command plan</strong>
                  <span>Natural commands become scoped drafting actions before generation.</span>
                </div>
                <button type="button" :disabled="!docsLiveVoiceCommandPlan.length" @click="appendDocsLiveVoiceCommandPlan">Use commands</button>
                <button type="button" :disabled="!docsLiveDraftingCommandCount" @click="runDocsLiveVoiceDraftingCommands">Run drafting</button>
                <button type="button" :disabled="!docsLiveWorkflowCommandCount" @click="runDocsLiveVoiceWorkflowCommands">Run workflows</button>
              </header>
              <ul v-if="docsLiveVoiceCommandPlan.length">
                <li v-for="item in docsLiveVoiceCommandPlan" :key="item.id">
                  <strong>{{ docsLiveVoiceCommandActionLabel(item.action) }} -> {{ item.target }}</strong>
                  <span>{{ item.prompt }}</span>
                  <small>{{ item.confidence }} | {{ item.rationale }}</small>
                  <button v-if="!item.workflowRoute" type="button" @click="runDocsLiveVoiceDraftingCommand(item)">Draft with agent</button>
                  <button v-if="item.workflowRoute" type="button" @click="runDocsLiveVoiceWorkflowCommand(item)">Run {{ item.workflowLabel }}</button>
                </li>
              </ul>
              <p v-else class="sidebar-hint">Dictate commands such as "expand the executive summary", "open Deep Research", "run QA", "prepare export", or "read selected text aloud".</p>
            </section>

            <div class="docs-live-context-grid">
              <label>
                Context and constraints
                <textarea v-model="docsLiveContext" rows="8" placeholder="Add freeform context, constraints, examples, evidence, tone, and review expectations." @input="refreshDocsLiveQuestionnaire"></textarea>
              </label>
              <label>
                AI-created questionnaire
                <textarea v-model="docsLiveQuestionnaireText" rows="8" readonly></textarea>
              </label>
            </div>

            <section class="docs-live-suggestions" aria-label="AI suggested optimal answers">
              <header>
                <div>
                  <strong>Suggested Answers</strong>
                  <span>Context-aware starting points for every wizard step.</span>
                </div>
                <button type="button" :disabled="!docsLiveSuggestedAnswers.length" @click="appendAllDocsLiveSuggestedAnswers">Use all</button>
              </header>
              <article v-for="suggestion in docsLiveSuggestedAnswers" :key="suggestion.id">
                <div>
                  <small>{{ suggestion.stepLabel }}</small>
                  <strong>{{ suggestion.question }}</strong>
                  <p>{{ suggestion.answer }}</p>
                  <p class="sidebar-hint">{{ suggestion.rationale }}</p>
                  <span>{{ suggestion.source }}</span>
                </div>
                <button
                  type="button"
                  :aria-label="`Use Docs Live suggested answer: ${suggestion.question}`"
                  :title="`Use Docs Live suggested answer: ${suggestion.question}`"
                  @click="appendDocsLiveSuggestedAnswer(suggestion)"
                >
                  Use
                </button>
              </article>
            </section>

            <label>
              Questionnaire answers
              <textarea
                v-model="docsLiveQuestionnaireAnswerText"
                rows="7"
                placeholder="1. The reader should approve renewal.&#10;2. Include usage growth, budget, risks, and named owner.&#10;3. Leave financial assumptions marked for review."
              ></textarea>
            </label>
          </section>

          <section v-else-if="docsLiveWizardStep === 'variables'" class="docs-live-wizard-panel" aria-label="Docs Live variables and evidence">
            <label>
              Placeholder values
              <textarea v-model="docsLivePlaceholderText" rows="7" placeholder="client: Acme&#10;audience: executive team&#10;deadline: June 1&#10;owner: Finance" @input="refreshDocsLiveQuestionnaire"></textarea>
            </label>
            <section class="docs-live-placeholder-manager" aria-label="Docs Live placeholder manager">
              <header>
                <div>
                  <strong>Placeholder Manager</strong>
                  <span>{{ docsLivePlaceholderRows.length }} values | Missing {{ docsLiveMissingPlaceholderKeys.join(", ") || "none" }}</span>
                </div>
              </header>
              <div class="docs-live-placeholder-add">
                <label>
                  Key
                  <input v-model="docsLivePlaceholderKey" placeholder="client, amount, source" />
                </label>
                <label>
                  Value
                  <input v-model="docsLivePlaceholderDraftValue" placeholder="Acme, $250K, audited forecast" />
                </label>
                <label>
                  Type
                  <select v-model="docsLivePlaceholderDraftKind">
                    <option v-for="kind in docsLivePlaceholderKindOptions" :key="kind.value" :value="kind.value">{{ kind.label }}</option>
                  </select>
                </label>
                <label>
                  Source
                  <input v-model="docsLivePlaceholderDraftSource" placeholder="Finance workbook, GC review, customer brief" />
                </label>
                <label>
                  Review
                  <select v-model="docsLivePlaceholderDraftStatus">
                    <option v-for="status in docsLivePlaceholderReviewStatusOptions" :key="status.value" :value="status.value">{{ status.label }}</option>
                  </select>
                </label>
                <button type="button" :disabled="!docsLivePlaceholderKey.trim() || !docsLivePlaceholderDraftValue.trim()" @click="addDocsLivePlaceholder">
                  Add value
                </button>
              </div>
              <div class="docs-live-placeholder-grid" role="table" aria-label="Managed variable table">
                <div class="docs-live-placeholder-head" role="row">
                  <span role="columnheader">Key</span>
                  <span role="columnheader">Value</span>
                  <span role="columnheader">Type</span>
                  <span role="columnheader">Source</span>
                  <span role="columnheader">Review</span>
                  <span role="columnheader">Action</span>
                </div>
                <div v-for="entry in docsLivePlaceholderRows" :key="entry.key" role="row">
                  <span role="cell">{{ entry.key }}</span>
                  <input role="cell" :value="entry.value" :aria-label="`Value for ${entry.key}`" @change="updateDocsLivePlaceholder(entry.key, inputValue($event))" />
                  <select role="cell" :value="entry.kind" :aria-label="`Type for ${entry.key}`" @change="updateDocsLivePlaceholderKind(entry.key, inputValue($event))">
                    <option v-for="kind in docsLivePlaceholderKindOptions" :key="kind.value" :value="kind.value">{{ kind.label }}</option>
                  </select>
                  <input role="cell" :value="entry.source" :aria-label="`Source for ${entry.key}`" placeholder="source or evidence" @change="updateDocsLivePlaceholderMetadata(entry.key, { source: inputValue($event) })" />
                  <select role="cell" :value="entry.reviewStatus" :aria-label="`Review status for ${entry.key}`" @change="updateDocsLivePlaceholderReviewStatus(entry.key, inputValue($event))">
                    <option v-for="status in docsLivePlaceholderReviewStatusOptions" :key="status.value" :value="status.value">{{ status.label }}</option>
                  </select>
                  <button type="button" role="cell" @click="removeDocsLivePlaceholderValue(entry.key)">Remove</button>
                </div>
              </div>
            </section>
          </section>

          <section v-else class="docs-live-wizard-panel" aria-label="Docs Live draft settings">
            <div class="docs-live-draft-settings">
              <label>
                Apply result
                <select v-model="docsLiveInsertMode">
                  <option value="replace">Replace document</option>
                  <option value="append">Append to document</option>
                  <option value="selection">Replace selection</option>
                  <option value="section">Replace matching section</option>
                </select>
              </label>
              <section class="docs-live-intent-brief" aria-label="AI document creation wizard stages">
                <header>
                  <div>
                    <strong>Creation workflow</strong>
                    <span>Identity, intent, outline, section drafting, QA, humanization, and review handoff.</span>
                  </div>
                </header>
                <ol class="wizard-step-list">
                  <li v-for="step in aiDocumentWizardSteps" :key="step.id">
                    <strong>{{ step.label }}</strong>
                    <span>{{ step.prompt }}</span>
                  </li>
                </ol>
                <div class="agent-cli-list" aria-label="Agentic local integrations">
                  <span v-for="integration in agenticCliIntegrations" :key="integration.id">
                    {{ integration.label }}
                    <code>{{ integration.command }}</code>
                  </span>
                </div>
              </section>
              <p v-if="docsLiveTargetSection" class="sidebar-hint">
                Target section: {{ docsLiveTargetSection.heading }}. Apply draft will replace that matching Markdown section when it exists, or append the generated section when it does not.
              </p>
            </div>
          </section>
        </section>

        <section v-if="docsLiveDraft?.issues.length" class="issue-list">
          <p v-for="issue in docsLiveDraft.issues" :key="issue">{{ issue }}</p>
        </section>

        <section v-if="docsLiveDraft" class="docs-live-workflow" aria-label="Docs Live section drafting workflow">
          <header>
            <strong>Systematic drafting workflow</strong>
            <span>{{ docsLiveDraft.sections.length }} sections prepared for review</span>
          </header>
          <ol>
            <li v-for="step in docsLiveDraft.workflow" :key="step.id" :data-status="step.status">
              <strong>{{ step.label }}</strong>
              <small>{{ step.status }}</small>
              <span>{{ step.detail }}</span>
              <p>{{ step.assistance }}</p>
              <em>{{ step.contextSignals.join(" | ") }}</em>
            </li>
          </ol>
          <div class="docs-live-section-cards">
            <article v-for="section in docsLiveDraft.sections" :key="section.title">
              <strong>{{ section.title }}</strong>
              <span>{{ section.qaFocus }}</span>
              <p>{{ section.draftingBrief }}</p>
              <ol class="docs-live-section-stage-list" :aria-label="`${section.title} drafting stages`">
                <li v-for="stage in section.stagePlan" :key="`${section.title}-${stage.id}`" :data-status="stage.status">
                  <strong>{{ stage.label }}</strong>
                  <small>{{ stage.status }}</small>
                  <span>{{ stage.detail }}</span>
                </li>
              </ol>
            </article>
          </div>
          <div class="docs-live-review-packet" aria-label="Docs Live review preparation packet">
            <header class="docs-live-review-packet-header">
              <div>
                <strong>Review preparation packet</strong>
                <span>Export the AI runbook, QA register, cleanup tasks, and reviewer prompts without replacing the draft.</span>
              </div>
              <div class="docs-live-review-actions">
                <button type="button" @click="insertDocsLiveReviewPacket">Insert packet</button>
                <button type="button" @click="copyDocsLiveReviewPacket">Copy packet</button>
              </div>
            </header>
            <section>
              <strong>Context package</strong>
              <ul>
                <li v-for="source in docsLiveDraft.reviewPacket.contextSources" :key="source">{{ source }}</li>
              </ul>
            </section>
            <section>
              <strong>Section runbook</strong>
              <ol>
                <li v-for="item in docsLiveDraft.reviewPacket.sectionRunbook" :key="item">{{ item }}</li>
              </ol>
            </section>
            <section>
              <strong>QA register</strong>
              <ul>
                <li v-for="item in docsLiveDraft.reviewPacket.qaRegister" :key="item">{{ item }}</li>
              </ul>
            </section>
            <section>
              <strong>Humanization checklist</strong>
              <ul>
                <li v-for="item in docsLiveDraft.reviewPacket.humanizationChecklist" :key="item">{{ item }}</li>
              </ul>
            </section>
            <section>
              <strong>Review packet</strong>
              <ul>
                <li v-for="item in docsLiveDraft.reviewPacket.reviewerHandoff" :key="item">{{ item }}</li>
              </ul>
            </section>
          </div>
        </section>

        <section v-if="docsLiveGeneratedMarkdown" class="docs-live-preview" aria-label="Docs Live generated draft">
          <header>
            <strong>{{ docsLiveDraft?.sections.length || 0 }} drafted sections</strong>
            <span>{{ docsLiveDraft?.title }}</span>
            <div class="docs-live-draft-actions">
              <button type="button" @click="appendDocsLiveDraftForReview">Append for review</button>
              <button type="button" @click="copyDocsLiveDraft">Copy draft</button>
            </div>
          </header>
          <textarea :value="docsLiveGeneratedMarkdown" rows="12" readonly aria-label="Docs Live generated Markdown"></textarea>
        </section>

        <section v-if="store.docsLiveDraftHistory.length" class="docs-live-history" aria-label="Docs Live draft history">
          <header>
            <div>
              <strong>Recent Docs Live drafts</strong>
              <span>{{ store.docsLiveDraftHistory.length }} saved locally for reuse</span>
            </div>
            <button type="button" @click="clearDocsLiveDraftHistory">Clear history</button>
          </header>
          <article v-for="item in store.docsLiveDraftHistory.slice(0, 6)" :key="item.draftId">
            <div>
              <strong>{{ item.title }}</strong>
              <span>{{ item.sectionCount }} sections / {{ item.documentType }}</span>
              <p>{{ item.markdownPreview }}</p>
            </div>
            <div class="docs-live-history-actions">
              <button type="button" @click="appendDocsLiveHistoryDraft(item)">Append draft</button>
              <button type="button" @click="copyDocsLiveHistoryDraft(item)">Copy draft</button>
              <button type="button" @click="insertDocsLiveHistoryReviewPacket(item)">Insert packet</button>
              <button type="button" @click="copyDocsLiveHistoryReviewPacket(item)">Copy packet</button>
              <button type="button" @click="removeDocsLiveHistoryDraft(item)">Remove</button>
            </div>
          </article>
        </section>

        <footer>
          <button type="button" @click="closeDocsLive">Cancel</button>
          <button type="button" :disabled="docsLiveWizardAtFirstStep" @click="goDocsLiveWizardStep(-1)">Back</button>
          <button type="button" :disabled="docsLiveWizardAtLastStep" @click="goDocsLiveWizardStep(1)">Next</button>
          <button type="button" @click="refreshDocsLiveQuestionnaire">Refresh questions</button>
          <button type="submit" @click="docsLiveWizardStep = 'draft'">Generate draft</button>
          <button type="button" :disabled="!docsLiveGeneratedMarkdown" @click="applyDocsLiveDraft">Apply draft</button>
        </footer>
      </form>
    </section>
</template>

<script setup lang="ts">
import { inject } from 'vue';

const _ctx = inject('docsLiveCtx') as Record<string, any>;
const {
  store,
  docsLiveDialog,
  docsLiveOpen,
  docsLiveDocumentType,
  docsLiveTitle,
  docsLiveOutlineText,
  docsLiveTranscript,
  docsLiveInterimTranscript,
  docsLiveContext,
  docsLivePlaceholderText,
  docsLivePlaceholderKey,
  docsLivePlaceholderDraftValue,
  docsLivePlaceholderDraftKind,
  docsLivePlaceholderDraftSource,
  docsLivePlaceholderDraftStatus,
  docsLivePlaceholderKindOptions,
  docsLivePlaceholderReviewStatusOptions,
  docsLiveWizardStep,
  docsLiveWizardSteps,
  docsLiveAutoUpdateOutline,
  docsLiveQuestionnaireText,
  docsLiveQuestionnaireAnswerText,
  docsLiveGeneratedMarkdown,
  docsLiveDraft,
  docsLiveDraftingDepth,
  docsLiveDraftingDepthOptions,
  docsLiveInsertMode,
  docsLiveTargetSection,
  docsLiveListening,
  docsLiveSpeechStatus,
  docsLiveRuntimeChecking,
  docsLiveRuntimeReport,
  docsLiveSpeechAvailable,
  docsLiveProfile,
  docsLiveIntentFields,
  docsLiveSelectedTypeOutline,
  docsLiveWizardStepIndex,
  docsLiveWizardAtFirstStep,
  docsLiveWizardAtLastStep,
  docsLivePlaceholderRows,
  docsLiveMissingPlaceholderKeys,
  docsLiveIntentCompletion,
  docsLiveSuggestedAnswers,
  docsLiveVoiceCommandPlan,
  docsLiveWorkflowCommandCount,
  docsLiveDraftingCommandCount,
  docsLiveDocumentTypes,
  aiDocumentWizardSteps,
  agenticCliIntegrations,
  handleModalKeydown,
  closeDocsLive,
  generateDocsLiveDraft,
  checkDocsLiveRuntime,
  refreshDocsLiveQuestionnaire,
  handleDocsLiveOutlineInput,
  handleDocsLiveDocumentTypeChange,
  applyDocsLiveSelectedTypeOutline,
  goDocsLiveWizardStep,
  setDocsLiveWizardStep,
  appendDocsLiveSuggestedAnswer,
  appendAllDocsLiveSuggestedAnswers,
  appendDocsLiveVoiceCommandPlan,
  runDocsLiveVoiceDraftingCommands,
  runDocsLiveVoiceDraftingCommand,
  runDocsLiveVoiceWorkflowCommands,
  runDocsLiveVoiceWorkflowCommand,
  docsLiveVoiceCommandActionLabel,
  toggleDocsLiveDictation,
  inputValue,
  loadDocsLiveOutlineFromDocument,
  docsLivePlaceholderValue,
  updateDocsLiveIntentField,
  addDocsLivePlaceholder,
  updateDocsLivePlaceholder,
  updateDocsLivePlaceholderKind,
  updateDocsLivePlaceholderMetadata,
  updateDocsLivePlaceholderReviewStatus,
  removeDocsLivePlaceholderValue,
  insertDocsLiveReviewPacket,
  copyDocsLiveReviewPacket,
  applyDocsLiveDraft,
  appendDocsLiveDraftForReview,
  copyDocsLiveDraft,
  clearDocsLiveDraftHistory,
  appendDocsLiveHistoryDraft,
  copyDocsLiveHistoryDraft,
  insertDocsLiveHistoryReviewPacket,
  copyDocsLiveHistoryReviewPacket,
  removeDocsLiveHistoryDraft,
} = _ctx;
</script>
