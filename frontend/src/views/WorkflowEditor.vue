<template>
  <div class="workflow-editor">
    <header class="toolbar">
      <div class="toolbar-left">
        <span class="eyebrow"><WorkflowIcon :size="13" />{{ t('workflow.eyebrow', 'Authoring flow') }}</span>
        <h1>{{ t('workflow.title', 'Workflow Editor') }}</h1>
        <span class="workflow-name" :title="workflow?.name || ''">{{ workflow?.name || t('workflow.untitled', 'Untitled') }}</span>
        <span v-if="workflowDirty" class="dirty-indicator" :title="t('workflow.unsaved', 'Unsaved changes')"></span>
      </div>
      <div class="toolbar-right">
        <div class="icon-actions">
          <button class="icon-command" :title="t('workflow.new', 'New workflow')" :aria-label="t('workflow.new', 'New workflow')" @click="requestNewWorkflow">
            <FilePlus2 :size="16" />
          </button>
          <button class="icon-command" :disabled="loadingWorkflow" :title="t('workflow.open', 'Open workflow')" :aria-label="t('workflow.open', 'Open workflow')" @click="requestOpenWorkflow">
            <FolderOpen :size="16" />
          </button>
          <button class="icon-command" :disabled="runningWorkflow" :title="t('workflow.run', 'Run workflow')" :aria-label="t('workflow.run', 'Run workflow')" @click="runCurrentWorkflow">
            <Play :size="16" />
          </button>
          <button class="icon-command" :title="t('workflow.validate', 'Validate workflow')" :aria-label="t('workflow.validate', 'Validate workflow')" @click="validateAndOpenInspector">
            <ShieldCheck :size="16" />
          </button>
        </div>
        <button class="btn btn-primary btn-sm" :disabled="savingWorkflow || !workflow" @click="saveWorkflow">
          <Save :size="14" />
          {{ savingWorkflow ? t('workflow.saving', 'Saving') : t('workflow.save', 'Save') }}
        </button>
        <button class="btn btn-secondary btn-sm" :disabled="!workflow" @click="exportJSON">
          <Download :size="14" />
          {{ t('workflow.export', 'Export') }}
        </button>
        <button class="validation-pill" :class="validationStatusClass" @click="openInspector('validation')">
          <CheckCircle2 v-if="validationResult?.valid" :size="13" />
          <CircleAlert v-else :size="13" />
          {{ validationStatusLabel }}
        </button>
        <button class="icon-command inspector-toggle" :title="t('workflow.inspector', 'Open inspector')" :aria-label="t('workflow.inspector', 'Open inspector')" @click="compactInspectorOpen = true">
          <PanelRightOpen :size="16" />
        </button>
      </div>
    </header>

    <main class="editor-body">
      <aside class="node-palette">
        <div class="panel-title">
          <span class="panel-heading"><Blocks :size="15" />{{ t('workflow.nodes', 'Nodes') }}</span>
          <strong>{{ filteredNodeCount }}/{{ nodeTypes.length }}</strong>
        </div>
        <label class="palette-search">
          <Search :size="14" />
          <input v-model="paletteQuery" :placeholder="t('workflow.search-nodes', 'Search nodes')" :aria-label="t('workflow.search-nodes', 'Search nodes')" />
        </label>
        <div class="palette-list">
          <section v-for="category in nodeCategories" :key="category.id" class="palette-category">
            <h2>{{ category.label }}</h2>
            <button
              v-for="nodeType in category.nodes"
              :key="nodeType.node_type"
              class="palette-node"
              draggable="true"
              :title="nodeTypeDescription(nodeType.node_type)"
              @click="addNodeFromPalette(nodeType)"
              @dragstart="onDragStart($event, nodeType)"
            >
              <span class="node-icon">{{ getNodeIcon(nodeType.node_type) }}</span>
              <span class="palette-copy">
                <strong>{{ nodeTypeLabel(nodeType.node_type) }}</strong>
                <small>{{ nodeTypeDescription(nodeType.node_type) }}</small>
              </span>
              <Plus :size="13" />
            </button>
          </section>
          <div v-if="filteredNodeCount === 0" class="palette-empty">
            <Search :size="20" />
            <span>{{ t('workflow.no-node-results', 'No matching nodes') }}</span>
          </div>
        </div>
      </aside>

      <section
        ref="canvasRef"
        class="canvas"
        :aria-label="t('workflow.canvas', 'Workflow canvas')"
        @drop="onDrop"
        @dragover.prevent
        @mousedown="onCanvasMouseDown"
      >
        <svg class="canvas-grid" width="100%" height="100%" aria-hidden="true">
          <defs>
            <pattern id="workflow-grid" width="24" height="24" patternUnits="userSpaceOnUse">
              <path d="M 24 0 L 0 0 0 24" fill="none" stroke="rgba(170,180,195,0.12)" stroke-width="1" />
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#workflow-grid)" />
        </svg>

        <svg class="connections" width="100%" height="100%" aria-hidden="true">
          <path
            v-for="(conn, index) in connections"
            :key="`${conn.sourceNodeId}:${conn.targetNodeId}:${index}`"
            :d="connectionPath(conn)"
            fill="none"
            stroke="var(--brand)"
            stroke-width="2"
          />
        </svg>

        <div v-if="nodes.length === 0" class="canvas-empty">
          <WorkflowIcon :size="26" />
          <strong>{{ t('workflow.empty-canvas', 'No nodes on canvas') }}</strong>
          <button class="btn btn-primary btn-sm" @click.stop="addDefaultFlow()"><Plus :size="14" />{{ t('workflow.add-start-flow', 'Add start flow') }}</button>
        </div>

        <article
          v-for="node in nodes"
          :key="node.id"
          class="workflow-node"
          :class="[{ selected: selectedNode?.id === node.id }, nodeRunClass(node), 'node-type-' + node.node_type]"
          :style="{ left: node.x + 'px', top: node.y + 'px' }"
          @mousedown.stop="onNodeMouseDown($event, node)"
          @click.stop="selectNode(node)"
        >
          <header class="node-header">
            <span class="node-icon">{{ getNodeIcon(node.node_type) }}</span>
            <strong>{{ node.label }}</strong>
            <span
              v-if="nodeRunBadge(node)"
              class="node-run-badge"
              :class="nodeRunOutcome(node)"
              :title="nodeRunTooltip(node)"
            >
              {{ nodeRunBadge(node) }}
            </span>
          </header>
          <div class="node-body">
            <span>{{ node.node_type }}</span>
            <small v-if="nodeRunDetail(node)" class="node-run-detail">{{ nodeRunDetail(node) }}</small>
            <button
              v-if="node.node_type !== 'end'"
              class="node-port output"
              :title="t('workflow.connect-node', 'Connect node')"
              :aria-label="t('workflow.connect-node', 'Connect node')"
              @mousedown.stop="startConnection($event, node)"
            ></button>
          </div>
        </article>

        <div class="canvas-status">
          <span>{{ t('workflow.node-count', '{count} nodes', { count: nodes.length }) }}</span>
          <span>{{ t('workflow.connection-count', '{count} connections', { count: connections.length }) }}</span>
        </div>
      </section>

      <aside class="properties-panel" :class="{ 'compact-open': compactInspectorOpen }">
        <header class="inspector-header">
          <div class="inspector-tabs" :aria-label="t('workflow.inspector', 'Inspector')">
            <button :class="{ active: inspectorTab === 'properties' }" @click="inspectorTab = 'properties'">
              <SlidersHorizontal :size="14" />{{ t('workflow.properties', 'Properties') }}
            </button>
            <button :class="{ active: inspectorTab === 'validation' }" @click="inspectorTab = 'validation'">
              <ShieldCheck :size="14" />{{ t('workflow.validation', 'Validation') }}
              <span v-if="validationResult" class="tab-count" :class="{ danger: !validationResult.valid }">{{ validationResult.error_count + validationResult.warning_count }}</span>
            </button>
            <button :class="{ active: inspectorTab === 'execution' }" @click="inspectorTab = 'execution'">
              <Play :size="14" />{{ t('workflow.execution', 'Run') }}
            </button>
          </div>
          <button class="icon-command inspector-close" :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="compactInspectorOpen = false"><X :size="15" /></button>
        </header>

        <div class="inspector-content">
          <section v-if="inspectorTab === 'properties'" class="inspector-section">
            <template v-if="selectedNode">
              <div class="section-title-row">
                <div>
                  <span class="eyebrow">{{ t('workflow.node-properties', 'Node properties') }}</span>
                  <h2>{{ nodeTypeLabel(selectedNode.node_type) }}</h2>
                </div>
                <code>{{ selectedNode.id }}</code>
              </div>

              <label class="property-group">
                <span>{{ t('workflow.field.label', 'Label') }}</span>
                <input v-model="selectedNode.label" class="input" @input="markWorkflowDirty" />
              </label>

              <div v-for="field in getConfigFields(selectedNode.node_type)" :key="field" class="property-group">
                <span>{{ configFieldLabel(field) }}</span>
                <select
                  v-if="field === 'event_id' && selectedNode.node_type === 'trigger_event'"
                  class="input"
                  :value="selectedNode.config[field] || ''"
                  @change="updateStoryEvent(($event.target as HTMLSelectElement).value)"
                >
                  <option value="" disabled>{{ t('workflow.select-event', 'Select event') }}</option>
                  <option v-for="event in storyEvents" :key="event.event_id" :value="event.event_id">
                    {{ event.event_id }} / {{ event.event_type }}
                  </option>
                </select>
                <input
                  v-else-if="field === 'event_type' && selectedNode.node_type === 'trigger_event'"
                  class="input mono-input"
                  :value="selectedNode.config[field] || ''"
                  readonly
                />
                <textarea
                  v-else-if="isLongField(field)"
                  class="input"
                  rows="4"
                  :value="selectedNode.config[field]"
                  @input="updateConfig(field, ($event.target as HTMLTextAreaElement).value)"
                ></textarea>
                <label v-else-if="isWorkflowBooleanField(selectedNode.node_type, field)" class="switch-row">
                  <span>{{ t('workflow.enabled', 'Enabled') }}</span>
                  <input type="checkbox" :checked="selectedNode.config[field]" @change="updateConfig(field, ($event.target as HTMLInputElement).checked)" />
                </label>
                <input
                  v-else
                  class="input"
                  :class="{ 'mono-input': !isNumericField(field) }"
                  :type="isNumericField(field) ? 'number' : 'text'"
                  :step="numericFieldStep(field)"
                  :value="selectedNode.config[field]"
                  @input="updateConfigFromInput(field, $event)"
                />
              </div>

              <div class="connection-editor">
                <div class="section-title-row compact">
                  <div><span class="eyebrow">{{ t('workflow.connections', 'Connections') }}</span><h3>{{ t('workflow.outgoing', 'Outgoing') }}</h3></div>
                  <span class="count-chip">{{ selectedNode.connections.length }}</span>
                </div>
                <div v-if="selectedNode.connections.length" class="connection-list">
                  <div v-for="targetId in selectedNode.connections" :key="targetId">
                    <Link2 :size="13" />
                    <span><strong>{{ connectionTargetLabel(targetId) }}</strong><small>{{ targetId }}</small></span>
                    <button :title="t('workflow.remove-connection', 'Remove connection')" :aria-label="t('workflow.remove-connection', 'Remove connection')" @click="removeConnection(targetId)"><X :size="13" /></button>
                  </div>
                </div>
                <p v-else class="muted-copy">{{ t('workflow.no-connections', 'No outgoing connections') }}</p>
              </div>

              <button class="btn btn-danger delete-node" @click="requestDeleteNode"><Trash2 :size="14" />{{ t('workflow.delete-node', 'Delete node') }}</button>
            </template>

            <template v-else-if="workflow">
              <div class="section-title-row">
                <div><span class="eyebrow">{{ t('workflow.workflow-properties', 'Workflow properties') }}</span><h2>{{ workflow.name }}</h2></div>
                <FileJson :size="18" />
              </div>
              <label class="property-group">
                <span>{{ t('workflow.field.name', 'Name') }}</span>
                <input v-model="workflow.name" class="input" @input="markWorkflowDirty" />
              </label>
              <label class="property-group">
                <span>{{ t('workflow.field.id', 'Workflow ID') }}</span>
                <input v-model="workflow.id" class="input mono-input" spellcheck="false" @input="markWorkflowDirty" />
              </label>
              <label class="property-group">
                <span>{{ t('workflow.field.start-node', 'Start node') }}</span>
                <input :value="workflow.start_node_id || t('workflow.not-set', 'Not set')" class="input mono-input" readonly />
              </label>
              <dl class="workflow-stats">
                <div><dt>{{ t('workflow.nodes', 'Nodes') }}</dt><dd>{{ nodes.length }}</dd></div>
                <div><dt>{{ t('workflow.connections', 'Connections') }}</dt><dd>{{ connections.length }}</dd></div>
                <div><dt>{{ t('workflow.file', 'File') }}</dt><dd>{{ currentWorkflowPath || t('workflow.not-saved', 'Not saved') }}</dd></div>
              </dl>
              <div class="empty-selection"><MousePointer2 :size="20" /><span>{{ t('workflow.no-node-selected', 'No node selected') }}</span></div>
            </template>
          </section>

          <section v-else-if="inspectorTab === 'validation'" class="inspector-section">
            <div class="section-title-row">
              <div><span class="eyebrow">{{ t('workflow.validation', 'Validation') }}</span><h2>{{ validationStatusLabel }}</h2></div>
              <button class="btn btn-secondary btn-sm" @click="validateCurrentWorkflow"><ShieldCheck :size="14" />{{ t('workflow.run-check', 'Run check') }}</button>
            </div>

            <div v-if="validationResult" class="validation-summary" :class="{ invalid: !validationResult.valid }">
              <CheckCircle2 v-if="validationResult.valid" :size="17" />
              <CircleAlert v-else :size="17" />
              <span><strong>{{ validationResult.valid ? t('workflow.ready-to-export', 'Ready to export') : t('workflow.needs-attention', 'Needs attention') }}</strong><small>{{ t('workflow.issue-summary', '{errors} errors · {warnings} warnings', { errors: validationResult.error_count, warnings: validationResult.warning_count }) }}</small></span>
            </div>

            <div v-if="validationMessage" class="validation-message">{{ validationMessage }}</div>

            <div v-if="validationResult?.issues.length" class="issue-list">
              <div v-for="(issue, index) in validationResult.issues" :key="`${issue.code}-${index}`" class="issue-item" :class="issue.severity">
                <CircleAlert :size="14" />
                <div><span>{{ validationSeverityLabel(issue.severity) }} · {{ issue.code }}</span><strong>{{ issue.node_id || t('workflow.workflow-scope', 'Workflow') }}</strong><p>{{ workflowIssueMessage(issue) }}</p></div>
              </div>
            </div>

            <div v-else-if="!validationResult" class="inspector-empty"><ShieldCheck :size="24" /><span>{{ t('workflow.validation-empty', 'Validation has not run yet') }}</span></div>
            <div v-else class="inspector-empty success"><CheckCircle2 :size="24" /><span>{{ t('workflow.validation-clean', 'No workflow issues detected') }}</span></div>
          </section>

          <section v-else class="inspector-section execution-panel">
            <div class="section-title-row">
              <div><span class="eyebrow">{{ t('workflow.execution', 'Execution') }}</span><h2>{{ executionHeading }}</h2></div>
              <button class="btn btn-primary btn-sm" :disabled="runningWorkflow" @click="runCurrentWorkflow"><Play :size="14" />{{ runningWorkflow ? t('workflow.running', 'Running') : t('workflow.run', 'Run') }}</button>
            </div>

            <div class="run-context-panel" :class="{ enabled: runContext.enabled }">
              <label class="run-context-toggle">
                <span><strong>{{ t('workflow.preview-context', 'Preview context') }}</strong><small>{{ t('workflow.preview-context-state', 'Use authored score and relationship inputs') }}</small></span>
                <input v-model="runContext.enabled" type="checkbox" />
              </label>
              <div class="run-context-presets">
                <button v-for="preset in runContextPresets" :key="preset.id" class="context-preset-btn" type="button" @click="applyRunContextPreset(preset)">{{ preset.label }}</button>
                <button class="context-preset-btn matrix" type="button" :disabled="runningWorkflow" @click="runPresetMatrix"><Grid2X2Check :size="12" />{{ t('workflow.run-matrix', 'Run matrix') }}</button>
              </div>
              <div v-if="runContext.enabled" class="run-context-grid">
                <label><span>{{ t('workflow.context.character', 'Character') }}</span><input v-model="runContext.character_id" class="input" /></label>
                <label><span>{{ t('workflow.context.eval-count', 'Evaluation count') }}</span><input v-model.number="runContext.evaluation_count" class="input" type="number" min="0" step="1" /></label>
                <label><span>{{ t('workflow.context.relationship', 'Relationship') }}</span><input v-model.number="runContext.relationship" class="input" type="number" min="-1" max="1" step="0.05" /></label>
                <label><span>{{ t('workflow.context.friendliness', 'Friendliness') }}</span><input v-model.number="runContext.friendliness" class="input" type="number" min="0" max="1" step="0.05" /></label>
                <label><span>{{ t('workflow.context.engagement', 'Engagement') }}</span><input v-model.number="runContext.engagement" class="input" type="number" min="0" max="1" step="0.05" /></label>
                <label><span>{{ t('workflow.context.creativity', 'Creativity') }}</span><input v-model.number="runContext.creativity" class="input" type="number" min="0" max="1" step="0.05" /></label>
                <label><span>{{ t('workflow.context.overall', 'Overall') }}</span><input v-model.number="runContext.overall_score" class="input" type="number" min="0" max="1" step="0.05" /></label>
                <label class="run-context-wide"><span>{{ t('workflow.context.already-triggered', 'Already triggered') }}</span><input v-model="runContext.already_triggered_events" class="input" :placeholder="t('workflow.context.triggered-placeholder', 'high_engagement')" /></label>
              </div>
            </div>

            <div v-if="executionReport" class="execution-summary" :class="{ complete: executionReport.completed }">
              <span><strong>{{ executionReport.completed ? t('workflow.completed', 'Completed') : t('workflow.stopped', 'Stopped') }}</strong><small>{{ t('workflow.execution-steps', '{count} steps · {reason}', { count: executionReport.steps.length, reason: executionReasonLabel(executionReport.stopped_reason) }) }}</small></span>
              <div class="coverage-row"><span>{{ t('workflow.coverage', 'Coverage') }}</span><strong>{{ formatCoverage(executionReport.coverage_percent) }}</strong><small>{{ t('workflow.covered-nodes', '{executed}/{total} nodes', { executed: executionReport.executed_node_count, total: executionReport.node_count }) }}</small></div>
              <div v-if="executionReport.unvisited_node_ids.length" class="unvisited-node-list"><span v-for="nodeId in executionReport.unvisited_node_ids" :key="nodeId">{{ nodeId }}</span></div>
            </div>

            <div v-if="presetMatrixReport" class="matrix-coverage-panel" :class="{ complete: presetMatrixReport.unvisited_node_ids.length === 0 }">
              <strong>{{ t('workflow.preset-matrix', 'Preset matrix') }}</strong>
              <span>{{ t('workflow.matrix-coverage', '{coverage} · {executed}/{total} nodes', { coverage: formatCoverage(presetMatrixReport.coverage_percent), executed: presetMatrixReport.executed_node_count, total: presetMatrixReport.node_count }) }}</span>
              <div class="matrix-run-list"><span v-for="run in presetMatrixReport.runs" :key="run.preset_id">{{ run.label }} {{ formatCoverage(run.coverage_percent) }}</span></div>
              <div v-if="presetMatrixReport.unvisited_node_ids.length" class="unvisited-node-list"><span v-for="nodeId in presetMatrixReport.unvisited_node_ids" :key="nodeId">{{ nodeId }}</span></div>
            </div>

            <div v-if="executionMessage" class="validation-message">{{ executionMessage }}</div>

            <div v-if="executionReport?.steps.length" class="trace-list">
              <div v-for="step in executionReport.steps" :key="`${step.step_index}-${step.node_id}`" class="trace-item" :class="{ 'trace-score': isEvaluationStep(step), 'trace-event': isTriggerEventStep(step), triggered: isTriggerEventTriggered(step) }">
                <span class="trace-index">{{ step.step_index + 1 }}</span>
                <strong>{{ step.label || step.node_id }}</strong>
                <small>{{ step.node_type }}{{ step.next_node_id ? ` → ${step.next_node_id}` : '' }}</small>
                <em v-if="step.stopped_reason">{{ executionReasonLabel(step.stopped_reason) }}</em>
                <div v-if="isEvaluationStep(step)" class="trace-diagnostics score-diagnostics">
                  <div class="diagnostic-row"><span>{{ t('workflow.metric', 'Metric') }}</span><strong>{{ metricLabel(stringValue(step.output.metric, 'overall')) }}</strong></div>
                  <div class="diagnostic-row"><span>{{ t('workflow.score', 'Score') }}</span><strong>{{ formatScore(step.output.score) }}</strong></div>
                  <div class="score-meter" aria-hidden="true"><i :style="{ width: scorePercent(step.output.score) }"></i></div>
                  <div class="diagnostic-row"><span>{{ t('workflow.threshold', 'Threshold') }}</span><strong>{{ formatThreshold(step.output.threshold) }}</strong></div>
                  <div class="diagnostic-row"><span>{{ t('workflow.source', 'Source') }}</span><strong>{{ executionSourceLabel(stringValue(step.output.source, 'unknown')) }}</strong></div>
                  <span class="gate-pill" :class="{ pass: step.output.passed === true, fail: step.output.passed === false }">{{ step.output.passed === true ? t('workflow.pass', 'Pass') : step.output.passed === false ? t('workflow.fail', 'Fail') : t('workflow.no-threshold', 'No threshold') }}</span>
                </div>
                <div v-if="isTriggerEventStep(step)" class="trace-diagnostics event-diagnostics">
                  <div class="event-state-row"><span class="event-status" :class="{ active: isTriggerEventTriggered(step) }">{{ isTriggerEventTriggered(step) ? t('workflow.triggered', 'Triggered') : t('workflow.blocked', 'Blocked') }}</span><strong>{{ stringValue(step.output.event_id, 'event') }}</strong></div>
                  <div class="diagnostic-row"><span>{{ t('workflow.type', 'Type') }}</span><strong>{{ stringValue(step.output.event_type, 'story_event') }}</strong></div>
                  <div class="diagnostic-row"><span>{{ t('workflow.metric', 'Metric') }}</span><strong>{{ metricLabel(eventMetric(step)) }}</strong></div>
                  <div class="diagnostic-row"><span>{{ t('workflow.actual', 'Actual') }}</span><strong>{{ formatScore(eventActualScore(step)) }}</strong></div>
                  <div v-if="eventBlockers(step).length" class="blocker-list"><span v-for="reason in eventBlockers(step)" :key="reason">{{ blockerLabel(reason) }}</span></div>
                </div>
                <div v-if="canChooseStep(step)" class="choice-debug">
                  <button v-for="(choice, index) in choiceOptionsFor(step)" :key="`${step.node_id}-${index}`" class="choice-debug-btn" @click="chooseWorkflowOption(step.node_id, index)">{{ choice }}</button>
                </div>
              </div>
            </div>

            <div v-else class="inspector-empty"><Play :size="24" /><span>{{ t('workflow.execution-empty', 'Run the workflow to inspect its trace') }}</span></div>
          </section>
        </div>
      </aside>
    </main>

    <Transition name="toast">
      <div v-if="notice" class="workflow-toast" :class="notice.type" role="status" aria-live="polite">
        <CheckCircle2 v-if="notice.type === 'success'" :size="16" />
        <CircleAlert v-else :size="16" />
        <span>{{ notice.message }}</span>
        <button :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="clearNotice"><X :size="14" /></button>
      </div>
    </Transition>

    <div v-if="pathDialogMode" class="modal-backdrop" @mousedown.self="closePathDialog">
      <section class="workflow-modal path-modal" role="dialog" aria-modal="true" :aria-labelledby="'workflow-path-dialog-title'">
        <header><div><span class="eyebrow">{{ pathDialogMode === 'open' ? t('workflow.open', 'Open workflow') : t('workflow.save', 'Save workflow') }}</span><h2 id="workflow-path-dialog-title">{{ pathDialogMode === 'open' ? t('workflow.choose-workflow', 'Choose a workflow') : t('workflow.choose-save-path', 'Choose a save path') }}</h2></div><button class="icon-command" :title="t('common.close', 'Close')" :aria-label="t('common.close', 'Close')" @click="closePathDialog"><X :size="15" /></button></header>
        <div v-if="pathDialogMode === 'open'" class="workflow-file-list">
          <button v-for="file in workflowFiles" :key="file.path" :class="{ active: pathDialogValue === file.path }" @click="pathDialogValue = file.path">
            <FileJson :size="15" />
            <span><strong>{{ file.name }}</strong><small>{{ file.path }}</small></span>
            <em>{{ t('workflow.node-count', '{count} nodes', { count: file.node_count }) }}</em>
          </button>
          <div v-if="workflowFiles.length === 0" class="modal-empty"><FolderOpen :size="22" /><span>{{ t('workflow.no-workflow-files', 'No workflow files found') }}</span></div>
        </div>
        <label class="property-group"><span>{{ t('workflow.relative-path', 'Relative workflow path') }}</span><input v-model="pathDialogValue" class="input mono-input" :placeholder="t('workflow.path-placeholder', 'workflow.json')" spellcheck="false" @keydown.enter="submitPathDialog" /></label>
        <footer><button class="btn btn-secondary btn-sm" @click="closePathDialog">{{ t('common.cancel', 'Cancel') }}</button><button class="btn btn-primary btn-sm" :disabled="!pathDialogValue.trim() || loadingWorkflow || savingWorkflow" @click="submitPathDialog">{{ pathDialogMode === 'open' ? t('workflow.open', 'Open') : t('workflow.save', 'Save') }}</button></footer>
      </section>
    </div>

    <div v-if="pendingAction" class="modal-backdrop" @mousedown.self="cancelPendingAction">
      <section class="workflow-modal confirm-modal" role="alertdialog" aria-modal="true" aria-labelledby="workflow-confirm-title">
        <CircleAlert :size="24" />
        <h2 id="workflow-confirm-title">{{ pendingAction === 'delete-node' ? t('workflow.delete-node-title', 'Delete this node?') : t('workflow.discard-title', 'Discard unsaved changes?') }}</h2>
        <p>{{ pendingAction === 'delete-node' ? t('workflow.delete-node-message', 'The node and every connection to it will be removed.') : t('workflow.discard-message', 'Changes since the last save will be lost.') }}</p>
        <footer><button class="btn btn-secondary btn-sm" @click="cancelPendingAction">{{ t('common.cancel', 'Cancel') }}</button><button class="btn btn-danger btn-sm" @click="confirmPendingAction">{{ pendingAction === 'delete-node' ? t('workflow.delete-node', 'Delete node') : t('workflow.discard', 'Discard') }}</button></footer>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from '../lib/i18n'
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { onBeforeRouteLeave, type NavigationGuardNext } from 'vue-router'
import {
  Blocks,
  CheckCircle2,
  CircleAlert,
  Download,
  FileJson,
  FilePlus2,
  FolderOpen,
  Grid2X2Check,
  Link2,
  MousePointer2,
  PanelRightOpen,
  Play,
  Plus,
  Save,
  Search,
  ShieldCheck,
  SlidersHorizontal,
  Trash2,
  Workflow as WorkflowIcon,
  X,
} from '@lucide/vue'
import { hasTauriRuntime, invokeCommand } from '../lib/tauri'
import { loadStoryEventCatalog, type StoryEventDefinition } from '../lib/storyEvents'
import {
  connectWorkflowNodes,
  createDefaultWorkflowFlow,
  createDefaultWorkflowNodeTypes,
  createWorkflowNode as buildWorkflowNode,
  findOpenWorkflowCanvasPosition,
  isWorkflowBooleanField,
  isWorkflowDocument,
  isWorkflowLongField as isLongField,
  isWorkflowNumericField as isNumericField,
  nextWorkflowNodeSequence,
  normalizeWorkflowPath,
  removeWorkflowNode,
  safeWorkflowFileName,
  synchronizeWorkflowDocument,
  workflowConfigFields,
  workflowConnectionPath as connectionPath,
  workflowConnections,
  workflowNodeAtPoint,
  workflowNodeIcon as getNodeIcon,
  workflowNumericFieldStep as numericFieldStep,
  WORKFLOW_NODE_HEIGHT as NODE_HEIGHT,
  WORKFLOW_NODE_WIDTH as NODE_WIDTH,
  type WorkflowFileSummary,
  type WorkflowNodeTypeInfo,
} from '../lib/workflowAuthoring'
import type {
  Workflow,
  WorkflowExecutionReport,
  WorkflowExecutionStep,
  WorkflowNode,
  WorkflowPresetMatrixReport,
  WorkflowRunContextForm,
  WorkflowRunContextPayload,
  WorkflowRunContextPreset,
  WorkflowValidationIssue,
  WorkflowValidationResult,
} from '../lib/workflowContract'
import {
  formatWorkflowCoverage as formatCoverage,
  formatWorkflowScore as formatScore,
  formatWorkflowThreshold,
  isWorkflowEvaluationStep as isEvaluationStep,
  isWorkflowTriggerEventStep as isTriggerEventStep,
  isWorkflowTriggerEventTriggered as isTriggerEventTriggered,
  lastWorkflowExecutionStep,
  workflowChoiceOptions as choiceOptionsFor,
  workflowEventActualScore as eventActualScore,
  workflowEventBlockers as eventBlockers,
  workflowEventMetric as eventMetric,
  workflowExecutionStepsByNode,
  workflowNodeRunClasses as executionNodeRunClasses,
  workflowNodeRunDetail as executionNodeRunDetail,
  workflowNodeRunOutcome as executionNodeRunOutcome,
  workflowScorePercent as scorePercent,
  workflowStepCanChoose as canChooseStep,
  workflowStringValue as stringValue,
} from '../lib/workflowExecutionPresentation'
import {
  aggregatePresetMatrixCoverage,
  runWorkflowLocally,
  validateWorkflowLocally,
  workflowRunContextPayloadFromValues,
} from '../lib/workflowPreview'

