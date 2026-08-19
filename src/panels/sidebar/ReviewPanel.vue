<template>
  <h2>Review</h2>
  <section v-if="activeAgentControlCenter" class="agent-control-center persistent-agent-control" :data-status="activeAgentControlCenter.status" aria-label="Persistent AI control center">
    <header>
      <div>
        <strong>AI Control Center</strong>
        <span>{{ activeAgentControlCenter.summary }}</span>
      </div>
      <small>{{ activeAgentControlCenter.readinessScore }}/100 readiness</small>
    </header>
    <section class="agent-control-grid">
      <article>
        <h3>Next actions</h3>
        <ul>
          <li v-for="action in activeAgentControlCenter.nextActions" :key="`persistent-${action.lane}-${action.label}`">
            <strong>{{ action.label }}</strong>
            <span>{{ action.lane }} | {{ action.status }}</span>
            <p>{{ action.detail }}</p>
            <div class="agent-lifecycle-actions">
              <button type="button" @click="runAgentControlAction(action)">Run action</button>
            </div>
          </li>
        </ul>
      </article>
      <article>
        <h3>Source grounding</h3>
        <ul>
          <li v-for="item in activeAgentControlCenter.sourceGrounding" :key="`persistent-source-${item.label}`" :data-status="item.status">
            <strong>{{ item.label }}</strong>
            <span>{{ item.status }}</span>
            <p>{{ item.detail }}</p>
          </li>
        </ul>
      </article>
      <article>
        <h3>Governance</h3>
        <ul>
          <li v-for="item in activeAgentControlCenter.governance" :key="`persistent-governance-${item.label}`" :data-status="item.status">
            <strong>{{ item.label }}</strong>
            <span>{{ item.status }}</span>
            <p>{{ item.detail }}</p>
          </li>
        </ul>
      </article>
      <article>
        <h3>Distribution state</h3>
        <ul>
          <li v-for="item in activeAgentControlCenter.distribution" :key="`persistent-distribution-${item.label}`" :data-status="item.status">
            <strong>{{ item.label }}</strong>
            <span>{{ item.status }}</span>
            <p>{{ item.detail }}</p>
          </li>
        </ul>
      </article>
    </section>
    <div class="agent-section-actions">
      <button type="button" @click="openAgentWorkspace()">Open agent workspace</button>
      <button type="button" @click="runAgentPlanReview">Review readiness</button>
      <button type="button" @click="runAgentPlanDistribution">Distribution prep</button>
    </div>
  </section>
  <h3>Summary</h3>
  <article class="snapshot-row">
    <p>{{ reviewSummary.status }} | {{ reviewSummary.unresolved }} unresolved | {{ reviewSummary.resolved }} resolved</p>
    <small>{{ reviewSummary.changeNotes }} change notes | {{ reviewSummary.aiPending }} AI review pending | {{ reviewSummary.aiReviewed }} AI reviewed</small>
  </article>
  <!-- Style guide findings -->
  <section class="style-guide-panel" aria-label="Style guide enforcement">
    <header>
      <h3>Style guide</h3>
      <label class="style-guide-toggle" :title="store.styleGuideEnabled ? 'Disable style guide' : 'Enable style guide'">
        <input type="checkbox" v-model="store.styleGuideEnabled" />
        {{ store.styleGuideEnabled ? 'On' : 'Off' }}
      </label>
    </header>
    <template v-if="store.styleGuideEnabled">
      <p v-if="!styleGuideFindings.length" class="sidebar-hint">No style issues found.</p>
      <article
        v-for="finding in styleGuideFindings.slice(0, 50)"
        :key="finding.ruleId + ':' + finding.line + ':' + finding.column"
        class="style-finding"
        :data-severity="finding.severity"
      >
        <div class="style-finding-header">
          <span class="style-badge" :class="finding.severity">{{ finding.severity }}</span>
          <small>line {{ finding.line }} · {{ finding.category }}</small>
        </div>
        <p>{{ finding.description }}: <em>"{{ finding.matchedText }}"</em></p>
        <small class="style-suggestion">{{ finding.suggestion }}</small>
      </article>
      <p v-if="styleGuideFindings.length > 50" class="sidebar-hint">{{ styleGuideFindings.length - 50 }} more findings not shown.</p>
    </template>
    <p v-else class="sidebar-hint">Enable the style guide to check for weak qualifiers, filler phrases, passive voice, and jargon.</p>
  </section>

  <section class="quality-recommendations" aria-label="Quality improvement recommendations">
    <header>
      <h3>Quality recommendations</h3>
      <span>{{ qualityRecommendationSummary }}</span>
    </header>
    <p>Deterministic QA scans surface evidence gaps, review risks, structure issues, and concrete quality-improvement actions before human review or export.</p>
    <div class="release-readiness-actions">
      <button type="button" @click="runQualityReview">Run QA review</button>
      <button type="button" @click="insertQualityImprovementReport">Insert QA report</button>
      <button type="button" @click="openQualityAgent">Improve with agent</button>
    </div>
    <article
      v-for="item in qualityImprovementRecommendations"
      :key="item.id"
      class="snapshot-row"
      :data-status="item.severity"
    >
      <strong>{{ item.label }}</strong>
      <p>{{ item.recommendation }}</p>
      <small>{{ item.action }}</small>
    </article>
    <section class="quality-step-assistance" aria-label="AI quality review assistance">
      <header>
        <h4>AI quality assistance</h4>
        <button type="button" @click="appendAllQualityStepAssistance">Use all</button>
      </header>
      <p>Context-aware suggestions turn the QA findings into triage, evidence, humanization, and handoff answers you can edit before inserting.</p>
      <article
        v-for="item in qualityStepAssistance"
        :key="item.id"
        class="snapshot-row"
        data-status="improve"
      >
        <strong>{{ item.label }}</strong>
        <p>{{ item.suggestedAnswer }}</p>
        <small>{{ item.rationale }}</small>
        <ul class="signal-list">
          <li v-for="signal in item.contextSignals" :key="`${item.id}-${signal}`">{{ signal }}</li>
        </ul>
        <button type="button" @click="appendQualityStepAssistance(item)">{{ item.actionLabel }}</button>
      </article>
      <label class="field">
        <span>Quality review notes</span>
        <textarea v-model="qualityReviewNotes" aria-label="Quality review notes" rows="6" placeholder="Accept guidance, add owner decisions, and record reviewer questions here."></textarea>
      </label>
      <button type="button" @click="insertQualityReviewNotes">Insert review notes</button>
    </section>
  </section>
  <section class="review-evidence-snapshot" aria-label="Document evidence and approval review">
    <header>
      <div>
        <h3>Evidence and approval review</h3>
        <span>{{ reviewEvidenceSnapshotSummary }}</span>
      </div>
      <button type="button" @click="refreshReviewEvidenceSnapshot">Refresh</button>
    </header>
    <p>Surface claims, missing citations, unresolved reviewer comments, approval metadata, and specialist reviewer actions before export.</p>
    <div class="release-readiness-actions">
      <button type="button" :disabled="!activeReviewEvidenceRun" @click="insertReviewEvidenceAudit">Insert evidence audit</button>
      <button type="button" :disabled="!activeReviewEvidenceRun" @click="insertClaimEvidenceMatrix">Insert claim-source matrix</button>
      <button type="button" @click="openAgentWorkspace('Review this document for claim inventory, citations, approval metadata, reviewer objections, and export release blockers.')">Open agent workspace</button>
    </div>
    <template v-if="activeReviewEvidenceRun">
      <section class="review-evidence-metrics" aria-label="Evidence review metrics">
        <span><strong>{{ activeReviewEvidenceRun.documentEvidence.claimInventory.length }}</strong> claims</span>
        <span><strong>{{ activeClaimSourceMatches.length }}</strong> source matches</span>
        <span><strong>{{ activeReviewEvidenceRun.documentEvidence.citationTodos.length }}</strong> citation TODOs</span>
        <span><strong>{{ activeReviewEvidenceRun.documentEvidence.reviewCommentResolutions.length }}</strong> comments</span>
        <span><strong>{{ activeReviewEvidenceRun.approvalGate.blockers.length }}</strong> approval blockers</span>
      </section>
      <details :open="Boolean(activeReviewEvidenceRun.documentEvidence.claimInventory.length)">
        <summary>Claim inventory</summary>
        <article
          v-for="claim in activeReviewEvidenceRun.documentEvidence.claimInventory.slice(0, 12)"
          :key="`${claim.sourceLine}-${claim.text}`"
          class="snapshot-row"
          :data-status="claim.kind"
        >
          <strong>Line {{ claim.sourceLine }} | {{ claim.kind }}</strong>
          <p>{{ claim.text }}</p>
          <small>{{ claim.reason }}</small>
          <small v-if="activeClaimSourceMatchByLine.get(claim.sourceLine)">
            Suggested source: @{{ activeClaimSourceMatchByLine.get(claim.sourceLine)?.source.citation_key }}
            | {{ activeClaimSourceMatchByLine.get(claim.sourceLine)?.reasons.join("; ") }}
          </small>
        </article>
        <p v-if="!activeReviewEvidenceRun.documentEvidence.claimInventory.length" class="sidebar-hint">No candidate claims detected in the current snapshot.</p>
      </details>
      <details :open="Boolean(activeReviewEvidenceRun.approvalGate.blockers.length)">
        <summary>Approval metadata gate</summary>
        <article v-for="field in activeReviewEvidenceRun.approvalGate.fields" :key="field.key" class="snapshot-row" :data-status="field.status">
          <strong>{{ field.label }}</strong>
          <p>{{ field.value || "Missing" }}</p>
          <small>{{ field.guidance }}</small>
        </article>
        <ul v-if="activeReviewEvidenceRun.approvalGate.blockers.length" class="signal-list">
          <li v-for="blocker in activeReviewEvidenceRun.approvalGate.blockers" :key="blocker">{{ blocker }}</li>
        </ul>
      </details>
      <details>
        <summary>Reviewer agents</summary>
        <article v-for="reviewer in activeReviewEvidenceRun.reviewerAgents" :key="reviewer.id" class="snapshot-row" :data-status="reviewer.status">
          <strong>{{ reviewer.label }}</strong>
          <p>{{ reviewer.mandate }}</p>
          <small>{{ reviewer.requiredActions.slice(0, 2).join(" | ") }}</small>
        </article>
      </details>
    </template>
    <p v-else class="sidebar-hint">Refresh to create an evidence review snapshot for the current document.</p>
  </section>
  <section class="release-evidence-dashboard" :data-status="releaseEvidenceDashboard.status" aria-label="Release evidence dashboard">
    <header>
      <h3>Release evidence</h3>
      <span>{{ releaseEvidenceDashboard.summary }}</span>
    </header>
    <p>Track complete, blocked, manual, credentialed, cross-platform, stale, and ready-to-send evidence before distribution.</p>
    <div class="release-evidence-metrics" aria-label="Release evidence lane counts">
      <span><strong>{{ releaseEvidenceDashboard.counts.complete }}</strong> complete</span>
      <span><strong>{{ releaseEvidenceDashboard.counts.blocked }}</strong> blocked</span>
      <span><strong>{{ releaseEvidenceDashboard.counts.manual }}</strong> manual</span>
      <span><strong>{{ releaseEvidenceDashboard.counts.credentialed }}</strong> credentialed</span>
      <span><strong>{{ releaseEvidenceDashboard.counts["cross-platform"] }}</strong> cross-platform</span>
      <span><strong>{{ releaseEvidenceDashboard.counts.stale }}</strong> stale</span>
      <span><strong>{{ releaseEvidenceDashboard.counts["ready-to-send"] }}</strong> ready</span>
    </div>
    <div class="release-readiness-actions">
      <button type="button" @click="openConfigurationSetup('release')">Setup release evidence</button>
      <button type="button" @click="insertReleaseEvidenceDashboard">Insert evidence dashboard</button>
      <button type="button" @click="insertProductionReadinessWorkOrders">Insert work orders</button>
    </div>
    <section class="production-readiness-work-orders" aria-label="Production readiness work orders">
      <header>
        <h4>Production readiness work orders</h4>
        <span>{{ productionReadinessWorkOrders.length }} open</span>
      </header>
      <article v-for="workOrder in productionReadinessWorkOrders.slice(0, 6)" :key="workOrder.id" class="snapshot-row" :data-status="workOrder.priority">
        <strong>{{ workOrder.title }}</strong>
        <p>{{ workOrder.owner }} | {{ workOrder.command }}</p>
        <small>{{ workOrder.priority }} | {{ workOrder.lane }} | {{ workOrder.acceptanceEvidence }}</small>
      </article>
      <p v-if="!productionReadinessWorkOrders.length" class="sidebar-hint">All production-readiness evidence lanes are closed or ready to send.</p>
    </section>
    <article
      v-for="item in releaseEvidenceDashboard.items"
      :key="item.id"
      class="snapshot-row"
      :data-status="item.lane"
    >
      <strong>{{ item.label }}</strong>
      <p>{{ item.detail }}</p>
      <small>{{ item.lane }} | {{ item.action }}</small>
    </article>
  </section>
  <section class="release-readiness-checklist" aria-label="Release readiness checklist">
    <header>
      <h3>Release readiness</h3>
      <span>{{ releaseChecklistSummary }}</span>
    </header>
    <p>{{ releaseChecklistHelp }}</p>
    <div class="release-readiness-actions">
      <button type="button" @click="applyReleaseMetadataScaffold">Prepare release metadata</button>
      <button type="button" @click="insertReleaseReadinessAudit">Insert release audit</button>
    </div>
    <article
      v-for="item in releaseReadinessChecklist"
      :key="item.id"
      class="snapshot-row"
      :data-status="item.status"
    >
      <strong>{{ item.label }}</strong>
      <p>{{ item.detail }}</p>
      <small>{{ item.action }}</small>
    </article>
  </section>
  <h3>Release</h3>
  <label>
    Status
    <select :value="String(active.compile?.semantic.status || 'draft')" @change="setDocumentStatus(inputValue($event))">
      <option v-for="status in releaseStatuses" :key="status" :value="status">{{ status }}</option>
    </select>
  </label>
  <label>
    Version
    <input :value="String(active.compile?.metadata.version || '')" @input="setFrontMatterField('version', inputValue($event))" @change="setFrontMatterField('version', inputValue($event))" />
  </label>
  <label>
    Document set
    <input :value="String(active.compile?.metadata.documentSet || '')" @input="setFrontMatterField('documentSet', inputValue($event))" @change="setFrontMatterField('documentSet', inputValue($event))" />
  </label>
  <label>
    Owner
    <input :value="String(active.compile?.metadata.owner || '')" @input="setFrontMatterField('owner', inputValue($event))" @change="setFrontMatterField('owner', inputValue($event))" />
  </label>
  <label>
    Release target
    <input :value="String(active.compile?.metadata.releaseTarget || '')" @input="setFrontMatterField('releaseTarget', inputValue($event))" @change="setFrontMatterField('releaseTarget', inputValue($event))" />
  </label>
  <label>
    Approved by
    <input :value="String(active.compile?.metadata.approvedBy || '')" @input="setFrontMatterField('approvedBy', inputValue($event))" @change="setFrontMatterField('approvedBy', inputValue($event))" />
  </label>
  <label>
    Approved at
    <input :value="String(active.compile?.metadata.approvedAt || '')" @input="setFrontMatterField('approvedAt', inputValue($event))" @change="setFrontMatterField('approvedAt', inputValue($event))" />
  </label>
  <label>
    Source confidence
    <input :value="String(active.compile?.metadata.sourceConfidence || active.compile?.metadata.source_confidence || '')" @input="setFrontMatterField('sourceConfidence', inputValue($event))" @change="setFrontMatterField('sourceConfidence', inputValue($event))" />
  </label>
  <button type="button" @click="setApprovalTimestampNow">Set approval time</button>
  <label>
    New comment
    <textarea v-model="reviewCommentText" rows="4" placeholder="Review note"></textarea>
  </label>
  <button type="button" @click="insertReviewComment">Add comment</button>
  <label>
    Change note
    <textarea v-model="changeNoteText" rows="3" placeholder="Change summary"></textarea>
  </label>
  <button type="button" @click="insertChangeNote">Add change note</button>
  <h3>Comments</h3>
  <article v-for="comment in active.compile?.semantic.comments || []" :key="String(comment.line)" class="snapshot-row">
    <p>{{ comment.text }}</p>
    <small>Line {{ comment.line }} | {{ comment.state }} | {{ comment.author || "local" }}{{ comment.created_at ? ` | ${comment.created_at}` : "" }}</small>
    <button v-if="comment.state !== 'resolved'" type="button" @click="store.resolveReviewComment(Number(comment.line))">Resolve</button>
  </article>
  <h3>Change notes</h3>
  <article v-for="note in active.compile?.semantic.change_notes || []" :key="`change-${note.line}`" class="snapshot-row">
    <p>{{ note.text }}</p>
    <small>Line {{ note.line }} | {{ note.author || "local" }}{{ note.created_at ? ` | ${note.created_at}` : "" }}</small>
  </article>
  <h3>AI provenance</h3>
  <article v-for="source in active.compile?.semantic.ai_sources || []" :key="`ai-source-${source.line}`" class="snapshot-row">
    <p>{{ source.provider || "unknown" }} / {{ source.model || "unknown" }}</p>
    <small>{{ source.status }} | {{ source.reviewed_by || "unreviewed" }}{{ source.reviewed_at ? ` | ${source.reviewed_at}` : "" }}{{ source.prompt_summary ? ` | ${source.prompt_summary}` : "" }}</small>
    <label>
      <input
        type="checkbox"
        :checked="source.status === 'human-reviewed'"
        @change="toggleAiSourceReview(Number(source.line), $event)"
      />
      Human reviewed
    </label>
  </article>
  <article v-for="section in active.compile?.semantic.ai_assisted_sections || []" :key="`ai-section-${section.line}`" class="snapshot-row">
    <p>{{ section.heading || "Document body" }}</p>
    <small>Line {{ section.line }} | {{ section.status }} | {{ section.reviewed_by || "unreviewed" }}{{ section.reviewed_at ? ` | ${section.reviewed_at}` : "" }}</small>
    <label>
      <input
        type="checkbox"
        :checked="section.status === 'human-reviewed'"
        @change="toggleAiSectionReview(Number(section.line), $event)"
      />
      Human reviewed
    </label>
  </article>
