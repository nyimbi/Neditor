<template>
    <section
      v-if="agentWorkspaceOpen"
      ref="agentWorkspaceDialog"
      class="modal-backdrop"
      role="dialog"
      aria-modal="true"
      aria-label="AI agent workspace"
      tabindex="-1"
      @keydown="handleModalKeydown('agent-workspace', $event)"
    >
      <form class="modal agent-workspace-modal" @submit.prevent="buildAgentWorkspacePlan">
        <header>
          <div>
            <h2>AI Agent Workspace</h2>
            <p>Plan creation, composition, editing, revision, review, and distribution from one instruction.</p>
          </div>
          <button type="button" aria-label="Close AI agent workspace" @click="closeAgentWorkspace">x</button>
        </header>
        <label>
          What should NEditor do?
          <textarea
            v-model="agentInstruction"
            rows="5"
            data-initial-focus
            placeholder="Create a board memo for the executive team, revise it for the CFO, check evidence gaps, and prepare PDF plus Google Docs distribution."
          ></textarea>
        </label>
        <label>
          Context answers and constraints
          <textarea
            v-model="agentContextAnswers"
            rows="4"
            placeholder="Answer missing inputs, add source facts, target reviewer, approvals, distribution constraints, tone, deadlines, or placeholder values. These answers feed the next plan, packet, Docs Live handoff, and provider request."
          ></textarea>
        </label>
        <section class="agent-source-pack-builder" aria-label="Agent source pack builder">
          <header>
            <div>
              <strong>Source Pack Builder</strong>
              <span>
                {{ agentSourcePackPreview.items.length }} items |
                {{ agentSourcePackPreview.claims.length }} claims |
                {{ agentSourcePackPreview.urls.length }} URLs |
                {{ agentSourcePackPreview.files.length }} files |
                {{ agentSourcePackPreview.reviewerComments.length }} reviewer comments
              </span>
            </div>
          </header>
          <div class="agent-source-pack-add">
            <label>
              Type
              <select v-model="agentSourcePackKind">
                <option value="note">Note</option>
                <option value="claim">Claim</option>
                <option value="url">URL</option>
                <option value="file">File</option>
                <option value="reference">Reference</option>
                <option value="reviewer-comment">Reviewer comment</option>
              </select>
            </label>
            <label>
              Label
              <input v-model="agentSourcePackLabel" placeholder="Q2 forecast, CFO comment, research URL" />
            </label>
            <label>
              Detail
              <textarea v-model="agentSourcePackDetail" rows="3" placeholder="Paste the fact, link, file path, reviewer note, or citation detail."></textarea>
            </label>
            <button type="button" :disabled="!agentSourcePackLabel.trim() && !agentSourcePackDetail.trim()" @click="addAgentSourcePackItem">Add source</button>
          </div>
          <label>
            Managed source pack
            <textarea
              v-model="agentSourcePackText"
              rows="6"
              placeholder="[claim] ARR forecast: ARR grows 18% in Q2 according to finance workbook&#10;[url] Pricing source: https://example.com/pricing&#10;[reviewer-comment] CFO: Check renewal risk before board review"
            ></textarea>
          </label>
          <section class="agent-document-memory-manager" aria-label="Document memory manager">
            <header>
              <div>
                <strong>Document memory</strong>
                <span>{{ agentDocumentMemoryPreview.summary }}</span>
              </div>
              <div class="agent-memory-header-actions">
                <button type="button" :disabled="!agentDocumentMemoryPreview.entries.length" @click="insertAgentDocumentMemoryPack">Insert pack</button>
                <button type="button" :disabled="agentMemoryText.trim() === store.documentMemoryText.trim()" @click="saveAgentDocumentMemoryLibrary">Save memory</button>
                <button type="button" :disabled="!store.documentMemoryText.trim()" @click="reloadAgentDocumentMemoryLibrary">Reload saved</button>
              </div>
            </header>
            <div class="agent-memory-add">
              <label>
                Memory type
                <select v-model="agentMemoryKind">
                  <option value="terminology">Terminology</option>
                  <option value="style">Style</option>
                  <option value="accepted-decision">Accepted decision</option>
                  <option value="rejected-direction">Rejected direction</option>
                  <option value="review-preference">Review preference</option>
                  <option value="distribution-preference">Distribution preference</option>
                </select>
              </label>
              <label>
                Label
                <input v-model="agentMemoryLabel" placeholder="ARR, executive voice, board handoff" />
              </label>
              <label>
                Detail
                <textarea v-model="agentMemoryDetail" rows="3" placeholder="Annual recurring revenue; keep tone concise; use Google Docs for comments and PDF for board pack."></textarea>
              </label>
              <button type="button" :disabled="!agentMemoryLabel.trim() && !agentMemoryDetail.trim()" @click="addAgentMemoryItem">Add memory</button>
            </div>
            <label>
              Managed memory
              <textarea
                v-model="agentMemoryText"
                rows="5"
                placeholder="[terminology] ARR: Annual recurring revenue&#10;[style] Executive tone: concise, concrete, no generic AI phrasing&#10;[rejected] Scope: Do not frame this as a product launch"
              ></textarea>
            </label>
            <div class="agent-memory-actions">
              <button type="button" @click="captureAgentMemoryFromCurrentDocument">Capture from document</button>
              <button type="button" :disabled="!store.documentMemoryText.trim()" @click="agentMemoryText = appendTextBlock(agentMemoryText, store.documentMemoryText)">Append saved memory</button>
              <button type="button" :disabled="!agentDocumentMemoryPreview.entries.length" @click="copyAgentDocumentMemoryPack">Copy pack</button>
            </div>
            <ul v-if="agentDocumentMemoryPreview.entries.length" class="agent-source-pack-list">
              <li v-for="item in agentDocumentMemoryPreview.entries.slice(0, 8)" :key="item.id">
                <strong>{{ item.kind }} | {{ item.label }}</strong>
                <span>{{ item.detail }}</span>
              </li>
            </ul>
          </section>
          <ul v-if="agentSourcePackPreview.items.length" class="agent-source-pack-list">
            <li v-for="item in agentSourcePackPreview.items" :key="item.id">
              <strong>{{ item.kind }} | {{ item.label }}</strong>
              <span>{{ item.detail }}</span>
              <button type="button" @click="removeAgentSourcePackItem(item.id)">Remove</button>
            </li>
          </ul>
        </section>
        <section class="agent-playbooks" aria-label="Agent workflow playbooks">
          <header>
            <div>
              <strong>Workflow Playbooks</strong>
              <span>{{ filteredAgenticWorkflowPlaybooks.length }} of {{ agenticWorkflowPlaybooks.length }} governed starts match the current filters.</span>
            </div>
          </header>
          <section class="agent-playbook-filters" aria-label="Filter agent workflow playbooks">
            <label>
              Search
              <input v-model="agentPlaybookQuery" type="search" placeholder="board, grant, policy, Substack, LaTeX" />
            </label>
            <label>
              Focus
              <select v-model="agentPlaybookFocusFilter">
                <option v-for="focus in agentPlaybookFocusOptions" :key="focus.value" :value="focus.value">{{ focus.label }}</option>
              </select>
            </label>
            <label>
              Output target
              <select v-model="agentPlaybookTargetFilter">
                <option v-for="target in agentPlaybookTargetOptions" :key="target.value" :value="target.value">{{ target.label }}</option>
              </select>
            </label>
          </section>
          <p v-if="!filteredAgenticWorkflowPlaybooks.length" class="sidebar-hint">No playbooks match the current filters.</p>
          <div class="agent-playbook-grid">
            <article v-for="playbook in filteredAgenticWorkflowPlaybooks" :key="playbook.id">
              <header>
                <div>
                  <strong>{{ playbook.label }}</strong>
                  <span>{{ playbook.summary }}</span>
                </div>
                <button
                  type="button"
                  :aria-label="`Use ${playbook.label} playbook`"
                  :data-help="`Fill the Agent Workspace instruction and context from the ${playbook.label} playbook.`"
                  @click="applyAgentWorkflowPlaybook(playbook)"
                >
                  Use
                </button>
              </header>
              <p class="agent-playbook-meta">
                {{ agentPlaybookFocusLabel(playbook) }} | {{ agentPlaybookTargets(playbook).map((target) => target.toUpperCase()).join(", ") || "No fixed export target" }}
              </p>
              <dl>
                <div>
                  <dt>Best for</dt>
                  <dd>{{ playbook.bestFor.join(", ") }}</dd>
                </div>
                <div>
                  <dt>Outputs</dt>
                  <dd>{{ playbook.expectedOutputs.join(", ") }}</dd>
                </div>
              </dl>
            </article>
          </div>
        </section>
        <div class="agent-workspace-actions">
          <button type="submit">Plan agent workflow</button>
          <button type="button" :disabled="!agentPlan" @click="generateAgentWorkspaceRun">Generate agent packet</button>
          <button type="button" :disabled="!agentRun" @click="applyAgentWorkspaceRun">Apply agent output</button>
          <button type="button" :disabled="!agentRun" @click="buildAgentProviderPackage">Build provider request</button>
          <button type="button" :disabled="!agentProviderPackage" @click="copyAgentProviderPackage">Copy provider package</button>
          <button type="button" :disabled="!agentProviderPackage" @click="copyAgentProviderSourcePack">Copy source pack</button>
          <button type="button" :disabled="!canPrepareLocalAgentHandoff" @click="prepareLocalAgentHandoff">
            {{ localAgentHandoffBusy ? "Preparing agent..." : "Prepare local agent" }}
          </button>
          <button type="button" :disabled="!canRunAgentProvider" @click="runAgentProviderRequest">
            {{ agentProviderBusy ? "Running provider..." : "Run provider request" }}
          </button>
          <button type="button" :disabled="!agentPlan" @click="hydrateDocsLiveFromAgentPlan">Send to Docs Live</button>
          <button type="button" :disabled="!agentPlan" @click="runAgentPlanReview">Review readiness</button>
          <button type="button" :disabled="!agentPlan" @click="runAgentPlanDistribution">Distribution prep</button>
        </div>
        <section v-if="agentPlan" class="agent-plan" aria-label="Agent workflow plan">
          <header>
            <div>
              <strong>{{ agentPlan.title }}</strong>
              <span>{{ agentPlan.documentType }} | {{ agentPlan.lanes.join(" -> ") }}</span>
            </div>
            <small>{{ agentPlan.steps.length }} steps</small>
          </header>
          <section class="agent-plan-grid">
            <article class="agent-context-score" :data-status="agentPlan.contextCompleteness.status">
              <h3>Context completeness</h3>
              <strong>{{ agentPlan.contextCompleteness.score }}/100 {{ agentPlan.contextCompleteness.status }}</strong>
              <p>Present: {{ agentPlan.contextCompleteness.present.join(", ") || "none" }}</p>
              <p>Missing: {{ agentPlan.contextCompleteness.missing.join(", ") || "none" }}</p>
              <ul>
                <li v-for="item in agentPlan.contextCompleteness.recommendations" :key="item">{{ item }}</li>
              </ul>
            </article>
            <article class="agent-intent-sheet" :data-status="agentPlan.documentIntent.status">
              <h3>Document intent sheet</h3>
              <strong>{{ agentPlan.documentIntent.completenessScore }}/100 {{ agentPlan.documentIntent.status }}</strong>
              <p>{{ agentPlan.documentIntent.summary }}</p>
              <dl>
                <div v-for="field in agentPlan.documentIntent.fields" :key="field.key" :data-status="field.status">
                  <dt>{{ field.label }}</dt>
                  <dd>{{ field.value }} <span>{{ field.source }}</span></dd>
                </div>
              </dl>
            </article>
            <article>
              <h3>Context pack</h3>
              <pre>{{ agentPlan.context }}</pre>
            </article>
            <article>
              <h3>Placeholders</h3>
              <pre>{{ agentPlan.placeholderText }}</pre>
            </article>
            <article class="agent-plan-source-pack">
              <h3>Source pack</h3>
              <p>{{ agentPlan.sourcePack.items.length }} managed source items</p>
              <ul>
                <li v-for="item in agentPlan.sourcePack.items.slice(0, 6)" :key="item.id">{{ item.kind }}: {{ item.label }}</li>
              </ul>
            </article>
            <article class="agent-document-memory">
              <h3>Document memory</h3>
              <p>{{ agentPlan.documentMemory.summary }}</p>
              <ul>
                <li v-for="item in agentPlan.documentMemory.entries.slice(0, 6)" :key="item.id">{{ item.kind }}: {{ item.label }}</li>
              </ul>
            </article>
            <article class="agent-quality-gates">
              <h3>Quality gates</h3>
              <p>{{ agentPlan.qualityGates.length }} document-type gates</p>
              <ul>
                <li v-for="gate in agentPlan.qualityGates" :key="gate.id">{{ gate.label }}</li>
              </ul>
            </article>
            <article>
              <h3>Suggested outline</h3>
              <pre>{{ agentPlan.suggestedOutline }}</pre>
            </article>
            <article class="agent-outline-variants">
              <h3>Outline variants</h3>
              <p>{{ agentPlan.outlineVariants.length }} structures ready for comparison before drafting.</p>
              <div v-for="variant in agentPlan.outlineVariants" :key="variant.id" class="agent-outline-variant">
                <strong>{{ variant.label }}</strong>
                <small>{{ variant.strategy }}</small>
                <p>{{ variant.summary }}</p>
                <pre>{{ variant.outline }}</pre>
                <dl>
                  <div>
                    <dt>Best for</dt>
                    <dd>{{ variant.bestFor.join(", ") }}</dd>
                  </div>
                  <div>
                    <dt>Tradeoffs</dt>
                    <dd>{{ variant.tradeoffs.join(" ") }}</dd>
                  </div>
                  <div>
                    <dt>Risks</dt>
                    <dd>{{ variant.risks.join(" ") }}</dd>
                  </div>
                </dl>
                <div class="agent-outline-variant-actions">
                  <button type="button" @click="hydrateDocsLiveFromOutlineVariant(variant)">Use in Docs Live</button>
                  <button type="button" @click="loadOutlineVariantInPlanner(variant)">Load in outline planner</button>
                </div>
              </div>
            </article>
            <article>
              <h3>Revision instruction</h3>
              <p>{{ agentPlan.revisionInstruction }}</p>
            </article>
            <article v-if="agentPlan.revisionModes.length" class="agent-revision-modes">
              <h3>Revision passes</h3>
              <ul>
                <li v-for="mode in agentPlan.revisionModes" :key="mode">{{ mode }}</li>
              </ul>
            </article>
          </section>
          <section v-if="agentPlan.missingInputs.length" class="agent-missing-inputs" aria-label="Agent missing inputs">
            <strong>Missing inputs</strong>
            <ul>
              <li v-for="input in agentPlan.missingInputs" :key="input">{{ input }}</li>
            </ul>
            <button type="button" @click="buildAgentWorkspacePlan">Replan with answers</button>
          </section>
          <section class="agent-step-assistance" aria-label="AI step-by-step assistance">
            <header>
              <div>
                <strong>AI Step Assistance</strong>
                <span>Suggested optimal answers for each creation, revision, review, and distribution step.</span>
              </div>
              <small>{{ agentPlan.stepAssistance.length }} suggestions</small>
            </header>
            <article v-for="assistance in agentPlan.stepAssistance" :key="assistance.id" :data-status="assistance.status">
              <div>
                <small>{{ assistance.lane }} | {{ assistance.status }}</small>
                <strong>{{ assistance.stepLabel }}</strong>
                <p>{{ assistance.suggestedAnswer }}</p>
                <p class="sidebar-hint">{{ assistance.rationale }}</p>
              </div>
              <ul>
                <li v-for="signal in assistance.contextUsed" :key="signal">{{ signal }}</li>
              </ul>
              <div class="agent-lifecycle-actions">
                <button type="button" @click="appendAgentStepAssistance(assistance)">Add answer and replan</button>
                <button type="button" @click="runAgentAssistedStep(assistance)">Run step</button>
              </div>
            </article>
          </section>
          <ol class="agent-step-list" aria-label="Agent workflow steps">
            <li v-for="step in agentPlan.steps" :key="step.id" :data-lane="step.lane">
              <div>
                <small>{{ step.lane }} | {{ step.status }}</small>
                <strong>{{ step.title }}</strong>
                <p>{{ step.detail }}</p>
              </div>
              <button type="button" @click="runAgenticStep(step)">Run step</button>
            </li>
          </ol>
          <section v-if="agentRun" class="agent-run-output" aria-label="Agent generated output">
            <header>
              <div>
                <strong>{{ agentRun.summary }}</strong>
                <span>Apply mode: {{ agentRun.applicationMode }}</span>
              </div>
              <small>{{ agentRun.blockers.length }} blockers</small>
              <div class="agent-run-packet-actions">
                <button type="button" @click="appendAgentWorkspacePacket">Append packet</button>
                <button type="button" @click="copyAgentWorkspacePacket">Copy packet</button>
              </div>
            </header>
            <section class="agent-control-center" :data-status="agentRun.controlCenter.status" aria-label="AI control center">
              <header>
                <div>
                  <strong>AI Control Center</strong>
                  <span>{{ agentRun.controlCenter.summary }}</span>
                </div>
                <small>{{ agentRun.controlCenter.readinessScore }}/100 readiness</small>
              </header>
              <section class="agent-control-grid">
                <article>
                  <h3>Next actions</h3>
                  <ul>
                    <li v-for="action in agentRun.controlCenter.nextActions" :key="`${action.lane}-${action.label}`">
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
                    <li v-for="item in agentRun.controlCenter.sourceGrounding" :key="item.label" :data-status="item.status">
                      <strong>{{ item.label }}</strong>
                      <span>{{ item.status }}</span>
                      <p>{{ item.detail }}</p>
                    </li>
                  </ul>
                </article>
                <article>
                  <h3>Governance</h3>
                  <ul>
                    <li v-for="item in agentRun.controlCenter.governance" :key="item.label" :data-status="item.status">
                      <strong>{{ item.label }}</strong>
                      <span>{{ item.status }}</span>
                      <p>{{ item.detail }}</p>
                    </li>
                  </ul>
                </article>
                <article>
                  <h3>Distribution state</h3>
                  <ul>
                    <li v-for="item in agentRun.controlCenter.distribution" :key="item.label" :data-status="item.status">
                      <strong>{{ item.label }}</strong>
                      <span>{{ item.status }}</span>
                      <p>{{ item.detail }}</p>
                    </li>
                  </ul>
                </article>
              </section>
            </section>
            <section class="agent-automation-scheduler" aria-label="Agent automation scheduler">
              <header>
                <div>
                  <strong>Automation Scheduler</strong>
                  <span>Safe local checks queued for evidence, outline, transforms, export preflight, accessibility, and readiness refresh.</span>
                </div>
                <small>{{ completedAgentAutomationCount }} of {{ agentRun.automationQueue.length }} complete</small>
                <div class="agent-section-actions">
                  <button type="button" :disabled="!safeRunnableAgentAutomationRows.length" @click="runSafeAgentAutomationQueue">Run safe queue</button>
                  <button type="button" @click="insertAgentAutomationAudit">Insert audit</button>
                  <button type="button" @click="copyAgentAutomationAudit">Copy audit</button>
                </div>
              </header>
              <ol>
                <li v-for="row in agentAutomationRows" :key="row.task.id" :data-status="row.state.status">
                  <div>
                    <small>{{ row.task.kind }} | {{ row.task.owner }} | {{ row.task.safeToAutoRun ? "safe" : "manual" }} | {{ row.state.status }}</small>
                    <strong>{{ row.task.label }}</strong>
                    <p>{{ row.task.trigger }}</p>
                    <p>{{ row.task.nextStep }}</p>
                    <p v-if="row.task.manualOnlyReason" class="sidebar-hint">{{ row.task.manualOnlyReason }}</p>
                    <p v-if="row.state.result" class="sidebar-hint">Result: {{ row.state.result }}</p>
                    <div class="agent-lifecycle-actions">
                      <button type="button" :disabled="row.state.status === 'running' || row.task.status === 'blocked' || !row.task.safeToAutoRun" @click="runAgentAutomationTask(row.task)">Run check</button>
                      <button type="button" @click="openAgentAutomationTaskSurface(row.task)">Open surface</button>
                    </div>
                  </div>
                  <ul>
                    <li v-for="item in row.task.evidence" :key="item">{{ item }}</li>
                  </ul>
                </li>
              </ol>
            </section>
            <section v-if="agentRun.documentEvidence.claimInventory.length" class="agent-claim-inventory" aria-label="Agent claim inventory">
              <header>
                <div>
                  <strong>Claim Inventory</strong>
                  <span>Trace numbers, dates, commitments, quotes, and risk claims before approval.</span>
                </div>
                <small>{{ agentRun.documentEvidence.claimInventory.length }} claims</small>
              </header>
              <div class="agent-section-actions">
                <button type="button" @click="insertAgentClaimInventoryAudit">Insert claim audit</button>
                <button type="button" @click="copyAgentClaimInventoryAudit">Copy claim audit</button>
              </div>
              <article v-for="claim in agentRun.documentEvidence.claimInventory" :key="`${claim.sourceLine}-${claim.text}`" class="snapshot-row" :data-status="claim.kind">
                <p>{{ claim.text }}</p>
                <small>Line {{ claim.sourceLine }} | {{ claim.kind }} | {{ claim.reason }}</small>
                <div class="reference-actions">
                  <button type="button" @click="goToSourceTarget({ line: claim.sourceLine })">Go to claim</button>
                  <button type="button" @click="insertClaimCitationTodo(claim)">Add citation TODO</button>
                </div>
              </article>
            </section>
            <section
              v-if="agentRun.documentEvidence.reviewCommentResolutions.length"
              class="agent-review-comment-queue"
              aria-label="Review comment resolution queue"
            >
              <header>
                <div>
                  <strong>Review Comment Resolution Queue</strong>
                  <span>Turn unresolved comments into reviewer-owned decisions with notes before release.</span>
                </div>
                <small>{{ agentRun.documentEvidence.reviewCommentResolutions.length }} unresolved</small>
              </header>
              <ol>
                <li
                  v-for="comment in agentRun.documentEvidence.reviewCommentResolutions"
                  :key="comment.id"
                  :data-blocker="comment.blocker"
                  :data-status="agentReviewCommentState(comment)?.status || 'queued'"
                >
                  <div>
                    <small>
                      Line {{ comment.line }} | {{ comment.author }} | {{ agentReviewCommentState(comment)?.status || "queued" }}
                    </small>
                    <strong>{{ comment.excerpt }}</strong>
                    <p>{{ comment.requiredAction }}</p>
                    <p v-if="agentReviewCommentState(comment)?.note" class="sidebar-hint">
                      Resolution note: {{ agentReviewCommentState(comment)?.note }}
                    </p>
                  </div>
                  <ul>
                    <li v-for="option in comment.resolutionOptions" :key="option">{{ option }}</li>
                  </ul>
                  <div class="agent-lifecycle-actions">
                    <button type="button" @click="setAgentReviewCommentStatus(comment, 'in-progress')">Start</button>
                    <button type="button" @click="setAgentReviewCommentStatus(comment, 'needs-review')">Carry forward</button>
                    <button type="button" @click="setAgentReviewCommentStatus(comment, 'complete')">Resolve</button>
                  </div>
                  <label>
                    Resolution note
                    <input
                      :value="agentReviewCommentState(comment)?.note || ''"
                      placeholder="Decision, source, owner, date, or carry-forward reason"
                      @change="setAgentReviewCommentNote(comment, inputValue($event))"
                    />
                  </label>
                </li>
              </ol>
            </section>
            <section v-if="agentRun.editAcceptanceQueue.length" class="agent-edit-acceptance-queue" aria-label="Agent edit acceptance queue">
              <header>
                <div>
                  <strong>Edit Acceptance Queue</strong>
                  <span>Review generated edits one item at a time before applying accepted changes.</span>
                </div>
                <small>{{ acceptedAgentEditCount }} accepted of {{ agentRun.editAcceptanceQueue.length }}</small>
              </header>
              <ol>
                <li v-for="row in agentEditAcceptanceRows" :key="row.item.id" :data-scope="row.item.scope" :data-status="row.state.status">
                  <div>
                    <small>{{ row.item.scope }} | {{ row.state.status }}</small>
                    <strong>{{ row.item.heading }}</strong>
                    <p>{{ row.item.recommendation }}</p>
                    <p v-if="row.state.note" class="sidebar-hint">Acceptance note: {{ row.state.note }}</p>
                  </div>
                  <section class="agent-edit-acceptance-compare">
                    <article>
                      <h3>Original</h3>
                      <pre>{{ row.item.originalText }}</pre>
                    </article>
                    <article>
                      <h3>Proposed</h3>
                      <pre>{{ row.item.proposedText }}</pre>
                    </article>
                  </section>
                  <div>
                    <h3>Risk notes</h3>
                    <ul>
                      <li v-for="note in row.item.riskNotes" :key="note">{{ note }}</li>
                    </ul>
                  </div>
                  <div class="agent-lifecycle-actions">
                    <button type="button" @click="setAgentEditAcceptanceStatus(row.item, 'accepted')">Accept</button>
                    <button type="button" @click="setAgentEditAcceptanceStatus(row.item, 'rejected')">Reject</button>
                    <button type="button" @click="reviseAgentAcceptanceItem(row.item)">Revise</button>
                  </div>
                  <label>
                    Acceptance note
                    <input
                      :value="row.state.note || ''"
                      placeholder="Reason accepted, rejected, or sent for another pass"
                      @change="setAgentEditAcceptanceNote(row.item, inputValue($event))"
                    />
                  </label>
                </li>
              </ol>
              <button type="button" :disabled="acceptedAgentEditCount === 0" @click="applyAcceptedAgentEdits">Apply accepted edits</button>
            </section>
            <section class="agent-lifecycle-board" aria-label="Agent lifecycle task board">
              <header>
                <div>
                  <strong>Lifecycle Task Board</strong>
                  <span>Operational tasks for creating, composing, editing, revising, reviewing, and distributing the document.</span>
                </div>
                <small>{{ agentLifecycleTaskRows.length }} of {{ agentLifecycleTaskTotal }} tasks</small>
              </header>
              <section class="agent-lifecycle-filters" aria-label="Filter agent lifecycle tasks">
                <label>
                  Lane
                  <select v-model="agentTaskLaneFilter">
                    <option v-for="lane in agentTaskLaneOptions" :key="lane" :value="lane">{{ lane === "all" ? "All lanes" : lane }}</option>
                  </select>
                </label>
                <label>
                  Status
                  <select v-model="agentTaskStatusFilter">
                    <option v-for="status in agentTaskStatusOptions" :key="status" :value="status">{{ status === "all" ? "All statuses" : status }}</option>
                  </select>
                </label>
                <label>
                  Owner
                  <select v-model="agentTaskOwnerFilter">
                    <option v-for="owner in agentTaskOwnerOptions" :key="owner" :value="owner">{{ owner === "all" ? "All owners" : owner }}</option>
                  </select>
                </label>
                <label>
                  Section
                  <select v-model="agentTaskSectionFilter">
                    <option v-for="section in agentTaskSectionOptions" :key="section.value" :value="section.value">{{ section.label }}</option>
                  </select>
                </label>
                <label>
                  Target
                  <select v-model="agentTaskTargetFilter">
                    <option v-for="target in agentTaskTargetOptions" :key="target" :value="target">{{ target === "all" ? "All targets" : target }}</option>
                  </select>
                </label>
                <label>
                  Evidence
                  <select v-model="agentTaskEvidenceFilter">
                    <option value="all">All evidence states</option>
                    <option value="has-evidence">Has evidence</option>
                    <option value="missing-evidence">Missing evidence</option>
                    <option value="release-blocker">Release blockers</option>
                  </select>
                </label>
                <label>
                  Search tasks
                  <input v-model="agentTaskQuery" placeholder="search title, note, evidence, or next step" />
                </label>
              </section>
              <p v-if="!agentLifecycleTaskRows.length" class="sidebar-hint">No lifecycle tasks match the current filters.</p>
              <ol v-else>
                <li v-for="row in agentLifecycleTaskRows" :key="row.task.id" :data-lane="row.task.lane" :data-status="row.state.status">
                  <div>
                    <small>{{ row.task.lane }} | {{ row.state.status }} | {{ row.task.owner }}</small>
                    <strong>{{ row.task.title }}</strong>
                    <p>{{ row.task.nextStep }}</p>
                    <p v-if="row.state.note" class="sidebar-hint">Execution note: {{ row.state.note }}</p>
                    <div class="agent-lifecycle-actions">
                      <button type="button" @click="runAgentLifecycleTask(row.task)">Run task</button>
                      <button type="button" @click="setAgentLifecycleTaskStatus(row.task, 'in-progress')">Start</button>
                      <button type="button" @click="setAgentLifecycleTaskStatus(row.task, 'needs-review')">Needs review</button>
                      <button type="button" @click="setAgentLifecycleTaskStatus(row.task, 'complete')">Complete</button>
                      <button type="button" @click="insertAgentLifecycleTaskBrief(row.task)">Insert brief</button>
                      <button type="button" @click="copyAgentLifecycleTaskBrief(row.task)">Copy brief</button>
                    </div>
                    <label>
                      Task note
                      <input
                        :value="row.state.note || ''"
                        placeholder="Evidence, blocker, reviewer, or completion note"
                        @change="setAgentLifecycleTaskNote(row.task, inputValue($event))"
                      />
                    </label>
                  </div>
                  <ul>
                    <li v-for="item in row.task.evidence" :key="item">{{ item }}</li>
                  </ul>
                </li>
              </ol>
            </section>
            <section class="agent-reviewer-agents" aria-label="Agent reviewer agents">
              <header>
                <div>
                  <strong>Review Agents</strong>
                  <span>Specialized agent checks for editorial quality, evidence, risk, citations, governance, and export readiness.</span>
                </div>
                <small>{{ agentRun.reviewerAgents.length }} reviewers</small>
              </header>
              <section class="agent-reviewer-grid">
                <article v-for="reviewer in agentRun.reviewerAgents" :key="reviewer.id" :data-status="reviewer.status">
                  <header>
                    <div>
                      <strong>{{ reviewer.label }}</strong>
                      <span>{{ reviewer.status }}</span>
                    </div>
                  </header>
                  <p>{{ reviewer.mandate }}</p>
                  <div>
                    <h3>Findings</h3>
                    <ul>
                      <li v-for="item in reviewer.findings" :key="item">{{ item }}</li>
                    </ul>
                  </div>
                  <div>
                    <h3>Required actions</h3>
                    <ul>
                      <li v-for="item in reviewer.requiredActions" :key="item">{{ item }}</li>
                    </ul>
                  </div>
                </article>
              </section>
            </section>
            <section class="agent-pre-review-rehearsal" aria-label="Agent pre-review rehearsal">
              <header>
                <div>
                  <strong>Pre-review Rehearsal</strong>
                  <span>Likely reviewer questions, objections, redlines, and missing-evidence requests to resolve before formal review.</span>
                </div>
                <small>{{ agentRun.preReviewRehearsal.length }} prompts</small>
              </header>
              <ol>
                <li v-for="item in agentRun.preReviewRehearsal" :key="item.id" :data-kind="item.kind" :data-blocker="item.releaseBlocker">
                  <div>
                    <small>{{ item.kind }} | {{ item.reviewer }} reviewer <span v-if="item.releaseBlocker">| release blocker</span></small>
                    <strong>{{ item.prompt }}</strong>
                    <p>{{ item.whyItMatters }}</p>
                  </div>
                  <p>{{ item.suggestedResponse }}</p>
                </li>
              </ol>
            </section>
            <section class="agent-section-workqueue" aria-label="Agent section work queue">
              <header>
                <div>
                  <strong>Section Work Queue</strong>
                  <span>Draft and review the document section by section with assigned reviewer agents.</span>
                </div>
                <small>{{ agentRun.sectionWorkQueue.length }} sections</small>
              </header>
              <ol>
                <li v-for="section in agentRun.sectionWorkQueue" :key="section.id">
                  <div>
                    <small>Level {{ section.level }} | {{ section.lane }} | {{ section.draftingDepth }} depth</small>
                    <strong>{{ section.heading }}</strong>
                    <label class="agent-section-depth">
                      Depth
                      <select v-model="section.draftingDepth">
                        <option v-for="depth in agentSectionDraftingDepthOptions" :key="depth.value" :value="depth.value">{{ depth.label }}</option>
                      </select>
                    </label>
                    <p>{{ section.draftingInstruction }}</p>
                    <dl class="agent-section-contract">
                      <div>
                        <dt>Purpose</dt>
                        <dd>{{ section.contract.purpose }}</dd>
                      </div>
                      <div>
                        <dt>Reader</dt>
                        <dd>{{ section.contract.targetReader }}</dd>
                      </div>
                      <div>
                        <dt>Outcome</dt>
                        <dd>{{ section.contract.desiredDecision }}</dd>
                      </div>
                      <div>
                        <dt>Owner</dt>
                        <dd>{{ section.contract.owner }}</dd>
                      </div>
                      <div>
                        <dt>Risk</dt>
                        <dd>{{ section.contract.riskLevel }}</dd>
                      </div>
                    </dl>
                    <ul class="agent-section-contract-list" aria-label="Section contract evidence expectations">
                      <li v-for="item in section.contract.evidenceExpectations" :key="item">{{ item }}</li>
                    </ul>
                    <span>Reviewers: {{ section.reviewerAgentIds.join(", ") }}</span>
                    <div class="agent-section-actions">
                      <button type="button" @click="insertAgentSectionBrief(section)">Insert brief</button>
                      <button type="button" @click="draftAgentSectionWithDocsLive(section)">Draft in Docs Live</button>
                    </div>
                  </div>
                  <ul>
                    <li v-for="item in section.completionCriteria" :key="item">{{ item }}</li>
                  </ul>
                </li>
              </ol>
            </section>
            <section class="agent-section-draft-history" aria-label="Agent section draft history">
              <header>
                <div>
                  <strong>Section Draft History</strong>
                  <span>Composable section versions with prompt summaries, rationale, reviewer notes, fingerprints, and restore points.</span>
                </div>
                <small>{{ agentRun.sectionDraftHistory.length }} versions</small>
              </header>
              <ol>
                <li v-for="item in agentRun.sectionDraftHistory" :key="item.id" :data-status="item.acceptanceStatus">
                  <div>
                    <small>{{ item.versionLabel }} | {{ item.acceptanceStatus }} | {{ item.sectionFingerprint }}</small>
                    <strong>{{ item.sectionHeading }}</strong>
                    <p>{{ item.promptSummary }}</p>
                    <p>{{ item.rationale }}</p>
                    <ul>
                      <li v-for="note in item.reviewerNotes" :key="note">{{ note }}</li>
                    </ul>
                    <div class="agent-section-actions">
                      <button type="button" @click="insertAgentSectionDraftRestorePoint(item)">Insert restore point</button>
                      <button type="button" @click="draftAgentSectionHistoryWithDocsLive(item)">Draft in Docs Live</button>
                      <button type="button" @click="copyAgentSectionDraftRestorePoint(item)">Copy restore point</button>
                    </div>
                  </div>
                  <pre>{{ item.restorePointMarkdown }}</pre>
                </li>
              </ol>
            </section>
            <section class="agent-transform-recommendations" aria-label="Agent transform recommendations">
              <header>
                <div>
                  <strong>Agent-Selected Transforms</strong>
                  <span>Structured blocks the agent recommends from document intent, source data, evidence, and distribution needs.</span>
                </div>
                <small>{{ agentRun.transformRecommendations.length }} recommendations</small>
                <div class="agent-section-actions">
                  <button type="button" @click="openTransformTemplatesFromAgent">Open templates</button>
                </div>
              </header>
              <ol>
                <li v-for="item in agentRun.transformRecommendations" :key="item.id" :data-kind="item.kind" :data-risk="item.riskLevel">
                  <div>
                    <small>{{ item.kind }} | {{ item.owner }} | {{ item.riskLevel }} risk</small>
                    <strong>{{ item.label }}</strong>
                    <p>{{ item.purpose }}</p>
                    <p>Target: {{ item.insertionTarget }}</p>
                    <p>Trigger: {{ item.narrativeReviewTrigger }}</p>
                    <p class="sidebar-hint">Signal: {{ item.sourceSignal }}</p>
                    <div class="agent-section-actions">
                      <button type="button" @click="insertAgentTransformRecommendation(item)">Insert block</button>
                      <button type="button" @click="copyAgentTransformRecommendation(item)">Copy block</button>
                    </div>
                  </div>
                  <ul>
                    <li v-for="evidence in item.evidenceRequired" :key="evidence">{{ evidence }}</li>
                  </ul>
                </li>
              </ol>
            </section>
            <section class="agent-data-narrative-bridge" aria-label="Agent data-to-narrative bridge">
              <header>
                <div>
                  <strong>Data-to-Narrative Bridge</strong>
                  <span>Links claims, calculations, charts, tables, timelines, schemas, and publishing metadata to narrative review actions.</span>
                </div>
                <small>{{ agentRun.dataNarrativeLinks.length }} links</small>
                <div class="agent-section-actions">
                  <button type="button" @click="insertAgentDataNarrativeAudit">Insert audit</button>
                  <button type="button" @click="copyAgentDataNarrativeAudit">Copy audit</button>
                </div>
              </header>
              <ol>
                <li v-for="item in agentRun.dataNarrativeLinks" :key="item.id" :data-status="item.status">
                  <div>
                    <small>{{ item.sourceKind }} | {{ item.owner }} | {{ item.status }}</small>
                    <strong>{{ item.sourceLabel }}</strong>
                    <p>Affects: {{ item.affectedSection }}</p>
                    <p>{{ item.changeSignal }}</p>
                    <p>{{ item.narrativeRisk }}</p>
                    <p class="sidebar-hint">{{ item.reviewAction }}</p>
                  </div>
                  <ul>
                    <li v-for="evidence in item.evidenceRequired" :key="evidence">{{ evidence }}</li>
                  </ul>
                </li>
              </ol>
            </section>
            <section class="agent-approval-gate" aria-label="Agent approval metadata gate" :data-status="agentRun.approvalGate.status">
              <header>
                <div>
                  <strong>Approval Metadata Gate</strong>
                  <span>{{ agentRun.approvalGate.summary }}</span>
                </div>
                <small>{{ agentRun.approvalGate.status }} | {{ agentRun.approvalGate.blockers.length }} blockers</small>
                <div class="agent-section-actions">
                  <button type="button" @click="insertAgentApprovalGateScaffold">Insert scaffold</button>
                  <button type="button" @click="copyAgentApprovalGateScaffold">Copy scaffold</button>
                </div>
              </header>
              <section class="agent-approval-gate-grid">
                <article v-for="field in agentRun.approvalGate.fields" :key="field.key" :data-status="field.status">
                  <small>{{ field.status }}</small>
                  <strong>{{ field.label }}</strong>
                  <p>{{ field.value || "Missing" }}</p>
                  <p class="sidebar-hint">{{ field.guidance }}</p>
                </article>
              </section>
              <ul v-if="agentRun.approvalGate.blockers.length">
                <li v-for="blocker in agentRun.approvalGate.blockers" :key="blocker">{{ blocker }}</li>
              </ul>
            </section>
            <section class="agent-audit-trail" aria-label="Agent audit trail">
              <header>
                <div>
                  <strong>Agent Audit Trail</strong>
                  <span>{{ agentRun.auditTrail.runId }}</span>
                </div>
                <small>{{ agentRun.auditTrail.plannerVersion }}</small>
              </header>
              <section class="agent-audit-grid">
                <article>
                  <h3>Fingerprints</h3>
                  <dl>
                    <div>
                      <dt>Instruction</dt>
                      <dd>{{ agentRun.auditTrail.instructionFingerprint }}</dd>
                    </div>
                    <div>
                      <dt>Context</dt>
                      <dd>{{ agentRun.auditTrail.contextFingerprint }}</dd>
                    </div>
                    <div>
                      <dt>Source</dt>
                      <dd>{{ agentRun.auditTrail.sourceFingerprint }}</dd>
                    </div>
                    <div>
                      <dt>Output</dt>
                      <dd>{{ agentRun.auditTrail.outputFingerprint }}</dd>
                    </div>
                  </dl>
                </article>
                <article>
                  <h3>Rollback plan</h3>
                  <ul>
                    <li v-for="item in agentRun.auditTrail.rollbackPlan" :key="item">{{ item }}</li>
                  </ul>
                </article>
                <article>
                  <h3>Review events</h3>
                  <ul>
                    <li v-for="item in agentRun.auditTrail.reviewEvents" :key="item">{{ item }}</li>
                  </ul>
                </article>
              </section>
            </section>
            <section class="agent-release-evidence" aria-label="Agent release evidence bundle">
              <header>
                <div>
                  <strong>Release Evidence Bundle</strong>
                  <span>{{ agentRun.releaseEvidenceBundle.summary }}</span>
                </div>
                <small>{{ agentRun.releaseEvidenceBundle.blockers.length }} blockers</small>
                <div class="agent-release-evidence-actions">
                  <button type="button" @click="insertAgentReleaseEvidenceAuditPackage">Insert audit package</button>
                  <button type="button" @click="copyAgentReleaseEvidenceAuditPackage">Copy audit package</button>
                </div>
              </header>
              <section class="agent-release-evidence-grid">
                <article
                  v-for="item in agentRun.releaseEvidenceBundle.items"
                  :key="item.label"
                  :data-status="item.status"
                >
                  <small>{{ item.owner }} | {{ item.requiredBeforeRelease ? "required" : "optional" }}</small>
                  <strong>{{ item.label }}</strong>
                  <p>{{ item.detail }}</p>
                </article>
              </section>
            </section>
            <section v-if="agentRun.blockers.length" class="agent-missing-inputs" aria-label="Agent run blockers">
              <strong>Resolve before final release</strong>
              <ul>
                <li v-for="blocker in agentRun.blockers" :key="blocker">{{ blocker }}</li>
              </ul>
            </section>
            <section class="agent-run-columns">
              <article>
                <h3>QA gates</h3>
                <ul>
                  <li v-for="item in agentRun.reviewChecklist" :key="item">{{ item }}</li>
                </ul>
              </article>
              <article>
                <h3>Distribution gates</h3>
                <ul>
                  <li v-for="item in agentRun.distributionChecklist" :key="item">{{ item }}</li>
                </ul>
              </article>
            </section>
            <section v-if="agentRun.distributionTargetPlans.length" class="agent-distribution-runbooks" aria-label="Agent distribution target runbooks">
              <article v-for="targetPlan in agentRun.distributionTargetPlans" :key="targetPlan.target">
                <header>
                  <strong>{{ targetPlan.label }}</strong>
                  <span>{{ targetPlan.purpose }}</span>
                </header>
                <div>
                  <h3>Preflight</h3>
                  <ul>
                    <li v-for="item in targetPlan.preflightChecks" :key="item">{{ item }}</li>
                  </ul>
                </div>
                <div>
                  <h3>Handoff</h3>
                  <ul>
                    <li v-for="item in targetPlan.handoffSteps" :key="item">{{ item }}</li>
                  </ul>
                </div>
                <div>
                  <h3>Evidence</h3>
                  <ul>
                    <li v-for="item in targetPlan.evidenceRequired" :key="item">{{ item }}</li>
                  </ul>
                </div>
              </article>
            </section>
            <textarea :value="agentRun.markdown" rows="12" readonly aria-label="Agent generated Markdown"></textarea>
          </section>
          <section v-if="store.agentRunHistory.length" class="agent-history" aria-label="Agent run history">
            <header>
              <div>
                <strong>Agent Run History</strong>
                <span>Local audit records for generated and applied agent work.</span>
              </div>
              <small>{{ filteredAgentRunHistory.length }} of {{ store.agentRunHistory.length }} saved</small>
              <div class="agent-history-audit-actions">
                <button type="button" :disabled="!filteredAgentRunHistory.length" @click="insertAgentHistoryAudit">Insert audit</button>
                <button type="button" :disabled="!filteredAgentRunHistory.length" @click="copyAgentHistoryAudit">Copy audit</button>
                <button type="button" @click="clearAgentHistory">Clear history</button>
              </div>
            </header>
            <section class="agent-history-filters" aria-label="Filter agent run history">
              <label>
                Search
                <input v-model="agentHistoryQuery" type="search" placeholder="Instruction, evidence, provider, blocker" />
              </label>
              <label>
                Status
                <select v-model="agentHistoryStatusFilter">
                  <option value="all">All statuses</option>
                  <option value="generated">Generated</option>
                  <option value="applied">Applied</option>
                  <option value="provider-applied">Provider applied</option>
                </select>
              </label>
              <label>
                Lane
                <select v-model="agentHistoryLaneFilter">
                  <option v-for="lane in agentTaskLaneOptions" :key="lane" :value="lane">
                    {{ lane === "all" ? "All lanes" : lane }}
                  </option>
                </select>
              </label>
              <label>
                Target
                <select v-model="agentHistoryTargetFilter">
                  <option value="all">All targets</option>
                  <option v-for="option in agentPlaybookTargetOptions.filter((item) => item.value !== 'all')" :key="option.value" :value="option.value">
                    {{ option.label }}
                  </option>
                </select>
              </label>
            </section>
            <p v-if="!filteredAgentRunHistory.length" class="sidebar-hint">No agent runs match the current history filters.</p>
            <ol>
              <li v-for="item in filteredAgentRunHistory.slice(0, 12)" :key="item.runId">
                <div>
                  <strong>{{ item.title }}</strong>
                  <span>{{ item.status }} | {{ item.applicationMode }} | {{ item.readinessScore }}/100</span>
                  <small>{{ item.runId }} | {{ item.updatedAt }}</small>
                  <p v-if="item.packetPreview">{{ item.packetPreview }}</p>
                  <p v-if="item.controlCenter">Control: {{ item.controlCenter.status }} | {{ item.controlCenter.summary }}</p>
                  <p v-if="item.documentIntent">Intent: {{ agentRunHistoryIntentSummary(item) }}</p>
                  <p v-if="item.documentEvidence">Evidence: {{ agentRunHistoryEvidenceSummary(item) }}</p>
                  <p v-if="item.outlineCritique?.length">Outline: {{ agentRunHistoryOutlineSummary(item) }}</p>
                  <p v-if="item.sectionDraftHistory?.length">Section drafts: {{ agentRunHistorySectionDraftSummary(item) }}</p>
                  <p v-if="item.transformRecommendationCount">Transforms: {{ item.transformRecommendationCount }} agent-selected recommendations</p>
                  <p v-if="item.dataNarrativeLinkCount">Narrative links: {{ item.dataNarrativeLinkCount }} data-to-narrative dependencies</p>
                  <p v-if="item.approvalGateStatus">Approval gate: {{ item.approvalGateStatus }}</p>
                  <p v-if="item.automationTaskCount">Automation: {{ agentRunHistoryAutomationSummary(item) }}</p>
                  <p v-if="item.sourcePack">Source pack: {{ agentRunHistorySourcePackSummary(item) }}</p>
                  <p v-if="item.lifecycleTaskStates?.length">Task states: {{ agentRunHistoryTaskStateSummary(item) }}</p>
                  <div class="agent-history-actions">
                    <button type="button" @click="replanAgentHistoryRun(item)">Replan</button>
                    <button type="button" :disabled="!item.packetMarkdown" @click="appendAgentHistoryPacket(item)">Append packet</button>
                    <button type="button" :disabled="!item.packetMarkdown" @click="copyAgentHistoryPacket(item)">Copy packet</button>
                    <button type="button" @click="removeAgentHistoryRun(item)">Remove</button>
                  </div>
                </div>
                <dl>
                  <div>
                    <dt>Output</dt>
                    <dd>{{ item.outputFingerprint }}</dd>
                  </div>
                  <div>
                    <dt>Source</dt>
                    <dd>{{ item.sourceFingerprint }}</dd>
                  </div>
                  <div>
                    <dt>Provider</dt>
                    <dd>{{ item.providerProfile || "local planner" }}</dd>
                  </div>
                  <div>
                    <dt>Sections</dt>
                    <dd>{{ item.sectionCount || 0 }} / {{ item.sectionDraftVersionCount || item.sectionDraftHistory?.length || 0 }} draft versions</dd>
                  </div>
                  <div>
                    <dt>Reviewers</dt>
                    <dd>{{ item.reviewerCount || 0 }}</dd>
                  </div>
                  <div>
                    <dt>Tasks</dt>
                    <dd>{{ item.taskCount || 0 }}</dd>
                  </div>
                </dl>
              </li>
            </ol>
          </section>
          <section class="agent-provider-panel" aria-label="AI provider handoff">
            <header>
              <div>
                <strong>Provider handoff</strong>
                <span>Generate a redacted request package for an approved AI provider or local model gateway.</span>
              </div>
            </header>
            <section class="agent-provider-grid">
              <label>
                Provider profile
                <select v-model="agentProviderId" @change="syncAgentProviderProfile">
                  <option v-for="profile in aiProviderProfiles" :key="profile.id" :value="profile.id">
                    {{ profile.label }}
                  </option>
                </select>
              </label>
              <label>
                Model
                <select
                  v-if="isOllamaProvider"
                  v-model="agentProviderModel"
                  aria-label="Ollama model"
                  :disabled="ollamaModelBusy || (!ollamaModelOptions.length && !agentProviderModel.trim())"
                  @change="confirmOllamaModelSelection"
                >
                  <option v-if="!ollamaModelOptions.length && !agentProviderModel.trim()" value="">Refresh Ollama models to choose</option>
                  <option v-if="currentModelMissingFromOllamaList || (!ollamaModelOptions.length && agentProviderModel.trim())" :value="agentProviderModel">
                    {{ agentProviderModel }} (current)
                  </option>
                  <option v-for="model in ollamaModelOptions" :key="model.name" :value="model.name">
                    {{ formatOllamaModelOption(model) }}
                  </option>
                </select>
                <input v-else v-model="agentProviderModel" placeholder="Approved model or deployment name" />
                <small v-if="isOllamaProvider && ollamaSelectedModelMetadata" class="model-selection-note">{{ ollamaSelectedModelMetadata }}</small>
              </label>
              <div v-if="isOllamaProvider" class="ollama-model-picker" aria-label="Ollama model discovery">
                <button type="button" :disabled="ollamaModelBusy || !agentProviderEndpoint.trim()" @click="refreshOllamaModels">
                  {{ ollamaModelBusy ? "Loading models..." : "Refresh from Ollama" }}
                </button>
                <small>{{ ollamaModelPickerHelp }}</small>
              </div>
              <label>
                Endpoint
                <input v-model="agentProviderEndpoint" placeholder="https://provider.example/v1/messages" />
              </label>
              <label>
                API key environment variable
                <input v-model="agentProviderKeyEnv" placeholder="NEDITOR_AI_API_KEY" />
              </label>
              <label>
                Session API key
                <input v-model="agentProviderApiKey" type="password" autocomplete="off" placeholder="Used once, never saved" />
              </label>
            </section>
            <section v-if="agentProviderPackage" class="agent-provider-output" aria-label="AI provider request package">
              <header>
                <div>
                  <strong>{{ agentProviderPackage.profile.label }}</strong>
                  <span>{{ agentProviderPackage.profile.summary }}</span>
                </div>
              </header>
              <ul>
                <li v-for="item in agentProviderPackage.checklist" :key="item">{{ item }}</li>
              </ul>
              <label>
                Source evidence pack
                <textarea :value="agentProviderSourcePackMarkdown" rows="8" readonly aria-label="AI provider source evidence pack"></textarea>
              </label>
              <textarea :value="agentProviderPackage.markdown" rows="12" readonly aria-label="AI provider request Markdown"></textarea>
            </section>
            <section v-if="agentProviderPackage && currentLocalAgentProfile" class="agent-provider-output local-agent-handoff" aria-label="Local agent handoff">
              <header>
                <div>
                  <strong>{{ currentLocalAgentProfile.label }} workspace handoff</strong>
                  <span>{{ currentLocalAgentProfile.workspaceHint }}</span>
                </div>
                <button type="button" :disabled="!canPrepareLocalAgentHandoff" @click="prepareLocalAgentHandoff">
                  {{ localAgentHandoffBusy ? "Preparing..." : "Prepare local agent workspace" }}
                </button>
              </header>
              <dl v-if="localAgentHandoffResult" class="local-agent-handoff-details">
                <div>
                  <dt>CLI</dt>
                  <dd>{{ localAgentHandoffResult.command }} {{ localAgentHandoffResult.available ? "available" : "not found" }}</dd>
                </div>
                <div>
                  <dt>Workspace</dt>
                  <dd>{{ localAgentHandoffResult.workspace_path }}</dd>
                </div>
                <div>
                  <dt>Handoff file</dt>
                  <dd>{{ localAgentHandoffResult.handoff_path }}</dd>
                </div>
                <div>
                  <dt>Response file</dt>
                  <dd>{{ localAgentHandoffResult.response_path }}</dd>
                </div>
                <div>
                  <dt>Launch command</dt>
                  <dd>{{ localAgentHandoffResult.launch_command.join(" ") }}</dd>
                </div>
              </dl>
              <section class="agent-provider-grid" aria-label="Local agent response import">
                <label class="wide-field">
                  Response Markdown file
                  <input v-model="localAgentResponsePath" placeholder=".neditor/agent-handoffs/neditor-codex-cli-...response.md" />
                </label>
                <div class="reference-actions">
                  <button type="button" :disabled="!canImportLocalAgentResponse" @click="importLocalAgentResponse">
                    {{ localAgentResponseBusy ? "Importing..." : "Import local response" }}
                  </button>
                  <button type="button" :disabled="!localAgentResponsePath" @click="copyLocalAgentResponsePath">Copy response path</button>
                </div>
              </section>
              <p v-if="localAgentResponseImport" class="sidebar-hint">
                Imported {{ localAgentResponseImport.characters }} characters from {{ localAgentResponseImport.label }}; sha256 {{ localAgentResponseImport.sha256.slice(0, 12) }}.
              </p>
              <ul v-if="localAgentResponseImport?.warnings.length">
                <li v-for="item in localAgentResponseImport.warnings" :key="item">{{ item }}</li>
              </ul>
              <ul v-if="localAgentHandoffResult">
                <li v-for="item in localAgentHandoffResult.instructions" :key="item">{{ item }}</li>
                <li v-for="item in localAgentHandoffResult.warnings" :key="item">{{ item }}</li>
              </ul>
              <p v-if="localAgentHandoffError" class="field-error">{{ localAgentHandoffError }}</p>
            </section>
            <section v-if="agentProviderResult" class="agent-provider-output" aria-label="AI provider response">
              <header>
                <div>
                  <strong>Provider response</strong>
                  <span>{{ agentProviderResult.status }} {{ agentProviderResult.statusText }} | Apply wraps this output in needs-review provenance.</span>
                </div>
                <button type="button" @click="applyAgentProviderResponse">Apply response</button>
              </header>
              <textarea :value="agentProviderResult.markdown" rows="12" readonly aria-label="AI provider response Markdown"></textarea>
            </section>
          </section>
        </section>
      </form>
    </section>