const { t } = useI18n()

type InspectorTab = 'properties' | 'validation' | 'execution'
type WorkflowPathDialogMode = 'open' | 'save'
type PendingWorkflowAction = 'new' | 'open' | 'leave' | 'delete-node'
type WorkflowNotice = { type: 'success' | 'error'; message: string }

const workflow = ref<Workflow | null>(null)
const nodes = ref<WorkflowNode[]>([])
const selectedNode = ref<WorkflowNode | null>(null)
const nodeTypes = ref<WorkflowNodeTypeInfo[]>([])
const storyEvents = ref<StoryEventDefinition[]>([])
const canvasRef = ref<HTMLDivElement>()
const validationResult = ref<WorkflowValidationResult | null>(null)
const validationMessage = ref('')
const executionReport = ref<WorkflowExecutionReport | null>(null)
const executionMessage = ref('')
const runningWorkflow = ref(false)
const loadingWorkflow = ref(false)
const savingWorkflow = ref(false)
const choiceSelections = ref<Record<string, number>>({})
const presetMatrixReport = ref<WorkflowPresetMatrixReport | null>(null)
const inspectorTab = ref<InspectorTab>('properties')
const compactInspectorOpen = ref(false)
const paletteQuery = ref('')
const workflowDirty = ref(false)
const currentWorkflowPath = ref('')
const workflowFiles = ref<WorkflowFileSummary[]>([])
const pathDialogMode = ref<WorkflowPathDialogMode | null>(null)
const pathDialogValue = ref('')
const pendingAction = ref<PendingWorkflowAction | null>(null)
const notice = ref<WorkflowNotice | null>(null)
const desktopRuntimeAvailable = hasTauriRuntime()
const fallbackNodeTypes = createDefaultWorkflowNodeTypes()
let noticeTimer: number | undefined
let pendingNavigation: NavigationGuardNext | null = null
const runContext = ref<WorkflowRunContextForm>({
  enabled: false,
  character_id: 'sakura',
  friendliness: 0.65,
  engagement: 0.85,
  creativity: 0.6,
  overall_score: 0.7,
  relationship: 0,
  evaluation_count: 2,
  already_triggered_events: '',
})