</template>

<script setup lang="ts">
import { inject } from 'vue';
import { useDocumentsStore } from '../../stores/documents';

const store = useDocumentsStore();
const _ctx = inject('sidebarCtx') as Record<string, any>;
const {
  active,
  activeAgentControlCenter,
  activeClaimSourceMatchByLine,
  activeClaimSourceMatches,
  activeReviewEvidenceRun,
  appendAllQualityStepAssistance,
  appendQualityStepAssistance,
  applyReleaseMetadataScaffold,
  changeNoteText,
  inputValue,
  insertChangeNote,
  insertClaimEvidenceMatrix,
  insertProductionReadinessWorkOrders,
  insertQualityImprovementReport,
  insertQualityReviewNotes,
  insertReleaseEvidenceDashboard,
  insertReleaseReadinessAudit,
  insertReviewComment,
  insertReviewEvidenceAudit,
  openAgentWorkspace,
  openConfigurationSetup,
  openQualityAgent,
  productionReadinessWorkOrders,
  qualityImprovementRecommendations,
  qualityRecommendationSummary,
  qualityReviewNotes,
  qualityStepAssistance,
  refreshReviewEvidenceSnapshot,
  releaseChecklistHelp,
  releaseChecklistSummary,
  releaseEvidenceDashboard,
  releaseReadinessChecklist,
  releaseStatuses,
  reviewCommentText,
  reviewEvidenceSnapshotSummary,
  reviewSummary,
  runAgentControlAction,
  runAgentPlanDistribution,
  runAgentPlanReview,
  runQualityReview,
  setApprovalTimestampNow,
  setDocumentStatus,
  setFrontMatterField,
  styleGuideFindings,
  toggleAiSectionReview,
  toggleAiSourceReview,
} = _ctx;
</script>