</template>

<script setup lang="ts">
import { inject } from 'vue';

const _ctx = inject('agentWorkspaceCtx') as Record<string, any>;
const {
  store,
  agentWorkspaceDialog,
  agentWorkspaceOpen,
  agentInstruction,
  agentContextAnswers,
  agentSourcePackText,
  agentSourcePackKind,
  agentSourcePackLabel,
  agentSourcePackDetail,
  agentMemoryText,
  agentMemoryKind,
  agentMemoryLabel,
  agentMemoryDetail,
  agentPlaybookQuery,
  agentPlaybookFocusFilter,
  agentPlaybookTargetFilter,
  agentPlan,
  agentRun,
  agentProviderPackage,
  localAgentHandoffBusy,
  agentProviderBusy,
  agentProviderId,
  agentProviderModel,
  ollamaModelBusy,
  ollamaModelOptions,
  agentProviderEndpoint,
  agentProviderKeyEnv,
  agentProviderApiKey,
  agentTaskLaneFilter,
  agentTaskStatusFilter,
  agentTaskOwnerFilter,
  agentTaskSectionFilter,
  agentTaskTargetFilter,
  agentTaskEvidenceFilter,
  agentTaskQuery,
  agentHistoryQuery,
  agentHistoryStatusFilter,
  agentHistoryLaneFilter,
  agentHistoryTargetFilter,
  localAgentHandoffResult,
  localAgentResponsePath,
  localAgentResponseBusy,
  localAgentResponseImport,
  localAgentHandoffError,
  agentProviderResult,
  agentSourcePackPreview,
  agentDocumentMemoryPreview,
  filteredAgenticWorkflowPlaybooks,
  isOllamaProvider,
  currentModelMissingFromOllamaList,
  ollamaSelectedModelMetadata,
  ollamaModelPickerHelp,
  canPrepareLocalAgentHandoff,
  canRunAgentProvider,
  canImportLocalAgentResponse,
  agentProviderSourcePackMarkdown,
  currentLocalAgentProfile,
  agentLifecycleTaskRows,
  agentLifecycleTaskTotal,
  agentTaskOwnerOptions,
  agentTaskSectionOptions,
  agentTaskTargetOptions,
  agentEditAcceptanceRows,
  acceptedAgentEditCount,
  agentAutomationRows,
  completedAgentAutomationCount,
  safeRunnableAgentAutomationRows,
  filteredAgentRunHistory,
  agentPlaybookFocusOptions,
  agentPlaybookTargetOptions,
  agentTaskLaneOptions,
  agentTaskStatusOptions,
  agentSectionDraftingDepthOptions,
  agenticWorkflowPlaybooks,
  aiProviderProfiles,
  handleModalKeydown,
  closeAgentWorkspace,
  buildAgentWorkspacePlan,
  addAgentSourcePackItem,
  removeAgentSourcePackItem,
  insertAgentDocumentMemoryPack,
  saveAgentDocumentMemoryLibrary,
  reloadAgentDocumentMemoryLibrary,
  addAgentMemoryItem,
  captureAgentMemoryFromCurrentDocument,
  appendTextBlock,
  copyAgentDocumentMemoryPack,
  applyAgentWorkflowPlaybook,
  agentPlaybookFocusLabel,
  agentPlaybookTargets,
  generateAgentWorkspaceRun,
  applyAgentWorkspaceRun,
  buildAgentProviderPackage,
  copyAgentProviderPackage,
  copyAgentProviderSourcePack,
  prepareLocalAgentHandoff,
  runAgentProviderRequest,
  hydrateDocsLiveFromAgentPlan,
  runAgentPlanReview,
  runAgentPlanDistribution,
  runAgenticStep,
  appendAgentStepAssistance,
  runAgentAssistedStep,
  hydrateDocsLiveFromOutlineVariant,
  loadOutlineVariantInPlanner,
  appendAgentWorkspacePacket,
  copyAgentWorkspacePacket,
  runSafeAgentAutomationQueue,
  insertAgentAutomationAudit,
  copyAgentAutomationAudit,
  runAgentAutomationTask,
  openAgentAutomationTaskSurface,
  insertAgentClaimInventoryAudit,
  copyAgentClaimInventoryAudit,
  goToSourceTarget,
  insertClaimCitationTodo,
  agentReviewCommentState,
  setAgentReviewCommentStatus,
  setAgentReviewCommentNote,
  inputValue,
  setAgentEditAcceptanceStatus,
  reviseAgentAcceptanceItem,
  setAgentEditAcceptanceNote,
  applyAcceptedAgentEdits,
  runAgentLifecycleTask,
  setAgentLifecycleTaskStatus,
  insertAgentLifecycleTaskBrief,
  copyAgentLifecycleTaskBrief,
  setAgentLifecycleTaskNote,
  insertAgentSectionBrief,
  draftAgentSectionWithDocsLive,
  insertAgentSectionDraftRestorePoint,
  draftAgentSectionHistoryWithDocsLive,
  copyAgentSectionDraftRestorePoint,
  openTransformTemplatesFromAgent,
  insertAgentTransformRecommendation,
  copyAgentTransformRecommendation,
  insertAgentDataNarrativeAudit,
  copyAgentDataNarrativeAudit,
  insertAgentApprovalGateScaffold,
  copyAgentApprovalGateScaffold,
  insertAgentReleaseEvidenceAuditPackage,
  copyAgentReleaseEvidenceAuditPackage,
  insertAgentHistoryAudit,
  copyAgentHistoryAudit,
  clearAgentHistory,
  replanAgentHistoryRun,
  appendAgentHistoryPacket,
  copyAgentHistoryPacket,
  removeAgentHistoryRun,
  agentRunHistoryIntentSummary,
  agentRunHistoryEvidenceSummary,
  agentRunHistoryOutlineSummary,
  agentRunHistorySectionDraftSummary,
  agentRunHistoryAutomationSummary,
  agentRunHistorySourcePackSummary,
  agentRunHistoryTaskStateSummary,
  syncAgentProviderProfile,
  formatOllamaModelOption,
  confirmOllamaModelSelection,
  refreshOllamaModels,
  applyAgentProviderResponse,
  importLocalAgentResponse,
  copyLocalAgentResponsePath,
  runAgentControlAction,
} = _ctx;
</script>