const runContextPresets = computed<WorkflowRunContextPreset[]>(() => [
  {
    id: 'unlock',
    label: t('workflow.preset.unlock', 'Unlock'),
    values: {
      character_id: 'sakura',
      friendliness: 0.72,
      engagement: 0.9,
      creativity: 0.62,
      overall_score: 0.75,
      relationship: 0.2,
      evaluation_count: 2,
      already_triggered_events: '',
    },
  },
  {
    id: 'low-score',
    label: t('workflow.preset.low-score', 'Low score'),
    values: {
      character_id: 'sakura',
      friendliness: 0.45,
      engagement: 0.45,
      creativity: 0.35,
      overall_score: 0.42,
      relationship: 0,
      evaluation_count: 2,
      already_triggered_events: '',
    },
  },
  {
    id: 'repeat-block',
    label: t('workflow.preset.repeat-block', 'Repeat block'),
    values: {
      character_id: 'sakura',
      friendliness: 0.72,
      engagement: 0.92,
      creativity: 0.65,
      overall_score: 0.76,
      relationship: 0.2,
      evaluation_count: 2,
      already_triggered_events: 'high_engagement',
    },
  },
])

let nextNodeId = 1
let draggingNode: WorkflowNode | null = null
let connectingFrom: WorkflowNode | null = null
let dragOffset = { x: 0, y: 0 }

const nodeCategories = computed(() => {
  const query = paletteQuery.value.trim().toLocaleLowerCase()
  const categories: Record<string, WorkflowNodeTypeInfo[]> = {}
  for (const nt of nodeTypes.value) {
    if (query && ![
      nt.node_type,
      nodeTypeLabel(nt.node_type),
      nodeTypeDescription(nt.node_type),
    ].some((value) => value.toLocaleLowerCase().includes(query))) continue
    if (!categories[nt.category]) categories[nt.category] = []
    categories[nt.category].push(nt)
  }
  return Object.entries(categories).map(([id, categoryNodes]) => ({
    id,
    label: nodeCategoryLabel(id),
    nodes: categoryNodes,
  }))
})

const filteredNodeCount = computed(() => nodeCategories.value.reduce((count, category) => count + category.nodes.length, 0))

const connections = computed(() => workflowConnections(nodes.value))

const executionStepsByNode = computed(() => workflowExecutionStepsByNode(executionReport.value))

const lastExecutionStep = computed(() => lastWorkflowExecutionStep(executionReport.value))

const validationStatusLabel = computed(() => {
  if (!validationResult.value) return t('workflow.not-checked', 'Not checked')
  if (validationResult.value.valid) {
    return validationResult.value.warning_count > 0
      ? t('workflow.warnings', 'Warnings')
      : t('workflow.valid', 'Valid')
  }
  return t('workflow.error-count', '{count} errors', { count: validationResult.value.error_count })
})

const validationStatusClass = computed(() => {
  if (!validationResult.value) return 'neutral'
  if (validationResult.value.valid) return validationResult.value.warning_count > 0 ? 'warning' : 'valid'
  return 'invalid'
})

const executionHeading = computed(() => {
  if (!executionReport.value) return t('workflow.not-run', 'Not run')
  return executionReport.value.completed ? t('workflow.completed', 'Completed') : t('workflow.stopped', 'Stopped')
})

function nodeTypeLabel(type: string): string {
  if (type === 'start') return t('workflow.node.start', 'Start')
  if (type === 'dialogue') return t('workflow.node.dialogue', 'Dialogue')
  if (type === 'choice') return t('workflow.node.choice', 'Choice')
  if (type === 'condition') return t('workflow.node.condition', 'Condition')
  if (type === 'set_variable') return t('workflow.node.set-variable', 'Set variable')
  if (type === 'set_flag') return t('workflow.node.set-flag', 'Set flag')
  if (type === 'llm_generate') return t('workflow.node.llm-generate', 'LLM generate')
  if (type === 'evaluation') return t('workflow.node.evaluation', 'Evaluation')
  if (type === 'trigger_event') return t('workflow.node.trigger-event', 'Trigger event')
  if (type === 'scene_change') return t('workflow.node.scene-change', 'Scene change')
  if (type === 'emotion_change') return t('workflow.node.emotion-change', 'Change emotion')
  if (type === 'relationship') return t('workflow.node.relationship', 'Relationship')
  if (type === 'narration') return t('workflow.node.narration', 'Narration')
  if (type === 'bgm') return t('workflow.node.bgm', 'BGM')
  if (type === 'sfx') return t('workflow.node.sfx', 'SFX')
  if (type === 'wait') return t('workflow.node.wait', 'Wait')
  if (type === 'random_branch') return t('workflow.node.random-branch', 'Random branch')
  if (type === 'sub_workflow') return t('workflow.node.sub-workflow', 'Sub workflow')
  if (type === 'camera') return t('workflow.node.camera', 'Camera')
  if (type === 'shake') return t('workflow.node.shake', 'Shake')
  if (type === 'end') return t('workflow.node.end', 'End')
  return type
}

function nodeTypeDescription(type: string): string {
  if (type === 'start') return t('workflow.node-desc.start', 'Workflow entry point')
  if (type === 'dialogue') return t('workflow.node-desc.dialogue', 'Show character dialogue')
  if (type === 'choice') return t('workflow.node-desc.choice', 'Present player choices')
  if (type === 'condition') return t('workflow.node-desc.condition', 'Branch by expression')
  if (type === 'set_variable') return t('workflow.node-desc.set-variable', 'Set a game variable')
  if (type === 'set_flag') return t('workflow.node-desc.set-flag', 'Set a game flag')
  if (type === 'llm_generate') return t('workflow.node-desc.llm-generate', 'Generate text with the active model')
  if (type === 'evaluation') return t('workflow.node-desc.evaluation', 'Read the latest conversation score')
  if (type === 'trigger_event') return t('workflow.node-desc.trigger-event', 'Trigger a score-aware story event')
  if (type === 'scene_change') return t('workflow.node-desc.scene-change', 'Switch the background scene')
  if (type === 'emotion_change') return t('workflow.node-desc.emotion-change', 'Change character emotion')
  if (type === 'relationship') return t('workflow.node-desc.relationship', 'Modify relationship score')
  if (type === 'narration') return t('workflow.node-desc.narration', 'Display narrator text')
  if (type === 'bgm') return t('workflow.node-desc.bgm', 'Control background music')
  if (type === 'sfx') return t('workflow.node-desc.sfx', 'Play a sound effect')
  if (type === 'wait') return t('workflow.node-desc.wait', 'Pause workflow execution')
  if (type === 'random_branch') return t('workflow.node-desc.random-branch', 'Randomly choose a branch')
  if (type === 'sub_workflow') return t('workflow.node-desc.sub-workflow', 'Delegate to another workflow')
  if (type === 'camera') return t('workflow.node-desc.camera', 'Control camera motion')
  if (type === 'shake') return t('workflow.node-desc.shake', 'Apply a screen shake effect')
  if (type === 'end') return t('workflow.node-desc.end', 'Workflow exit')
  return type
}

function nodeCategoryLabel(category: string): string {
  if (category === 'flow') return t('workflow.category.flow', 'Flow')
  if (category === 'content') return t('workflow.category.content', 'Content')
  if (category === 'logic') return t('workflow.category.logic', 'Logic')
  if (category === 'ai') return t('workflow.category.ai', 'AI')
  if (category === 'character') return t('workflow.category.character', 'Character')
  if (category === 'media') return t('workflow.category.media', 'Media')
  return category
}

function configFieldLabel(field: string): string {
  if (field === 'speaker_id') return t('workflow.field.speaker-id', 'Speaker ID')
  if (field === 'text') return t('workflow.field.text', 'Text')
  if (field === 'choices') return t('workflow.field.choices', 'Choices')
  if (field === 'condition') return t('workflow.field.condition', 'Condition')
  if (field === 'variable_name') return t('workflow.field.variable-name', 'Variable name')
  if (field === 'value') return t('workflow.field.value', 'Value')
  if (field === 'flag_name') return t('workflow.field.flag-name', 'Flag name')
  if (field === 'prompt') return t('workflow.field.prompt', 'Prompt')
  if (field === 'system_prompt') return t('workflow.field.system-prompt', 'System prompt')
  if (field === 'max_tokens') return t('workflow.field.max-tokens', 'Max tokens')
  if (field === 'use_llm') return t('workflow.field.use-llm', 'Use LLM')
  if (field === 'character_id') return t('workflow.field.character-id', 'Character ID')
  if (field === 'criteria') return t('workflow.field.criteria', 'Metric')
  if (field === 'threshold') return t('workflow.field.threshold', 'Threshold')
  if (field === 'event_id') return t('workflow.field.event-id', 'Event ID')
  if (field === 'event_type') return t('workflow.field.event-type', 'Event type')
  if (field === 'scene_id') return t('workflow.field.scene-id', 'Scene ID')
  if (field === 'emotion') return t('workflow.field.emotion', 'Emotion')
  if (field === 'delta') return t('workflow.field.delta', 'Delta')
  if (field === 'speaker') return t('workflow.field.speaker', 'Speaker')
  if (field === 'track_path') return t('workflow.field.track-path', 'Track path')
  if (field === 'action') return t('workflow.field.action', 'Action')
  if (field === 'volume') return t('workflow.field.volume', 'Volume')
  if (field === 'sound_path') return t('workflow.field.sound-path', 'Sound path')
  if (field === 'duration_ms') return t('workflow.field.duration-ms', 'Duration (ms)')
  if (field === 'weights') return t('workflow.field.weights', 'Weights')
  if (field === 'workflow_id') return t('workflow.field.workflow-id', 'Workflow ID')
  if (field === 'workflow_path') return t('workflow.field.workflow-path', 'Workflow path')
  if (field === 'target_x') return t('workflow.field.target-x', 'Target X')
  if (field === 'target_y') return t('workflow.field.target-y', 'Target Y')
  if (field === 'zoom') return t('workflow.field.zoom', 'Zoom')
  if (field === 'intensity') return t('workflow.field.intensity', 'Intensity')
  return field
}

function updateConfigFromInput(field: string, event: Event) {
  const value = (event.target as HTMLInputElement).value
  updateConfig(field, isNumericField(field) && value !== '' ? Number(value) : value)
}

function connectionTargetLabel(targetId: string): string {
  return nodes.value.find((node) => node.id === targetId)?.label || t('workflow.missing-target', 'Missing target')
}

function removeConnection(targetId: string) {
  if (!selectedNode.value) return
  selectedNode.value.connections = selectedNode.value.connections.filter((id) => id !== targetId)
  markWorkflowDirty()
}

function openInspector(tab: InspectorTab) {
  inspectorTab.value = tab
  compactInspectorOpen.value = true
}

function getConfigFields(nodeType: string): string[] {
  return workflowConfigFields(nodeTypes.value, nodeType)
}

function updateConfig(field: string, value: unknown) {
  if (selectedNode.value) {
    selectedNode.value.config[field] = value
    markWorkflowDirty()
  }
}

function updateStoryEvent(eventId: string) {
  if (!selectedNode.value) return
  const definition = storyEvents.value.find((event) => event.event_id === eventId)
  selectedNode.value.config.event_id = eventId
  selectedNode.value.config.event_type = definition?.event_type || ''
  markWorkflowDirty()
}

function markWorkflowDirty() {
  workflowDirty.value = true
  validationResult.value = null
  validationMessage.value = ''
  executionReport.value = null
  executionMessage.value = ''
  presetMatrixReport.value = null
  choiceSelections.value = {}
}

function showWorkflowNotice(message: string, type: WorkflowNotice['type'] = 'success') {
  if (noticeTimer !== undefined) window.clearTimeout(noticeTimer)
  notice.value = { type, message }
  noticeTimer = window.setTimeout(clearNotice, type === 'success' ? 4500 : 8000)
}

function clearNotice() {
  if (noticeTimer !== undefined) window.clearTimeout(noticeTimer)
  noticeTimer = undefined
  notice.value = null
}

function formatWorkflowError(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

function validationSeverityLabel(severity: string): string {
  return severity === 'error' ? t('workflow.severity.error', 'Error') : t('workflow.severity.warning', 'Warning')
}

function workflowIssueMessage(issue: WorkflowValidationIssue): string {
  if (issue.code === 'workflow_id_empty') return t('workflow.issue.workflow-id-empty', 'Workflow ID is required.')
  if (issue.code === 'workflow_name_empty') return t('workflow.issue.workflow-name-empty', 'Workflow name is required.')
  if (issue.code === 'workflow_empty') return t('workflow.issue.workflow-empty', 'The workflow must contain at least one node.')
  if (issue.code === 'node_id_empty') return t('workflow.issue.node-id-empty', 'Every node must have an ID.')
  if (issue.code === 'node_id_duplicate') return t('workflow.issue.node-id-duplicate', 'Node IDs must be unique.')
  if (issue.code === 'start_node_missing') return t('workflow.issue.start-node-missing', 'The workflow must include a start node.')
  if (issue.code === 'start_node_multiple') return t('workflow.issue.start-node-multiple', 'Multiple start nodes were found; only the configured start node is used.')
  if (issue.code === 'start_node_id_empty') return t('workflow.issue.start-node-id-empty', 'A start node must be configured.')
  if (issue.code === 'start_node_not_found') return t('workflow.issue.start-node-not-found', 'The configured start node does not exist.')
  if (issue.code === 'start_node_type_invalid') return t('workflow.issue.start-node-type-invalid', 'The configured start node must use the start type.')
  if (issue.code === 'end_node_missing') return t('workflow.issue.end-node-missing', 'The workflow has no end node.')
  if (issue.code === 'node_label_empty') return t('workflow.issue.node-label-empty', 'The node label is empty.')
  if (issue.code === 'node_type_unknown') return t('workflow.issue.node-type-unknown', 'The node type is not supported.')
  if (issue.code === 'node_config_missing') return t('workflow.issue.node-config-missing', 'A required node field is missing.')
  if (issue.code === 'node_event_unknown') return t('workflow.issue.node-event-unknown', 'The selected story event is not in the active project catalog.')
  if (issue.code === 'node_event_character_mismatch') return t('workflow.issue.node-event-character-mismatch', 'The selected story event is unavailable for this character.')
  if (issue.code === 'node_state_key_invalid') return t('workflow.issue.node-state-key-invalid', 'A state key is invalid.')
  if (issue.code === 'node_condition_invalid') return t('workflow.issue.node-condition-invalid', 'The condition expression is invalid.')
  if (issue.code === 'connection_empty') return t('workflow.issue.connection-empty', 'A connection has no target.')
  if (issue.code === 'connection_self') return t('workflow.issue.connection-self', 'A node cannot connect to itself.')
  if (issue.code === 'connection_target_missing') return t('workflow.issue.connection-target-missing', 'A connection points to a missing node.')
  if (issue.code === 'connection_duplicate') return t('workflow.issue.connection-duplicate', 'The same connection appears more than once.')
  if (issue.code === 'node_unreachable') return t('workflow.issue.node-unreachable', 'The node cannot be reached from the configured start node.')
  return issue.message
}

function executionReasonLabel(reason: string | null | undefined): string {
  if (!reason || reason === 'ready') return t('workflow.reason.ready', 'Ready')
  if (reason === 'completed') return t('workflow.reason.completed', 'Completed')
  if (reason === 'awaiting_choice') return t('workflow.reason.awaiting-choice', 'Awaiting choice')
  if (reason === 'no_next_node') return t('workflow.reason.no-next-node', 'No next node')
  if (reason === 'max_steps_reached') return t('workflow.reason.max-steps', 'Step limit reached')
  if (reason === 'validation_failed') return t('workflow.reason.validation-failed', 'Validation failed')
  return reason
}

function metricLabel(metric: string): string {
  if (metric === 'friendliness') return t('workflow.context.friendliness', 'Friendliness')
  if (metric === 'engagement') return t('workflow.context.engagement', 'Engagement')
  if (metric === 'creativity') return t('workflow.context.creativity', 'Creativity')
  if (metric === 'overall' || metric === 'overall_score') return t('workflow.context.overall', 'Overall')
  return metric
}

function executionSourceLabel(source: string): string {
  if (source === 'run_context_evaluation') return t('workflow.source.preview-context', 'Preview context')
  if (source === 'local_preview') return t('workflow.source.local-preview', 'Local preview')
  if (source === 'neutral_no_chat_session') return t('workflow.source.no-chat-session', 'No chat session')
  if (source === 'chat_session_evaluation') return t('workflow.source.chat-session', 'Chat session')
  return source
}

function blockerLabel(reason: string): string {
  if (reason === 'local_preview_no_chat_session') return t('workflow.blocker.no-chat-session', 'No chat session is available in local preview.')
  if (reason === 'event_rule_missing') return t('workflow.blocker.rule-missing', 'The event has no trigger rule.')
  if (reason === 'event_type_mismatch') return t('workflow.blocker.type-mismatch', 'The configured event type does not match.')
  if (reason === 'character_not_allowed') return t('workflow.blocker.character-not-allowed', 'The event does not allow this character.')
  if (reason === 'already_triggered') return t('workflow.blocker.already-triggered', 'The non-repeatable event already ran.')
  if (reason.startsWith('relationship ')) return t('workflow.blocker.relationship', 'Relationship gate: {detail}', { detail: reason.slice('relationship '.length) })
  if (reason.startsWith('evaluation_count ')) return t('workflow.blocker.evaluation-count', 'Evaluation count gate: {detail}', { detail: reason.slice('evaluation_count '.length) })
  const scoreMatch = /^([a-z_]+)\s+(.+)$/.exec(reason)
  if (scoreMatch) return t('workflow.blocker.score', '{metric} gate: {detail}', { metric: metricLabel(scoreMatch[1]), detail: scoreMatch[2] })
  return reason
}

function syncWorkflowFromCanvas(): Workflow | null {
  if (!workflow.value) return null
  workflow.value = synchronizeWorkflowDocument(workflow.value, nodes.value)
  return workflow.value
}

async function validateCurrentWorkflow(): Promise<WorkflowValidationResult | null> {
  const currentWorkflow = syncWorkflowFromCanvas()
  if (!currentWorkflow) return null

  validationMessage.value = ''
  try {
    const result = await invokeCommand<WorkflowValidationResult>(
      'validate_workflow',
      { workflow: currentWorkflow },
      () => validateWorkflowLocally(currentWorkflow, storyEvents.value)
    )
    validationResult.value = result
    return result
  } catch (error) {
    validationMessage.value = t('workflow.error.validation', 'Validation failed: {error}', { error: formatWorkflowError(error) })
    return null
  }
}

async function validateAndOpenInspector() {
  openInspector('validation')
  await validateCurrentWorkflow()
}

async function ensureWorkflowIsValid(): Promise<boolean> {
  const result = await validateCurrentWorkflow()
  if (!result?.valid) {
    validationMessage.value = t('workflow.fix-before-save', 'Fix validation errors before saving or exporting.')
    openInspector('validation')
    return false
  }
  return true
}

async function runCurrentWorkflow() {
  openInspector('execution')
  choiceSelections.value = {}
  await executeWorkflowWithSelections()
}

async function chooseWorkflowOption(nodeId: string, index: number) {
  choiceSelections.value = { ...choiceSelections.value, [nodeId]: index }
  await executeWorkflowWithSelections()
}

async function executeWorkflowWithSelections() {
  const currentWorkflow = syncWorkflowFromCanvas()
  if (!currentWorkflow) return

  executionMessage.value = ''
  const validation = await validateCurrentWorkflow()
  if (!validation?.valid) {
    executionMessage.value = t('workflow.fix-before-run', 'Fix validation errors before running.')
    return
  }

  runningWorkflow.value = true
  try {
    const runContextPayload = workflowRunContextPayload()
    executionReport.value = await invokeCommand<WorkflowExecutionReport>(
      'execute_workflow',
      { workflow: currentWorkflow, maxSteps: 64, choiceSelections: choiceSelections.value, runContext: runContextPayload },
      () => runWorkflowLocally(currentWorkflow, {
        maxSteps: 64,
        selections: choiceSelections.value,
        context: runContextPayload,
        storyEvents: storyEvents.value,
      })
    )
  } catch (error) {
    executionMessage.value = t('workflow.error.execution', 'Execution failed: {error}', { error: formatWorkflowError(error) })
  } finally {
    runningWorkflow.value = false
  }
}

async function runPresetMatrix() {
  const currentWorkflow = syncWorkflowFromCanvas()
  if (!currentWorkflow) return

  choiceSelections.value = {}
  executionMessage.value = ''
  const validation = await validateCurrentWorkflow()
  if (!validation?.valid) {
    executionMessage.value = t('workflow.fix-before-run', 'Fix validation errors before running.')
    return
  }

  runningWorkflow.value = true
  try {
    const matrixRuns = []
    for (const preset of runContextPresets.value) {
      const runContextPayload = workflowRunContextPayloadFromValues(preset.values)
      const report = await invokeCommand<WorkflowExecutionReport>(
        'execute_workflow',
        { workflow: currentWorkflow, maxSteps: 64, choiceSelections: {}, runContext: runContextPayload },
        () => runWorkflowLocally(currentWorkflow, {
          maxSteps: 64,
          context: runContextPayload,
          storyEvents: storyEvents.value,
        })
      )
      matrixRuns.push({ preset, report })
    }
    executionReport.value = matrixRuns[matrixRuns.length - 1]?.report || null
    presetMatrixReport.value = aggregatePresetMatrixCoverage(currentWorkflow, matrixRuns)
  } catch (error) {
    executionMessage.value = t('workflow.error.execution', 'Execution failed: {error}', { error: formatWorkflowError(error) })
  } finally {
    runningWorkflow.value = false
  }
}

function formatThreshold(value: unknown): string {
  return formatWorkflowThreshold(value, t('workflow.none', 'None'))
}

function workflowRunContextPayload(): WorkflowRunContextPayload | null {
  if (!runContext.value.enabled) return null
  return workflowRunContextPayloadFromValues(runContext.value)
}

function applyRunContextPreset(preset: WorkflowRunContextPreset) {
  runContext.value = {
    enabled: true,
    ...preset.values,
  }
}

function nodeExecutionStep(node: WorkflowNode): WorkflowExecutionStep | null {
  return executionStepsByNode.value.get(node.id) || null
}

function nodeRunOutcome(node: WorkflowNode): string {
  return executionNodeRunOutcome(nodeExecutionStep(node))
}

function nodeRunClass(node: WorkflowNode): Record<string, boolean> {
  const step = nodeExecutionStep(node)
  return executionNodeRunClasses(node.id, step, lastExecutionStep.value)
}

function nodeRunBadge(node: WorkflowNode): string {
  const outcome = nodeRunOutcome(node)
  if (outcome === 'pass') return t('workflow.pass', 'Pass')
  if (outcome === 'fail') return t('workflow.fail', 'Fail')
  if (outcome === 'score') return t('workflow.score', 'Score')
  if (outcome === 'triggered') return t('workflow.event', 'Event')
  if (outcome === 'blocked') return t('workflow.blocked', 'Blocked')
  if (outcome === 'wait') return t('workflow.node.choice', 'Choice')
  if (outcome === 'done') return t('workflow.done', 'Done')
  if (outcome === 'ran') return t('workflow.ran', 'Ran')
  return ''
}

function nodeRunDetail(node: WorkflowNode): string {
  return executionNodeRunDetail(nodeExecutionStep(node), {
    reasonLabel: executionReasonLabel,
    nextNodeLabel: (nodeId) => t('workflow.next-node', 'Next: {node}', { node: nodeId }),
  })
}

function nodeRunTooltip(node: WorkflowNode): string {
  const badge = nodeRunBadge(node)
  const detail = nodeRunDetail(node)
  return detail ? `${badge}: ${detail}` : badge
}


function createNode(type: WorkflowNodeTypeInfo, x: number, y: number): WorkflowNode {
  const result = buildWorkflowNode(
    type,
    nodeTypeLabel(type.node_type),
    x,
    y,
    nodes.value,
    nextNodeId,
  )
  nextNodeId = result.nextNodeSequence
  return result.node
}

function selectNode(node: WorkflowNode) {
  selectedNode.value = node
  inspectorTab.value = 'properties'
  compactInspectorOpen.value = true
}

function deleteNode() {
  if (!selectedNode.value) return
  const id = selectedNode.value.id
  nodes.value = removeWorkflowNode(nodes.value, id)
  selectedNode.value = null
  markWorkflowDirty()
}

function requestDeleteNode() {
  if (selectedNode.value) pendingAction.value = 'delete-node'
}

function addNodeFromPalette(nodeType: WorkflowNodeTypeInfo) {
  const position = findOpenCanvasPosition()
  const node = createNode(nodeType, position.x + NODE_WIDTH / 2, position.y + 28)
  nodes.value.push(node)
  selectNode(node)
  markWorkflowDirty()
}

function findOpenCanvasPosition(): { x: number; y: number } {
  return findOpenWorkflowCanvasPosition(
    nodes.value,
    canvasRef.value?.clientWidth || 640,
    canvasRef.value?.clientHeight || 520,
  )
}

function addDefaultFlow(markDirty = true) {
  const catalog = nodeTypes.value.some((type) => type.node_type === 'start')
    && nodeTypes.value.some((type) => type.node_type === 'end')
    ? nodeTypes.value
    : fallbackNodeTypes
  const flow = createDefaultWorkflowFlow(
    catalog,
    nodeTypeLabel,
    canvasRef.value?.clientWidth || 640,
    nextNodeId,
  )
  nodes.value = flow.nodes
  nextNodeId = flow.nextNodeSequence
  if (workflow.value) {
    workflow.value.nodes = nodes.value
    workflow.value.start_node_id = flow.startNodeId
  }
  selectedNode.value = null
  if (markDirty) markWorkflowDirty()
}

function onDragStart(event: DragEvent, nodeType: WorkflowNodeTypeInfo) {
  event.dataTransfer?.setData('nodeType', JSON.stringify(nodeType))
}

function onDrop(event: DragEvent) {
  if (!event.dataTransfer || !canvasRef.value) return
  const rect = canvasRef.value.getBoundingClientRect()
  const x = event.clientX - rect.left
  const y = event.clientY - rect.top

  try {
    const nodeType = JSON.parse(event.dataTransfer.getData('nodeType')) as WorkflowNodeTypeInfo
    const node = createNode(nodeType, x, y)
    nodes.value.push(node)
    selectNode(node)
    markWorkflowDirty()
  } catch (e) {
    showWorkflowNotice(t('workflow.error.add-node', 'Unable to add node: {error}', { error: formatWorkflowError(e) }), 'error')
  }
}

function onNodeMouseDown(event: MouseEvent, node: WorkflowNode) {
  draggingNode = node
  dragOffset.x = event.clientX - node.x
  dragOffset.y = event.clientY - node.y

  const onMouseMove = (e: MouseEvent) => {
    if (!draggingNode || !canvasRef.value) return
    const rect = canvasRef.value.getBoundingClientRect()
    draggingNode.x = Math.min(
      Math.max(0, rect.width - NODE_WIDTH),
      Math.max(0, e.clientX - rect.left - (dragOffset.x - rect.left)),
    )
    draggingNode.y = Math.min(
      Math.max(0, rect.height - NODE_HEIGHT),
      Math.max(0, e.clientY - rect.top - (dragOffset.y - rect.top)),
    )
    markWorkflowDirty()
  }

  const onMouseUp = () => {
    draggingNode = null
    window.removeEventListener('mousemove', onMouseMove)
    window.removeEventListener('mouseup', onMouseUp)
  }

  window.addEventListener('mousemove', onMouseMove)
  window.addEventListener('mouseup', onMouseUp)
}

function onCanvasMouseDown() {
  selectedNode.value = null
}

function getNodeAtClientPoint(event: MouseEvent): WorkflowNode | undefined {
  if (!canvasRef.value) return undefined
  const rect = canvasRef.value.getBoundingClientRect()
  return workflowNodeAtPoint(nodes.value, {
    x: event.clientX - rect.left,
    y: event.clientY - rect.top,
  })
}

function startConnection(event: MouseEvent, node: WorkflowNode) {
  event.preventDefault()
  connectingFrom = node

  const onMouseUp = (e: MouseEvent) => {
    const target = getNodeAtClientPoint(e)
    if (connectingFrom && target) {
      const sourceId = connectingFrom.id
      const update = connectWorkflowNodes(nodes.value, sourceId, target.id)
      if (update.changed) {
        nodes.value = update.nodes
        if (selectedNode.value?.id === sourceId) {
          selectedNode.value = nodes.value.find((node) => node.id === sourceId) || null
        }
        markWorkflowDirty()
      }
    }
    connectingFrom = null
    window.removeEventListener('mouseup', onMouseUp)
  }

  window.addEventListener('mouseup', onMouseUp)
}

function createNewWorkflow(markDirty = true) {
  workflow.value = {
    id: `wf_${Date.now()}`,
    name: t('workflow.new-name', 'New Workflow'),
    nodes: [],
    start_node_id: '',
  }
  nodes.value = []
  selectedNode.value = null
  nextNodeId = 1
  currentWorkflowPath.value = ''
  validationResult.value = null
  validationMessage.value = ''
  executionReport.value = null
  executionMessage.value = ''
  presetMatrixReport.value = null
  addDefaultFlow(false)
  workflowDirty.value = markDirty
  inspectorTab.value = 'properties'
}

function requestNewWorkflow() {
  if (workflowDirty.value) {
    pendingAction.value = 'new'
    return
  }
  createNewWorkflow(true)
}

function requestOpenWorkflow() {
  if (workflowDirty.value) {
    pendingAction.value = 'open'
    return
  }
  void openWorkflowPicker()
}

async function openWorkflowPicker() {
  if (!desktopRuntimeAvailable) {
    await importWorkflowFile()
    return
  }
  await refreshWorkflowFiles()
  pathDialogMode.value = 'open'
  pathDialogValue.value = workflowFiles.value[0]?.path || ''
}

async function refreshWorkflowFiles() {
  try {
    workflowFiles.value = await invokeCommand<WorkflowFileSummary[]>('list_workflows', undefined, [])
  } catch (error) {
    workflowFiles.value = []
    showWorkflowNotice(t('workflow.error.list-files', 'Unable to list workflows: {error}', { error: formatWorkflowError(error) }), 'error')
  }
}

function closePathDialog() {
  pathDialogMode.value = null
  pathDialogValue.value = ''
}

async function submitPathDialog() {
  const path = normalizeWorkflowPath(pathDialogValue.value, pathDialogMode.value === 'save')
  if (!path || !pathDialogMode.value) return
  const mode = pathDialogMode.value
  if (mode === 'open') await loadWorkflowPath(path)
  else await saveWorkflowPath(path)
}

async function loadWorkflowPath(path: string) {
  loadingWorkflow.value = true
  try {
    const loaded = await invokeCommand<Workflow>('load_workflow', { path })
    applyLoadedWorkflow(loaded, path)
    closePathDialog()
    await validateCurrentWorkflow()
    showWorkflowNotice(t('workflow.notice.loaded', 'Loaded {name}', { name: loaded.name }))
  } catch (error) {
    showWorkflowNotice(t('workflow.error.load', 'Unable to load workflow: {error}', { error: formatWorkflowError(error) }), 'error')
  } finally {
    loadingWorkflow.value = false
  }
}

function applyLoadedWorkflow(loaded: Workflow, path: string) {
  workflow.value = loaded
  nodes.value = loaded.nodes
  selectedNode.value = null
  nextNodeId = nextWorkflowNodeSequence(nodes.value)
  currentWorkflowPath.value = path
  workflowDirty.value = false
  inspectorTab.value = 'properties'
  compactInspectorOpen.value = false
  executionReport.value = null
  executionMessage.value = ''
  presetMatrixReport.value = null
}

async function importWorkflowFile() {
  try {
    const file = await chooseWorkflowJsonFile()
    if (!file) return
    const loaded = JSON.parse(await file.text()) as Workflow
    if (!isWorkflowDocument(loaded)) throw new Error(t('workflow.error.invalid-document', 'The selected file is not a workflow document.'))
    applyLoadedWorkflow(loaded, file.name)
    await validateCurrentWorkflow()
    showWorkflowNotice(t('workflow.notice.loaded', 'Loaded {name}', { name: loaded.name }))
  } catch (error) {
    showWorkflowNotice(t('workflow.error.load', 'Unable to load workflow: {error}', { error: formatWorkflowError(error) }), 'error')
  }
}

function chooseWorkflowJsonFile(): Promise<File | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input')
    input.type = 'file'
    input.accept = 'application/json,.json'
    input.addEventListener('change', () => resolve(input.files?.[0] || null), { once: true })
    input.click()
  })
}

async function saveWorkflow() {
  if (!workflow.value) return
  if (!(await ensureWorkflowIsValid())) return
  syncWorkflowFromCanvas()
  if (!desktopRuntimeAvailable) {
    localStorage.setItem('monogatari-workflow-draft', JSON.stringify(workflow.value))
    currentWorkflowPath.value = 'browser-draft.json'
    workflowDirty.value = false
    showWorkflowNotice(t('workflow.notice.browser-saved', 'Workflow saved as a browser draft'))
    return
  }
  if (currentWorkflowPath.value) {
    await saveWorkflowPath(currentWorkflowPath.value)
    return
  }
  pathDialogMode.value = 'save'
  pathDialogValue.value = `${safeWorkflowFileName(workflow.value.name)}.json`
}

async function saveWorkflowPath(path: string) {
  if (!workflow.value) return
  savingWorkflow.value = true
  try {
    await invokeCommand<void>('save_workflow', { workflow: workflow.value, path })
    currentWorkflowPath.value = path
    workflowDirty.value = false
    closePathDialog()
    await refreshWorkflowFiles()
    showWorkflowNotice(t('workflow.notice.saved', 'Workflow saved to {path}', { path }))
  } catch (error) {
    showWorkflowNotice(t('workflow.error.save', 'Unable to save workflow: {error}', { error: formatWorkflowError(error) }), 'error')
  } finally {
    savingWorkflow.value = false
  }
}

async function exportJSON() {
  if (!workflow.value) return
  if (!(await ensureWorkflowIsValid())) return
  syncWorkflowFromCanvas()
  const json = JSON.stringify(workflow.value, null, 2)
  const blob = new Blob([json], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `${safeWorkflowFileName(workflow.value.name)}.json`
  a.click()
  URL.revokeObjectURL(url)
  showWorkflowNotice(t('workflow.notice.exported', 'Workflow JSON exported'))
}

function confirmPendingAction() {
  const action = pendingAction.value
  pendingAction.value = null
  if (action === 'delete-node') {
    deleteNode()
    return
  }
  workflowDirty.value = false
  if (action === 'new') createNewWorkflow(true)
  else if (action === 'open') void openWorkflowPicker()
  else if (action === 'leave' && pendingNavigation) {
    const next = pendingNavigation
    pendingNavigation = null
    next()
  }
}

function cancelPendingAction() {
  if (pendingAction.value === 'leave' && pendingNavigation) {
    const next = pendingNavigation
    pendingNavigation = null
    next(false)
  }
  pendingAction.value = null
}

function handleBeforeUnload(event: BeforeUnloadEvent) {
  if (!workflowDirty.value) return
  event.preventDefault()
  event.returnValue = ''
}

function restoreBrowserDraft(): boolean {
  if (desktopRuntimeAvailable) return false
  const raw = localStorage.getItem('monogatari-workflow-draft')
  if (!raw) return false
  try {
    const draft = JSON.parse(raw) as Workflow
    if (!isWorkflowDocument(draft)) return false
    applyLoadedWorkflow(draft, 'browser-draft.json')
    return true
  } catch {
    return false
  }
}

onBeforeRouteLeave((_to, _from, next) => {
  if (!workflowDirty.value) {
    next()
    return
  }
  pendingNavigation = next
  pendingAction.value = 'leave'
})

onMounted(async () => {
  try {
    const [loadedNodeTypes, eventCatalog] = await Promise.all([
      invokeCommand<WorkflowNodeTypeInfo[]>('get_workflow_nodes', undefined, fallbackNodeTypes),
      loadStoryEventCatalog(),
    ])
    nodeTypes.value = loadedNodeTypes
    storyEvents.value = eventCatalog.events
  } catch (error) {
    nodeTypes.value = fallbackNodeTypes
    showWorkflowNotice(t('workflow.error.catalogs', 'Unable to load authoring catalogs: {error}', { error: formatWorkflowError(error) }), 'error')
  }
  if (!restoreBrowserDraft()) createNewWorkflow(false)
  if (desktopRuntimeAvailable) await refreshWorkflowFiles()
  window.addEventListener('beforeunload', handleBeforeUnload)
})

onUnmounted(() => {
  window.removeEventListener('beforeunload', handleBeforeUnload)
  clearNotice()
})
</script>

<style scoped>
.workflow-editor {
  display: flex;
  height: calc(100svh - 56px);
  min-height: 0;
  flex-direction: column;
  overflow: hidden;
  background: var(--surface-0);
}

.toolbar {
  display: flex;
  min-height: 58px;
  flex: 0 0 auto;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  padding: 9px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--surface-1);
}

.toolbar-left {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: 10px;
}

.eyebrow,
.panel-heading {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-tertiary);
  font-size: 9px;
  font-weight: 800;
  text-transform: uppercase;
}

.eyebrow svg,
.panel-heading svg {
  flex: 0 0 auto;
  color: var(--brand-light);
}

.toolbar-left h1 {
  flex: 0 0 auto;
  color: var(--text-primary);
  font-size: 17px;
  line-height: 1.2;
}

.workflow-name {
  max-width: 220px;
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: 10px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dirty-indicator {
  width: 6px;
  height: 6px;
  flex: 0 0 auto;
  border-radius: 50%;
  background: var(--warning);
}

.toolbar-right,
.icon-actions {
  display: flex;
  flex: 0 0 auto;
  align-items: center;
  gap: 6px;
}

.icon-actions {
  padding-right: 6px;
  border-right: 1px solid var(--border);
}

.btn {
  display: inline-flex;
  min-height: 34px;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.btn:disabled,
.icon-command:disabled,
.validation-pill:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.icon-command {
  display: inline-grid;
  width: 34px;
  height: 34px;
  flex: 0 0 auto;
  place-items: center;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-secondary);
  cursor: pointer;
}

.icon-command:hover:not(:disabled) {
  border-color: var(--border-strong);
  color: var(--text-primary);
}

.validation-pill {
  display: inline-flex;
  min-height: 28px;
  align-items: center;
  gap: 5px;
  padding: 3px 8px;
  border: 1px solid var(--border);
  border-radius: 999px;
  background: var(--surface-2);
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 9px;
  font-weight: 800;
}

.validation-pill.valid {
  border-color: color-mix(in srgb, var(--success) 40%, var(--border));
  color: var(--success);
}

.validation-pill.warning {
  border-color: color-mix(in srgb, var(--warning) 40%, var(--border));
  color: var(--warning);
}

.validation-pill.invalid {
  border-color: color-mix(in srgb, var(--danger) 40%, var(--border));
  color: var(--danger);
}

.inspector-toggle {
  display: none;
}

.editor-body {
  position: relative;
  display: grid;
  min-height: 0;
  flex: 1;
  grid-template-columns: 230px minmax(0, 1fr) 320px;
  overflow: hidden;
}

.node-palette,
.properties-panel {
  min-width: 0;
  min-height: 0;
  background: var(--surface-1);
}

.node-palette {
  display: grid;
  grid-template-rows: auto auto minmax(0, 1fr);
  gap: 10px;
  padding: 12px;
  border-right: 1px solid var(--border);
}

.panel-title {
  display: flex;
  min-height: 24px;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.panel-title strong {
  color: var(--brand-light);
  font-family: var(--font-mono);
  font-size: 9px;
}

.palette-search {
  display: grid;
  min-height: 32px;
  grid-template-columns: 18px minmax(0, 1fr);
  align-items: center;
  gap: 6px;
  padding: 0 8px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-tertiary);
}

.palette-search:focus-within {
  border-color: var(--brand);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--brand) 14%, transparent);
}

.palette-search input {
  min-width: 0;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text-primary);
  font: inherit;
  font-size: 10px;
}

.palette-search input::placeholder {
  color: var(--text-tertiary);
}

.palette-list {
  min-height: 0;
  overflow-y: auto;
  scrollbar-width: none;
}

.palette-list::-webkit-scrollbar {
  display: none;
}

.palette-category {
  margin-bottom: 14px;
}

.palette-category h2 {
  margin: 0 0 5px;
  color: var(--text-tertiary);
  font-size: 8px;
  font-weight: 800;
  text-transform: uppercase;
}

.palette-node {
  display: grid;
  width: 100%;
  min-height: 43px;
  grid-template-columns: 30px minmax(0, 1fr) 15px;
  align-items: center;
  gap: 8px;
  margin-bottom: 3px;
  padding: 6px 7px;
  border: 1px solid transparent;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-tertiary);
  cursor: grab;
  text-align: left;
}

.palette-node:hover {
  border-color: var(--border);
  background: var(--surface-2);
  color: var(--brand-light);
}

.node-icon {
  display: inline-flex;
  width: 30px;
  height: 26px;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  background: var(--surface-3);
  color: var(--brand-light);
  font-family: var(--font-mono);
  font-size: 9px;
  font-weight: 900;
}

.palette-copy {
  display: grid;
  min-width: 0;
  gap: 1px;
}

.palette-copy strong,
.palette-copy small {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.palette-copy strong {
  color: var(--text-primary);
  font-size: 10px;
}

.palette-copy small {
  color: var(--text-tertiary);
  font-size: 8px;
}

.palette-empty,
.canvas-empty,
.inspector-empty,
.modal-empty,
.empty-selection {
  display: grid;
  place-items: center;
  align-content: center;
  gap: 8px;
  color: var(--text-tertiary);
  text-align: center;
}

.palette-empty {
  min-height: 130px;
  font-size: 9px;
}

.canvas {
  position: relative;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  background: var(--surface-0);
}

.canvas-grid,
.connections {
  position: absolute;
  inset: 0;
  pointer-events: none;
}

.canvas-empty {
  position: absolute;
  inset: 0;
  z-index: 1;
}

.canvas-empty svg {
  color: var(--text-tertiary);
}

.canvas-empty strong {
  color: var(--text-secondary);
  font-size: 11px;
}

.canvas-status {
  position: absolute;
  left: 10px;
  bottom: 10px;
  z-index: 5;
  display: flex;
  gap: 8px;
  padding: 5px 7px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--surface-1) 92%, transparent);
  color: var(--text-tertiary);
  font-size: 8px;
}

.workflow-node {
  position: absolute;
  z-index: 2;
  width: 214px;
  height: 92px;
  overflow: visible;
  border: 1px solid var(--border);
  border-radius: var(--radius);
  background: var(--surface-1);
  box-shadow: var(--shadow);
  cursor: move;
}

.workflow-node.selected {
  border-color: var(--brand);
  box-shadow: var(--shadow-brand), var(--shadow);
}

.workflow-node.run-executed { border-color: color-mix(in srgb, var(--info) 42%, var(--border)); }
.workflow-node.run-current { box-shadow: 0 0 0 2px color-mix(in srgb, var(--info) 20%, transparent), var(--shadow); }
.workflow-node.run-pass { border-color: color-mix(in srgb, var(--success) 48%, var(--border)); }
.workflow-node.run-fail { border-color: color-mix(in srgb, var(--danger) 48%, var(--border)); }
.workflow-node.run-wait { border-color: color-mix(in srgb, var(--warning) 48%, var(--border)); }
.workflow-node.run-score { border-color: color-mix(in srgb, var(--info) 48%, var(--border)); }

.node-header {
  display: flex;
  min-height: 44px;
  align-items: center;
  gap: 8px;
  padding: 8px 9px;
  overflow: hidden;
  border-bottom: 1px solid var(--border);
}

.node-header strong {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.node-run-badge {
  max-width: 70px;
  flex: 0 0 auto;
  margin-left: auto;
  overflow: hidden;
  padding: 3px 6px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--text-tertiary) 14%, transparent);
  color: var(--text-tertiary);
  font-size: 8px;
  font-weight: 900;
  text-overflow: ellipsis;
  text-transform: uppercase;
  white-space: nowrap;
}

.node-run-badge.pass,
.node-run-badge.triggered,
.node-run-badge.done { background: color-mix(in srgb, var(--success) 14%, transparent); color: var(--success); }
.node-run-badge.fail,
.node-run-badge.blocked { background: color-mix(in srgb, var(--danger) 14%, transparent); color: var(--danger); }
.node-run-badge.wait { background: color-mix(in srgb, var(--warning) 14%, transparent); color: var(--warning); }
.node-run-badge.score { background: color-mix(in srgb, var(--info) 14%, transparent); color: var(--info); }

.node-body {
  position: relative;
  display: grid;
  height: 47px;
  align-content: center;
  gap: 2px;
  padding: 0 30px 0 11px;
  overflow: hidden;
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: 9px;
}

.node-body > span,
.node-run-detail {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.node-run-detail {
  color: var(--text-secondary);
  font-size: 8px;
}

.node-port {
  position: absolute;
  top: 50%;
  right: -8px;
  width: 15px;
  height: 15px;
  border: 2px solid var(--surface-0);
  border-radius: 50%;
  background: var(--brand);
  cursor: crosshair;
  transform: translateY(-50%);
}

.node-type-narration { border-left: 3px solid var(--narrative); }
.node-type-bgm { border-left: 3px solid #f472b6; }
.node-type-sfx { border-left: 3px solid #fb923c; }
.node-type-wait { border-left: 3px solid #94a3b8; }
.node-type-random_branch { border-left: 3px solid #4ade80; }
.node-type-sub_workflow { border-left: 3px solid var(--info); }
.node-type-camera { border-left: 3px solid #38bdf8; }
.node-type-shake { border-left: 3px solid var(--accent); }
.node-type-start .node-icon { color: var(--success); }
.node-type-choice .node-icon,
.node-type-condition .node-icon { color: var(--warning); }
.node-type-llm_generate .node-icon,
.node-type-evaluation .node-icon { color: var(--info); }
.node-type-relationship .node-icon,
.node-type-emotion_change .node-icon { color: var(--narrative); }
.node-type-end .node-icon { color: var(--danger); }

.properties-panel {
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  border-left: 1px solid var(--border);
}

.inspector-header {
  display: flex;
  min-height: 47px;
  align-items: center;
  gap: 6px;
  padding: 6px;
  border-bottom: 1px solid var(--border);
}

.inspector-tabs {
  display: grid;
  min-width: 0;
  flex: 1;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 2px;
}

.inspector-tabs button {
  display: flex;
  min-width: 0;
  min-height: 32px;
  align-items: center;
  justify-content: center;
  gap: 4px;
  padding: 4px 5px;
  border: 0;
  border-radius: 4px;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 8px;
  font-weight: 750;
}

.inspector-tabs button:hover {
  color: var(--text-primary);
}

.inspector-tabs button.active {
  background: var(--surface-3);
  color: var(--brand-light);
}

.tab-count {
  min-width: 15px;
  height: 15px;
  padding: 0 3px;
  border-radius: 999px;
  background: var(--surface-4);
  color: var(--warning);
  font-family: var(--font-mono);
  font-size: 7px;
  line-height: 15px;
}

.tab-count.danger { color: var(--danger); }
.inspector-close { display: none; }

.inspector-content {
  min-height: 0;
  overflow-y: auto;
}

.inspector-section {
  display: grid;
  align-content: start;
  gap: 13px;
  padding: 14px;
}

.section-title-row {
  display: flex;
  min-width: 0;
  align-items: flex-start;
  justify-content: space-between;
  gap: 10px;
  padding-bottom: 11px;
  border-bottom: 1px solid var(--border);
}

.section-title-row.compact {
  align-items: center;
  padding-bottom: 8px;
}

.section-title-row > div {
  min-width: 0;
}

.section-title-row h2,
.section-title-row h3 {
  margin-top: 3px;
  overflow: hidden;
  color: var(--text-primary);
  font-size: 13px;
  line-height: 1.25;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.section-title-row h3 { font-size: 11px; }
.section-title-row code { max-width: 90px; overflow: hidden; color: var(--text-tertiary); font-size: 8px; text-overflow: ellipsis; white-space: nowrap; }
.section-title-row > svg { flex: 0 0 auto; color: var(--brand-light); }

.property-group {
  display: grid;
  min-width: 0;
  gap: 5px;
  color: var(--text-secondary);
  font-size: 9px;
  font-weight: 700;
}

.property-group > span {
  color: var(--text-secondary);
  font-size: 9px;
  font-weight: 750;
}

.property-group .input,
.inspector-section > .property-group .input {
  min-height: 34px;
  padding: 7px 9px;
  font-size: 10px;
}

.mono-input {
  font-family: var(--font-mono);
}

.switch-row,
.run-context-toggle {
  display: flex;
  min-height: 38px;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  cursor: pointer;
}

.switch-row input,
.run-context-toggle input {
  position: relative;
  width: 36px;
  height: 20px;
  flex: 0 0 auto;
  appearance: none;
  border: 1px solid var(--border-strong);
  border-radius: 999px;
  background: var(--surface-3);
  cursor: pointer;
}

.switch-row input::before,
.run-context-toggle input::before {
  position: absolute;
  top: 3px;
  left: 3px;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--text-tertiary);
  content: '';
  transition: transform var(--transition-fast), background var(--transition-fast);
}

.switch-row input:checked,
.run-context-toggle input:checked {
  border-color: var(--brand-dark);
  background: color-mix(in srgb, var(--brand) 25%, var(--surface-3));
}

.switch-row input:checked::before,
.run-context-toggle input:checked::before {
  background: var(--brand-light);
  transform: translateX(16px);
}

.connection-editor {
  display: grid;
  gap: 8px;
  padding-top: 3px;
}

.count-chip {
  display: inline-grid;
  min-width: 24px;
  height: 21px;
  place-items: center;
  padding: 0 5px;
  border: 1px solid var(--border);
  border-radius: 999px;
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: 8px;
}

.connection-list {
  display: grid;
  border-top: 1px solid var(--border);
}

.connection-list > div {
  display: grid;
  min-width: 0;
  grid-template-columns: 17px minmax(0, 1fr) 26px;
  align-items: center;
  gap: 7px;
  padding: 7px 0;
  border-bottom: 1px solid var(--border);
}

.connection-list svg { color: var(--brand-light); }
.connection-list > div > span { display: grid; min-width: 0; gap: 1px; }
.connection-list strong,
.connection-list small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.connection-list strong { color: var(--text-primary); font-size: 9px; }
.connection-list small { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 7px; }
.connection-list button { display: grid; width: 26px; height: 26px; place-items: center; border: 0; border-radius: 4px; background: transparent; color: var(--text-tertiary); cursor: pointer; }
.connection-list button:hover { background: var(--surface-3); color: var(--danger); }

.delete-node {
  width: 100%;
  margin-top: 4px;
}

.workflow-stats {
  display: grid;
  margin: 0;
  border-top: 1px solid var(--border);
}

.workflow-stats > div {
  display: grid;
  min-width: 0;
  grid-template-columns: 92px minmax(0, 1fr);
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid var(--border);
}

.workflow-stats dt { color: var(--text-tertiary); font-size: 8px; font-weight: 800; text-transform: uppercase; }
.workflow-stats dd { margin: 0; overflow: hidden; color: var(--text-primary); font-family: var(--font-mono); font-size: 8px; text-align: right; text-overflow: ellipsis; white-space: nowrap; }
.empty-selection { min-height: 110px; font-size: 9px; }

.validation-summary,
.execution-summary {
  display: grid;
  grid-template-columns: 20px minmax(0, 1fr);
  align-items: start;
  gap: 8px;
  padding: 10px;
  border-left: 2px solid var(--success);
  background: color-mix(in srgb, var(--success) 7%, transparent);
}

.validation-summary.invalid,
.execution-summary:not(.complete) {
  border-left-color: var(--danger);
  background: color-mix(in srgb, var(--danger) 7%, transparent);
}

.validation-summary > svg,
.execution-summary > svg { color: var(--success); }
.validation-summary.invalid > svg { color: var(--danger); }
.validation-summary > span,
.execution-summary > span { display: grid; min-width: 0; gap: 2px; }
.validation-summary strong,
.execution-summary strong { color: var(--text-primary); font-size: 10px; }
.validation-summary small,
.execution-summary small,
.muted-copy,
.validation-message { color: var(--text-tertiary); font-size: 9px; line-height: 1.4; }

.validation-message {
  padding: 8px 9px;
  border-left: 2px solid var(--warning);
  background: color-mix(in srgb, var(--warning) 7%, transparent);
  color: var(--warning);
  overflow-wrap: anywhere;
}

.issue-list,
.trace-list {
  display: grid;
  border-top: 1px solid var(--border);
}

.issue-item {
  display: grid;
  grid-template-columns: 20px minmax(0, 1fr);
  gap: 7px;
  padding: 9px 0;
  border-bottom: 1px solid var(--border);
}

.issue-item > svg { color: var(--warning); }
.issue-item.error > svg { color: var(--danger); }
.issue-item > div { display: grid; min-width: 0; gap: 2px; }
.issue-item span { color: var(--text-tertiary); font-size: 7px; font-weight: 800; text-transform: uppercase; }
.issue-item strong { overflow: hidden; color: var(--text-primary); font-family: var(--font-mono); font-size: 8px; text-overflow: ellipsis; white-space: nowrap; }
.issue-item p { color: var(--text-secondary); font-size: 9px; line-height: 1.4; }
.inspector-empty { min-height: 220px; padding: 20px; font-size: 9px; }
.inspector-empty.success svg { color: var(--success); }

.run-context-panel {
  display: grid;
  gap: 9px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.run-context-panel.enabled { border-color: color-mix(in srgb, var(--info) 42%, var(--border)); }
.run-context-toggle > span { display: grid; gap: 2px; }
.run-context-toggle strong { color: var(--text-primary); font-size: 9px; }
.run-context-toggle small { color: var(--text-tertiary); font-size: 7px; }
.run-context-presets { display: flex; min-width: 0; flex-wrap: wrap; gap: 5px; }

.context-preset-btn {
  display: inline-flex;
  min-width: 0;
  max-width: 100%;
  min-height: 26px;
  align-items: center;
  gap: 4px;
  padding: 4px 6px;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--info) 32%, var(--border));
  border-radius: var(--radius-sm);
  background: color-mix(in srgb, var(--info) 9%, transparent);
  color: var(--info);
  cursor: pointer;
  font-size: 7px;
  font-weight: 850;
  text-overflow: ellipsis;
  text-transform: uppercase;
  white-space: nowrap;
}

.context-preset-btn.matrix { border-color: color-mix(in srgb, var(--success) 35%, var(--border)); background: color-mix(in srgb, var(--success) 9%, transparent); color: var(--success); }
.context-preset-btn:disabled { cursor: not-allowed; opacity: 0.5; }

.run-context-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 7px;
}

.run-context-grid label { display: grid; min-width: 0; gap: 3px; }
.run-context-grid span { color: var(--text-tertiary); font-size: 7px; font-weight: 800; text-transform: uppercase; }
.run-context-grid .input { min-width: 0; height: 29px; padding: 4px 6px; font-size: 9px; }
.run-context-wide { grid-column: 1 / -1; }

.execution-summary { grid-template-columns: minmax(0, 1fr); }
.coverage-row { display: grid; min-width: 0; grid-template-columns: minmax(0, 1fr) auto; gap: 2px 8px; padding-top: 6px; border-top: 1px solid color-mix(in srgb, var(--border) 70%, transparent); }
.coverage-row span,
.coverage-row small { color: var(--text-tertiary); font-size: 7px; font-weight: 800; text-transform: uppercase; }
.coverage-row strong { color: var(--brand-light); font-size: 10px; }
.coverage-row small { grid-column: 1 / -1; }

.unvisited-node-list,
.matrix-run-list,
.blocker-list,
.choice-debug { display: flex; min-width: 0; flex-wrap: wrap; gap: 4px; }
.unvisited-node-list span,
.matrix-run-list span,
.blocker-list span { max-width: 100%; padding: 3px 5px; overflow: hidden; border-radius: 4px; background: var(--surface-3); color: var(--text-secondary); font-family: var(--font-mono); font-size: 7px; text-overflow: ellipsis; white-space: nowrap; }

.matrix-coverage-panel {
  display: grid;
  gap: 5px;
  padding: 9px;
  border-left: 2px solid var(--warning);
  background: color-mix(in srgb, var(--warning) 7%, transparent);
}

.matrix-coverage-panel.complete { border-left-color: var(--success); background: color-mix(in srgb, var(--success) 7%, transparent); }
.matrix-coverage-panel strong { color: var(--text-primary); font-size: 10px; }
.matrix-coverage-panel > span { color: var(--text-tertiary); font-size: 8px; }
.matrix-run-list span { color: var(--info); }

.trace-item {
  display: grid;
  grid-template-columns: 22px minmax(0, 1fr);
  align-items: center;
  gap: 3px 7px;
  padding: 9px 0;
  border-bottom: 1px solid var(--border);
}

.trace-index {
  display: grid;
  width: 21px;
  height: 21px;
  grid-row: span 2;
  place-items: center;
  border-radius: 4px;
  background: var(--surface-3);
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: 8px;
  font-weight: 900;
}

.trace-item > strong,
.trace-item > small,
.trace-item > em { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.trace-item > strong { color: var(--text-primary); font-size: 9px; }
.trace-item > small,
.trace-item > em { grid-column: 2; color: var(--text-tertiary); font-size: 7px; font-style: normal; }
.trace-item > em { color: var(--warning); }

.trace-diagnostics {
  display: grid;
  min-width: 0;
  grid-column: 1 / -1;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 6px;
  margin-top: 4px;
  padding: 7px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
}

.diagnostic-row { display: grid; min-width: 0; gap: 2px; }
.diagnostic-row span { color: var(--text-tertiary); font-size: 7px; font-weight: 800; text-transform: uppercase; }
.diagnostic-row strong { min-width: 0; overflow: hidden; color: var(--text-primary); font-size: 8px; text-overflow: ellipsis; white-space: nowrap; }
.score-meter { height: 4px; grid-column: 1 / -1; overflow: hidden; border-radius: 999px; background: var(--surface-3); }
.score-meter i { display: block; height: 100%; border-radius: inherit; background: var(--info); }

.gate-pill,
.event-status {
  display: inline-flex;
  width: max-content;
  min-height: 19px;
  align-items: center;
  justify-content: center;
  padding: 2px 6px;
  border-radius: 999px;
  background: color-mix(in srgb, var(--text-tertiary) 14%, transparent);
  color: var(--text-tertiary);
  font-size: 7px;
  font-weight: 900;
  text-transform: uppercase;
}

.gate-pill.pass,
.event-status.active { background: color-mix(in srgb, var(--success) 14%, transparent); color: var(--success); }
.gate-pill.fail { background: color-mix(in srgb, var(--danger) 14%, transparent); color: var(--danger); }
.event-state-row { display: grid; min-width: 0; grid-column: 1 / -1; grid-template-columns: auto minmax(0, 1fr); align-items: center; gap: 6px; }
.event-state-row strong { min-width: 0; overflow: hidden; color: var(--text-primary); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
.blocker-list { grid-column: 1 / -1; }
.blocker-list span { background: color-mix(in srgb, var(--danger) 10%, var(--surface-3)); color: var(--danger); }
.choice-debug { grid-column: 1 / -1; }

.choice-debug-btn {
  min-width: 0;
  max-width: 100%;
  padding: 4px 7px;
  overflow: hidden;
  border: 1px solid color-mix(in srgb, var(--info) 36%, var(--border));
  border-radius: 4px;
  background: color-mix(in srgb, var(--info) 10%, transparent);
  color: var(--info);
  cursor: pointer;
  font-size: 8px;
  font-weight: 750;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.workflow-toast {
  position: fixed;
  right: 18px;
  bottom: 18px;
  z-index: 100;
  display: grid;
  max-width: min(460px, calc(100vw - 32px));
  grid-template-columns: 18px minmax(0, 1fr) 26px;
  align-items: center;
  gap: 8px;
  padding: 9px 9px 9px 12px;
  border: 1px solid var(--border);
  border-radius: var(--radius-sm);
  background: var(--surface-2);
  color: var(--text-primary);
  box-shadow: var(--shadow-lg);
  font-size: 9px;
}

.workflow-toast.success > svg { color: var(--success); }
.workflow-toast.error > svg { color: var(--danger); }
.workflow-toast > span { min-width: 0; overflow-wrap: anywhere; }
.workflow-toast > button { display: grid; width: 26px; height: 26px; place-items: center; border: 0; border-radius: 4px; background: transparent; color: var(--text-tertiary); cursor: pointer; }
.workflow-toast > button:hover { background: var(--surface-4); color: var(--text-primary); }

.modal-backdrop {
  position: fixed;
  inset: 0;
  z-index: 120;
  display: grid;
  place-items: center;
  padding: 18px;
  background: rgba(0, 0, 0, 0.66);
}

.workflow-modal {
  display: grid;
  width: min(520px, 100%);
  max-height: min(680px, calc(100svh - 36px));
  gap: 14px;
  padding: 16px;
  overflow: hidden;
  border: 1px solid var(--border-strong);
  border-radius: var(--radius);
  background: var(--surface-1);
  box-shadow: var(--shadow-lg);
}

.workflow-modal > header,
.workflow-modal > footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.workflow-modal h2 { margin-top: 3px; color: var(--text-primary); font-size: 15px; }
.workflow-modal > footer { justify-content: flex-end; padding-top: 11px; border-top: 1px solid var(--border); }
.workflow-file-list { display: grid; min-height: 100px; max-height: 360px; overflow-y: auto; border-top: 1px solid var(--border); }
.workflow-file-list > button { display: grid; min-width: 0; grid-template-columns: 20px minmax(0, 1fr) auto; align-items: center; gap: 8px; padding: 9px 5px; border: 0; border-bottom: 1px solid var(--border); background: transparent; color: var(--text-tertiary); cursor: pointer; text-align: left; }
.workflow-file-list > button:hover,
.workflow-file-list > button.active { background: var(--surface-2); color: var(--brand-light); }
.workflow-file-list > button > span { display: grid; min-width: 0; gap: 2px; }
.workflow-file-list strong,
.workflow-file-list small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.workflow-file-list strong { color: var(--text-primary); font-size: 10px; }
.workflow-file-list small { color: var(--text-tertiary); font-family: var(--font-mono); font-size: 8px; }
.workflow-file-list em { color: var(--text-tertiary); font-size: 8px; font-style: normal; }
.modal-empty { min-height: 120px; font-size: 9px; }

.confirm-modal {
  width: min(390px, 100%);
  place-items: center;
  text-align: center;
}

.confirm-modal > svg { color: var(--warning); }
.confirm-modal h2 { margin: 0; }
.confirm-modal p { color: var(--text-secondary); font-size: 10px; line-height: 1.5; }
.confirm-modal > footer { width: 100%; }

.toast-enter-active,
.toast-leave-active { transition: opacity 0.16s ease, transform 0.16s ease; }
.toast-enter-from,
.toast-leave-to { opacity: 0; transform: translateY(5px); }

@media (max-width: 1120px) {
  .editor-body {
    grid-template-columns: 220px minmax(0, 1fr);
  }

  .properties-panel {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
    z-index: 40;
    display: none;
    width: min(360px, 100%);
    border-left: 1px solid var(--border-strong);
    box-shadow: var(--shadow-lg);
  }

  .properties-panel.compact-open { display: grid; }
  .inspector-toggle,
  .inspector-close { display: inline-grid; }
}

@media (max-width: 760px) {
  .workflow-editor {
    height: calc(100svh - 56px - 60px - env(safe-area-inset-bottom, 0px));
  }

  .toolbar {
    min-height: 96px;
    align-items: flex-start;
    flex-direction: column;
    gap: 7px;
    padding: 8px 10px;
  }

  .toolbar-left { width: 100%; }
  .toolbar-left h1 { font-size: 15px; }
  .toolbar-left .eyebrow { display: none; }
  .workflow-name { margin-left: auto; }
  .toolbar-right { width: 100%; overflow-x: auto; scrollbar-width: none; }
  .toolbar-right::-webkit-scrollbar { display: none; }
  .validation-pill { margin-left: auto; }

  .editor-body {
    grid-template-columns: 1fr;
    grid-template-rows: 176px minmax(280px, 1fr);
  }

  .node-palette {
    grid-row: 1;
    border-right: 0;
    border-bottom: 1px solid var(--border);
  }

  .canvas { grid-row: 2; }
  .properties-panel { left: 0; width: 100%; border-left: 0; }
  .palette-list { display: flex; gap: 12px; overflow-x: auto; overflow-y: hidden; }
  .palette-category {
    min-width: 188px;
    max-height: 130px;
    margin-bottom: 0;
    overflow-y: auto;
    scrollbar-width: none;
  }
  .palette-category::-webkit-scrollbar { display: none; }
  .palette-node { min-height: 38px; }
  .workflow-node { transform-origin: top left; }
  .workflow-toast { right: 12px; bottom: calc(68px + env(safe-area-inset-bottom, 0px)); }
}

@media (max-width: 430px) {
  .toolbar-right > .btn {
    width: 34px;
    padding: 0;
    overflow: hidden;
    font-size: 0;
  }

  .toolbar-right > .btn svg { margin: 0; }
  .workflow-name { max-width: 130px; }
  .validation-pill { max-width: 88px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .run-context-grid { grid-template-columns: 1fr; }
  .run-context-wide { grid-column: auto; }
  .workflow-modal { padding: 14px; }
}
</style>
