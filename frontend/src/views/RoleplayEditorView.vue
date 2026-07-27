<template>
  <div class="roleplay-editor">
    <header class="editor-header">
      <div>
        <span class="eyebrow">Live LLM NPC</span>
        <h1>Scene Roleplay</h1>
        <p>Author scene-bound character goals, evaluation evidence, scores, and deterministic routes.</p>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" :disabled="busy" @click="createRoleplay"><Plus :size="14" />New</button>
        <button class="btn btn-secondary btn-sm" :disabled="!draft || busy" @click="duplicateRoleplay"><Copy :size="14" />Duplicate</button>
        <button class="btn btn-secondary btn-sm" :disabled="busy" @click="reloadCatalog"><RotateCcw :size="14" />Reload</button>
        <button class="btn btn-secondary btn-sm" :disabled="!canPreview || busy" @click="previewRoleplay"><Play :size="14" />Playtest</button>
        <button class="btn btn-primary btn-sm" :disabled="!canSave || busy" @click="saveRoleplay"><Save :size="14" />{{ busy ? 'Working' : 'Save' }}</button>
      </div>
    </header>

    <section class="metrics-strip">
      <span><strong>{{ snapshot?.roleplay_count || 0 }}</strong> roleplays</span>
      <span><strong>{{ snapshot?.node_count || 0 }}</strong> scene nodes</span>
      <span><strong>{{ snapshot?.score_dimension_count || 0 }}</strong> score dimensions</span>
      <span><strong>{{ snapshot?.catalog_fingerprint.slice(0, 12) || 'unavailable' }}</strong> catalog</span>
      <span v-if="dirty" class="dirty-indicator">Unsaved changes</span>
    </section>

    <label class="mobile-roleplay-picker">
      <span>Roleplay</span>
      <select aria-label="Roleplay" :value="originalRoleplayId || ''" :disabled="busy" @change="selectRoleplayById(selectValue($event))">
        <option v-if="!originalRoleplayId" value="">Unsaved roleplay</option>
        <option v-for="entry in snapshot?.roleplays || []" :key="entry.definition.id" :value="entry.definition.id">
          {{ entry.definition.title }}
        </option>
      </select>
    </label>

    <div v-if="notice" class="notice" :class="notice.type">
      <strong>{{ notice.title }}</strong>
      <span>{{ notice.message }}</span>
      <button class="icon-button" title="Dismiss" @click="notice = null"><X :size="15" /></button>
    </div>

    <main class="editor-grid">
      <aside class="catalog-pane">
        <label class="search-field">
          <Search :size="15" />
          <input v-model="search" placeholder="Search roleplays" />
        </label>
        <div class="catalog-list">
          <button
            v-for="entry in filteredRoleplays"
            :key="entry.definition.id"
            class="catalog-row"
            :class="{ active: entry.definition.id === originalRoleplayId }"
            @click="selectRoleplay(entry)"
          >
            <strong>{{ entry.definition.title }}</strong>
            <span>{{ entry.definition.id }}</span>
            <small>{{ entry.definition.nodes.length }} nodes · {{ entry.definition.score_dimensions.length }} scores</small>
          </button>
          <div v-if="!filteredRoleplays.length" class="empty-list">No roleplays in this catalog.</div>
        </div>
      </aside>

      <section v-if="draft" class="work-pane">
        <nav class="editor-tabs" aria-label="Roleplay editor sections">
          <button v-for="item in tabs" :key="item.id" :class="{ active: tab === item.id }" @click="tab = item.id">
            <component :is="item.icon" :size="15" />{{ item.label }}
          </button>
        </nav>

        <div class="work-scroll">
          <section v-if="validationIssues.length" class="issues-panel">
            <AlertTriangle :size="17" />
            <div>
              <strong>{{ validationIssues.length }} blocking issues</strong>
              <p v-for="issue in validationIssues.slice(0, 6)" :key="issue">{{ issue }}</p>
            </div>
          </section>

          <template v-if="tab === 'story'">
            <section class="editor-section">
              <div class="section-heading">
                <div><span class="eyebrow">Identity</span><h2>Live story contract</h2></div>
                <code>{{ sourcePath }}</code>
              </div>
              <div class="form-grid">
                <label class="form-field"><span>Roleplay ID</span><input v-model="draft.id" :disabled="Boolean(originalRoleplayId)" /></label>
                <label class="form-field wide"><span>Title</span><input v-model="draft.title" /></label>
                <label class="form-field"><span>Start node</span>
                  <select v-model="draft.start_node_id"><option v-for="node in draft.nodes" :key="node.id" :value="node.id">{{ node.id }}</option></select>
                </label>
                <label class="form-field"><span>Exhaustion ending</span>
                  <select v-model="draft.exhaustion_ending_id"><option value="">Select ending</option><option v-for="ending in endings" :key="ending.id" :value="ending.id">{{ ending.title }}</option></select>
                </label>
                <label class="form-field"><span>Maximum total turns</span><input v-model.number="draft.max_total_turns" type="number" min="1" max="512" /></label>
              </div>
            </section>

            <section class="editor-section">
              <div class="section-heading"><div><span class="eyebrow">Inference</span><h2>Bounded two-stage generation</h2></div></div>
              <div class="form-grid four">
                <label class="form-field"><span>Context characters</span><input v-model.number="draft.inference.max_context_characters" type="number" min="1024" max="32000" step="256" /></label>
                <label class="form-field"><span>Recent turns</span><input v-model.number="draft.inference.max_recent_turns" type="number" min="1" max="16" /></label>
                <label class="form-field"><span>NPC output tokens</span><input v-model.number="draft.inference.npc_max_tokens" type="number" min="16" max="512" /></label>
                <label class="form-field"><span>Evaluator tokens</span><input v-model.number="draft.inference.evaluator_max_tokens" type="number" min="32" max="512" /></label>
              </div>
              <div class="pipeline">
                <span>Scene + character + knowledge</span><ArrowRight :size="15" /><span>NPC generation</span><ArrowRight :size="15" /><span>Independent evaluation</span><ArrowRight :size="15" /><span>Deterministic route</span>
              </div>
            </section>
          </template>

          <section v-else-if="tab === 'scores'" class="editor-section">
            <div class="section-heading">
              <div><span class="eyebrow">Global state</span><h2>Score dimensions</h2></div>
              <button class="btn btn-secondary btn-sm" @click="addDimension"><Plus :size="14" />Dimension</button>
            </div>
            <div class="repeat-list">
              <article v-for="(dimension, index) in draft.score_dimensions" :key="`${index}-${dimension.id}`" class="repeat-row dimension-row">
                <label class="form-field"><span>ID</span><input v-model="dimension.id" /></label>
                <label class="form-field"><span>Label</span><input v-model="dimension.label" /></label>
                <label class="form-field description"><span>Description</span><input v-model="dimension.description" /></label>
                <label class="form-field numeric"><span>Min</span><input v-model.number="dimension.min" type="number" /></label>
                <label class="form-field numeric"><span>Initial</span><input v-model.number="dimension.initial" type="number" /></label>
                <label class="form-field numeric"><span>Max</span><input v-model.number="dimension.max" type="number" /></label>
                <button class="icon-button danger" title="Remove dimension" :disabled="draft.score_dimensions.length === 1" @click="removeDimension(index)"><Trash2 :size="15" /></button>
              </article>
            </div>
          </section>

          <template v-else>
            <div class="node-layout">
              <aside class="node-list">
                <div class="node-list-head"><strong>Scene nodes</strong><button class="icon-button" title="Add node" @click="addNode"><Plus :size="15" /></button></div>
                <button v-for="node in draft.nodes" :key="node.id" :class="{ active: node.id === selectedNodeId }" @click="selectedNodeId = node.id">
                  <strong>{{ node.id || 'untitled' }}</strong><span>{{ sceneName(node.scene_id) }}</span>
                </button>
              </aside>

              <div v-if="selectedNode" class="node-work">
                <section v-if="tab === 'nodes'" class="editor-section">
                  <div class="section-heading">
                    <div><span class="eyebrow">Current node</span><h2>{{ selectedNode.id || 'Untitled node' }}</h2></div>
                    <button class="btn btn-danger btn-sm" :disabled="draft.nodes.length === 1" @click="removeSelectedNode"><Trash2 :size="14" />Delete node</button>
                  </div>
                  <div class="form-grid">
                    <label class="form-field"><span>Node ID</span><input v-model="selectedNode.id" @change="synchronizeNodeId" /></label>
                    <label class="form-field"><span>Scene</span><select v-model="selectedNode.scene_id"><option value="">Select scene</option><option v-for="scene in scenes" :key="scene.id" :value="scene.id">{{ scene.name }}</option></select></label>
                    <label class="form-field"><span>Primary NPC</span><select v-model="selectedNode.character_id"><option value="">Select character</option><option v-for="character in characters" :key="character.id" :value="character.id">{{ character.name }}</option></select></label>
                    <label class="form-field"><span>Initial emotion</span><input v-model="selectedNode.emotion" placeholder="neutral" /></label>
                    <label class="form-field numeric"><span>Minimum turns</span><input v-model.number="selectedNode.min_turns" type="number" min="1" /></label>
                    <label class="form-field numeric"><span>Maximum turns</span><input v-model.number="selectedNode.max_turns" type="number" min="1" /></label>
                    <label class="form-field full"><span>Opening narration</span><textarea v-model="selectedNode.opening_narration" rows="3" /></label>
                    <label class="form-field full"><span>Observable situation</span><textarea v-model="selectedNode.situation" rows="4" /></label>
                    <label class="form-field full"><span>Player goal</span><textarea v-model="selectedNode.player_goal" rows="3" /></label>
                    <label class="form-field full"><span>Primary NPC goal</span><textarea v-model="selectedNode.character_goal" rows="3" /></label>
                  </div>

                  <div class="subsection">
                    <div class="subsection-head"><strong>Scene participants</strong><small>Every selected supporting NPC gets an independent local motive.</small></div>
                    <div class="check-grid">
                      <label v-for="character in characters" :key="character.id" class="check-row">
                        <input type="checkbox" :checked="nodeHasSupportingCharacter(character.id)" :disabled="character.id === selectedNode.character_id" @change="toggleSupportingCharacter(character.id)" />
                        <span>{{ character.name }}</span>
                      </label>
                    </div>
                    <div v-for="characterId in selectedNode.supporting_character_ids" :key="characterId" class="participant-goal">
                      <strong>{{ characterName(characterId) }}</strong>
                      <input :value="selectedNode.participant_goals?.[characterId] || ''" placeholder="Scene-local motive" @input="setParticipantGoal(characterId, inputValue($event))" />
                    </div>
                  </div>

                  <div class="subsection">
                    <div class="subsection-head"><strong>Pinned knowledge</strong><small>Only checked entries enter this node's grounding context.</small></div>
                    <div class="check-grid">
                      <label v-for="entry in knowledge" :key="entry.id" class="check-row">
                        <input type="checkbox" :checked="selectedNode.knowledge_refs.includes(entry.id)" @change="toggleKnowledge(entry.id)" />
                        <span>{{ entry.title }}</span>
                      </label>
                    </div>
                  </div>

                  <div class="subsection">
                    <div class="subsection-head"><strong>Per-turn score rules</strong><button class="btn btn-secondary btn-xs" @click="addScoreRule"><Plus :size="13" />Rule</button></div>
                    <div v-for="(rule, index) in selectedNode.score_rules" :key="index" class="compact-row">
                      <select v-model="rule.dimension_id"><option v-for="dimension in draft.score_dimensions" :key="dimension.id" :value="dimension.id">{{ dimension.label }}</option></select>
                      <input v-model="rule.guidance" placeholder="Evaluation guidance" />
                      <input v-model.number="rule.max_delta_per_turn" class="short-input" type="number" min="0.1" max="10" step="0.1" />
                      <button class="icon-button danger" title="Remove score rule" @click="selectedNode.score_rules.splice(index, 1)"><Trash2 :size="14" /></button>
                    </div>
                  </div>

                  <div class="subsection">
                    <div class="subsection-head"><strong>Evidence gates</strong><button class="btn btn-secondary btn-xs" @click="addEvidenceRule"><Plus :size="13" />Evidence</button></div>
                    <div v-for="(rule, index) in selectedNode.evidence_rules" :key="index" class="compact-row evidence-row">
                      <input v-model="rule.id" placeholder="evidence_id" />
                      <input v-model="rule.description" placeholder="What exact player statement proves this?" />
                      <button class="icon-button danger" title="Remove evidence" @click="selectedNode.evidence_rules.splice(index, 1)"><Trash2 :size="14" /></button>
                    </div>
                  </div>

                  <label class="toggle-row">
                    <input type="checkbox" :checked="Boolean(selectedNode.relationship_rule)" @change="toggleRelationshipRule" />
                    <span><strong>Relationship evaluation</strong><small>Apply a bounded affinity delta to the active speaker.</small></span>
                  </label>
                  <div v-if="selectedNode.relationship_rule" class="compact-row relationship-row">
                    <input v-model="selectedNode.relationship_rule.guidance" placeholder="Relationship evaluation guidance" />
                    <input v-model.number="selectedNode.relationship_rule.max_delta_per_turn" class="short-input" type="number" min="0.01" max="0.5" step="0.01" />
                  </div>
                </section>

                <section v-else-if="tab === 'routes'" class="editor-section">
                  <div class="section-heading">
                    <div><span class="eyebrow">Deterministic state machine</span><h2>Transitions</h2></div>
                    <button class="btn btn-secondary btn-sm" @click="addTransition"><Plus :size="14" />Transition</button>
                  </div>
                  <article v-for="(transition, transitionIndex) in selectedNode.transitions" :key="transitionIndex" class="route-block">
                    <div class="route-head">
                      <input v-model="transition.id" placeholder="transition_id" />
                      <label>Priority <input v-model.number="transition.priority" type="number" /></label>
                      <select :value="transition.target.kind" @change="setTransitionTargetKind(transitionIndex, selectValue($event) as 'node' | 'ending')"><option value="node">Node</option><option value="ending">Ending</option></select>
                      <select v-if="transition.target.kind === 'node'" v-model="transition.target.node_id"><option v-for="node in draft.nodes" :key="node.id" :value="node.id">{{ node.id }}</option></select>
                      <select v-else v-model="transition.target.ending_id"><option v-for="ending in endings" :key="ending.id" :value="ending.id">{{ ending.title }}</option></select>
                      <button class="icon-button danger" title="Remove transition" @click="selectedNode.transitions.splice(transitionIndex, 1)"><Trash2 :size="14" /></button>
                    </div>
                    <div class="condition-list">
                      <div v-for="(condition, conditionIndex) in transition.conditions" :key="conditionIndex" class="condition-row">
                        <select :value="condition.kind" @change="replaceCondition(transitionIndex, conditionIndex, selectValue($event) as any)">
                          <option value="score_at_least">Score at least</option><option value="score_at_most">Score at most</option>
                          <option value="evidence_observed">Evidence observed</option>
                          <option value="relationship_at_least">Relationship at least</option><option value="relationship_at_most">Relationship at most</option>
                          <option value="node_turns_at_least">Node turns at least</option><option value="total_turns_at_least">Total turns at least</option>
                        </select>
                        <select v-if="'dimension_id' in condition" v-model="condition.dimension_id"><option v-for="dimension in draft.score_dimensions" :key="dimension.id" :value="dimension.id">{{ dimension.label }}</option></select>
                        <select v-if="'evidence_id' in condition" v-model="condition.evidence_id"><option v-for="evidence in selectedNode.evidence_rules" :key="evidence.id" :value="evidence.id">{{ evidence.id }}</option></select>
                        <select v-if="'character_id' in condition" v-model="condition.character_id"><option v-for="characterId in nodeParticipantIds" :key="characterId" :value="characterId">{{ characterName(characterId) }}</option></select>
                        <input v-if="'value' in condition" v-model.number="condition.value" type="number" step="0.1" />
                        <button class="icon-button danger" title="Remove condition" @click="transition.conditions.splice(conditionIndex, 1)"><Trash2 :size="14" /></button>
                      </div>
                      <button class="btn btn-secondary btn-xs" @click="addCondition(transitionIndex)"><Plus :size="13" />Condition</button>
                    </div>
                  </article>
                  <div v-if="!selectedNode.transitions.length" class="empty-state">No conditional transition. The timeout target will be used when the node turn limit is reached.</div>

                  <div class="timeout-row">
                    <div><strong>Timeout target</strong><small>Mandatory deterministic route when maximum node turns are exhausted.</small></div>
                    <select :value="selectedNode.timeout_target.kind" @change="setTimeoutTargetKind(selectValue($event) as 'node' | 'ending')"><option value="node">Node</option><option value="ending">Ending</option></select>
                    <select v-if="selectedNode.timeout_target.kind === 'node'" v-model="selectedNode.timeout_target.node_id"><option v-for="node in draft.nodes" :key="node.id" :value="node.id">{{ node.id }}</option></select>
                    <select v-else v-model="selectedNode.timeout_target.ending_id"><option v-for="ending in endings" :key="ending.id" :value="ending.id">{{ ending.title }}</option></select>
                  </div>
                </section>

                <section v-else class="editor-section">
                  <div class="section-heading"><div><span class="eyebrow">Containment and recovery</span><h2>Scene safety</h2></div></div>
                  <label class="toggle-row"><input type="checkbox" :checked="Boolean(selectedNode.intrusion_response)" @change="toggleIntrusionResponse" /><span><strong>Authored intrusion containment</strong><small>Control attempts never enter model context literally and cannot change story state.</small></span></label>
                  <div v-if="selectedNode.intrusion_response" class="safety-grid">
                    <label class="form-field"><span>Reality anchors, one per line</span><textarea :value="selectedNode.intrusion_response.reality_anchors.join('\n')" rows="5" @input="setIntrusionList('reality_anchors', inputValue($event))" /></label>
                    <label class="form-field"><span>In-world interpretations</span><textarea :value="selectedNode.intrusion_response.interpretations.join('\n')" rows="5" @input="setIntrusionList('interpretations', inputValue($event))" /></label>
                    <label class="form-field"><span>Redirects</span><textarea :value="selectedNode.intrusion_response.redirects.join('\n')" rows="5" @input="setIntrusionList('redirects', inputValue($event))" /></label>
                  </div>

                  <label class="toggle-row"><input type="checkbox" :checked="Boolean(selectedNode.response_guard)" @change="toggleResponseGuard" /><span><strong>Generated reply guard</strong><small>Reject identity drift, hidden-state leaks, and ungrounded output before display.</small></span></label>
                  <div v-if="selectedNode.response_guard" class="safety-grid">
                    <label class="form-field"><span>Forbidden markers</span><textarea :value="selectedNode.response_guard.forbidden_markers.join('\n')" rows="5" @input="setGuardList('forbidden_markers', inputValue($event))" /></label>
                    <label class="form-field"><span>Grounding markers</span><textarea :value="(selectedNode.response_guard.grounding_markers || []).join('\n')" rows="5" @input="setGuardList('grounding_markers', inputValue($event))" /></label>
                    <label class="form-field"><span>In-world recoveries</span><textarea :value="selectedNode.response_guard.recoveries.join('\n')" rows="5" @input="setGuardList('recoveries', inputValue($event))" /></label>
                    <label class="form-field numeric"><span>Minimum grounding terms</span><input v-model.number="selectedNode.response_guard.min_grounding_matches" type="number" min="1" /></label>
                    <label class="form-field numeric"><span>Maximum characters</span><input v-model.number="selectedNode.response_guard.max_characters" type="number" min="40" max="1000" /></label>
                    <label class="form-field numeric"><span>Maximum sentences</span><input v-model.number="selectedNode.response_guard.max_sentences" type="number" min="1" max="5" /></label>
                  </div>

                  <label class="toggle-row"><input type="checkbox" :checked="Boolean(selectedNode.fallback_evaluation)" @change="toggleFallbackEvaluation" /><span><strong>Provider-free fallback evidence</strong><small>Conservative markers for Quality replay; never substitutes for a successful live model turn.</small></span></label>
                  <div v-if="selectedNode.fallback_evaluation" class="subsection">
                    <div class="subsection-head"><strong>Fallback score signals</strong><button class="btn btn-secondary btn-xs" @click="addFallbackScore"><Plus :size="13" />Signal</button></div>
                    <div v-for="(signal, index) in selectedNode.fallback_evaluation.score_signals" :key="`score-${index}`" class="fallback-row">
                      <select v-model="signal.dimension_id"><option v-for="rule in selectedNode.score_rules" :key="rule.dimension_id" :value="rule.dimension_id">{{ rule.dimension_id }}</option></select>
                      <input :value="signal.positive_markers.join(', ')" placeholder="positive markers" @input="signal.positive_markers = commaList(inputValue($event))" />
                      <input :value="signal.negative_markers.join(', ')" placeholder="negative markers" @input="signal.negative_markers = commaList(inputValue($event))" />
                      <input v-model.number="signal.delta" class="short-input" type="number" min="0.1" step="0.1" />
                      <button class="icon-button danger" title="Remove signal" @click="selectedNode.fallback_evaluation!.score_signals.splice(index, 1)"><Trash2 :size="14" /></button>
                    </div>
                    <div class="subsection-head"><strong>Fallback evidence signals</strong><button class="btn btn-secondary btn-xs" @click="addFallbackEvidence"><Plus :size="13" />Signal</button></div>
                    <div v-for="(signal, index) in selectedNode.fallback_evaluation.evidence_signals" :key="`evidence-${index}`" class="fallback-row evidence">
                      <select v-model="signal.evidence_id"><option v-for="rule in selectedNode.evidence_rules" :key="rule.id" :value="rule.id">{{ rule.id }}</option></select>
                      <input :value="signal.markers.join(', ')" placeholder="required markers" @input="signal.markers = commaList(inputValue($event))" />
                      <button class="icon-button danger" title="Remove signal" @click="selectedNode.fallback_evaluation!.evidence_signals.splice(index, 1)"><Trash2 :size="14" /></button>
                    </div>
                  </div>
                </section>
              </div>
            </div>
          </template>
        </div>
      </section>

      <aside v-if="draft" class="inspector-pane">
        <section><span class="eyebrow">Runtime graph</span><div class="metric-row"><span>Nodes</span><strong>{{ draft.nodes.length }}</strong></div><div class="metric-row"><span>Transitions</span><strong>{{ transitionCount }}</strong></div><div class="metric-row"><span>Endings targeted</span><strong>{{ targetedEndingCount }}</strong></div></section>
        <section><span class="eyebrow">Active node</span><div class="metric-row"><span>Participants</span><strong>{{ nodeParticipantIds.length }}</strong></div><div class="metric-row"><span>Knowledge</span><strong>{{ selectedNode?.knowledge_refs.length || 0 }}</strong></div><div class="metric-row"><span>Evidence gates</span><strong>{{ selectedNode?.evidence_rules.length || 0 }}</strong></div></section>
        <section><span class="eyebrow">Transaction</span><p>{{ validationIssues.length ? `${validationIssues.length} issues block save.` : dirty ? 'Ready to validate and save.' : 'Catalog and draft are synchronized.' }}</p><small>NPC text, evaluator output, score, evidence, relationship, and route commit as one turn.</small></section>
        <button v-if="originalRoleplayId" class="btn btn-danger btn-sm delete-roleplay" :disabled="busy" @click="removeRoleplay"><Trash2 :size="14" />Delete roleplay</button>
      </aside>
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, markRaw, onMounted, onUnmounted, ref } from 'vue'
import { onBeforeRouteLeave, useRouter } from 'vue-router'
import {
  AlertTriangle, ArrowRight, BookOpen, Copy, GitBranch, ListTree, Play, Plus,
  RotateCcw, Save, Search, ShieldCheck, SlidersHorizontal, Trash2, X,
} from '@lucide/vue'
import {
  cloneSceneRoleplayDefinition,
  createRoleplayCondition,
  createRoleplayScoreDimension,
  createRoleplayTarget,
  createSceneRoleplayDraft,
  createSceneRoleplayNodeDraft,
  deleteSceneRoleplayDefinition,
  loadSceneRoleplayAuthoringCatalog,
  normalizeSceneRoleplayDefinition,
  saveSceneRoleplayDefinition,
  sceneRoleplayDraftSnapshot,
  validateSceneRoleplayDefinition,
  type SceneRoleplayAuthoringCatalog,
  type SceneRoleplayAuthoringEntry,
} from '../lib/sceneRoleplayAuthoring'
import type {
  RoleplayCondition,
  SceneRoleplayDefinition,
  SceneRoleplayNode,
} from '../lib/sceneRoleplay'
import type { RoleplayIntrusionResponse } from '../lib/sceneRoleplaySafety'
import { loadKnowledgeAuthoringCatalog, type KnowledgeEntryDefinition } from '../lib/knowledgeContent'
import {
  loadStoryCharacters, loadStoryEndings, loadStoryScenes,
  type StoryCharacterInfo, type StoryEndingInfo, type StorySceneInfo,
} from '../lib/storyContent'

type EditorTab = 'story' | 'scores' | 'nodes' | 'routes' | 'safety'

const router = useRouter()
const snapshot = ref<SceneRoleplayAuthoringCatalog | null>(null)
const draft = ref<SceneRoleplayDefinition | null>(null)
const originalRoleplayId = ref<string | null>(null)
const baseline = ref('')
const search = ref('')
const tab = ref<EditorTab>('story')
const selectedNodeId = ref('')
const busy = ref(false)
const notice = ref<{ type: 'success' | 'error'; title: string; message: string } | null>(null)
const scenes = ref<StorySceneInfo[]>([])
const characters = ref<StoryCharacterInfo[]>([])
const endings = ref<StoryEndingInfo[]>([])
const knowledge = ref<KnowledgeEntryDefinition[]>([])

const tabs = [
  { id: 'story' as const, label: 'Story', icon: markRaw(BookOpen) },
  { id: 'scores' as const, label: 'Scores', icon: markRaw(SlidersHorizontal) },
  { id: 'nodes' as const, label: 'Nodes', icon: markRaw(ListTree) },
  { id: 'routes' as const, label: 'Routes', icon: markRaw(GitBranch) },
  { id: 'safety' as const, label: 'Safety', icon: markRaw(ShieldCheck) },
]

const filteredRoleplays = computed(() => {
  const query = search.value.trim().toLowerCase()
  const entries = snapshot.value?.roleplays || []
  return query ? entries.filter(entry => `${entry.definition.id} ${entry.definition.title}`.toLowerCase().includes(query)) : entries
})
const selectedNode = computed(() => draft.value?.nodes.find(node => node.id === selectedNodeId.value) || draft.value?.nodes[0] || null)
const serializedDraft = computed(() => sceneRoleplayDraftSnapshot(draft.value))
const dirty = computed(() => serializedDraft.value !== baseline.value)
const sourcePath = computed(() => `roleplays/${draft.value?.id || 'new'}.json`)
const nodeParticipantIds = computed(() => selectedNode.value
  ? [...new Set([selectedNode.value.character_id, ...selectedNode.value.supporting_character_ids].filter(Boolean))]
  : [])
const transitionCount = computed(() => draft.value?.nodes.reduce((sum, node) => sum + node.transitions.length, 0) || 0)
const targetedEndingCount = computed(() => {
  if (!draft.value) return 0
  return new Set(draft.value.nodes.flatMap(node => [
    node.timeout_target.kind === 'ending' ? node.timeout_target.ending_id : '',
    ...node.transitions.map(route => route.target.kind === 'ending' ? route.target.ending_id : ''),
  ]).filter(Boolean)).size
})
const validationIssues = computed(() => draft.value
  ? [...validateSceneRoleplayDefinition(draft.value), ...referenceIssues(draft.value)]
  : ['No roleplay selected.'])
const canSave = computed(() => Boolean(snapshot.value && draft.value && dirty.value && !validationIssues.value.length))
const canPreview = computed(() => Boolean(originalRoleplayId.value && !dirty.value && !validationIssues.value.length))

function setDraft(definition: SceneRoleplayDefinition, originalId: string | null) {
  draft.value = cloneSceneRoleplayDefinition(definition)
  originalRoleplayId.value = originalId
  selectedNodeId.value = draft.value.start_node_id || draft.value.nodes[0]?.id || ''
  baseline.value = sceneRoleplayDraftSnapshot(draft.value)
}

function selectRoleplay(entry: SceneRoleplayAuthoringEntry) {
  if (entry.definition.id === originalRoleplayId.value || !confirmDiscard()) return
  setDraft(entry.definition, entry.definition.id)
}

function selectRoleplayById(id: string) {
  const entry = snapshot.value?.roleplays.find(candidate => candidate.definition.id === id)
  if (!entry) return
  selectRoleplay(entry)
}

function createRoleplay() {
  if (!confirmDiscard()) return
  const definition = createSceneRoleplayDraft(snapshot.value?.roleplays.map(entry => entry.definition.id) || [])
  hydrateNewDefinitionReferences(definition)
  setDraft(definition, null)
  baseline.value = ''
  tab.value = 'story'
}

function duplicateRoleplay() {
  if (!draft.value || !confirmDiscard()) return
  const copy = cloneSceneRoleplayDefinition(draft.value)
  const fresh = createSceneRoleplayDraft(snapshot.value?.roleplays.map(entry => entry.definition.id) || [])
  copy.id = fresh.id
  copy.title = `${copy.title} Copy`
  setDraft(copy, null)
  baseline.value = ''
}

async function loadCatalog(preferredId?: string | null) {
  busy.value = true
  try {
    const [next, nextScenes, nextCharacters, nextEndings, nextKnowledge] = await Promise.all([
      loadSceneRoleplayAuthoringCatalog(),
      loadStoryScenes(),
      loadStoryCharacters(),
      loadStoryEndings(),
      loadKnowledgeAuthoringCatalog(),
    ])
    snapshot.value = next
    scenes.value = nextScenes
    characters.value = nextCharacters
    endings.value = nextEndings
    knowledge.value = nextKnowledge.entries
    const target = next.roleplays.find(entry => entry.definition.id === preferredId) || next.roleplays[0]
    if (target) setDraft(target.definition, target.definition.id)
    else createRoleplay()
  } catch (error) {
    showNotice('error', 'Roleplay catalog unavailable', String(error))
  } finally {
    busy.value = false
  }
}

async function reloadCatalog() {
  if (!confirmDiscard()) return
  await loadCatalog(originalRoleplayId.value)
  showNotice('success', 'Roleplay catalog reloaded', 'Definitions and referenced project catalogs are current.')
}

async function saveRoleplay() {
  if (!draft.value || !snapshot.value || !canSave.value) return
  busy.value = true
  try {
    const normalized = normalizeSceneRoleplayDefinition(draft.value)
    const next = await saveSceneRoleplayDefinition(normalized, originalRoleplayId.value, snapshot.value.catalog_fingerprint)
    snapshot.value = next
    const saved = next.roleplays.find(entry => entry.definition.id === normalized.id)
    if (saved) setDraft(saved.definition, saved.definition.id)
    showNotice('success', 'Live roleplay saved', `${normalized.title} passed schema and project reference validation.`)
  } catch (error) {
    showNotice('error', 'Save rejected', String(error))
  } finally {
    busy.value = false
  }
}

async function removeRoleplay() {
  if (!snapshot.value || !originalRoleplayId.value) return
  const id = originalRoleplayId.value
  if (!window.confirm(`Delete roleplay "${id}"?`)) return
  busy.value = true
  try {
    const next = await deleteSceneRoleplayDefinition(id, snapshot.value.catalog_fingerprint)
    snapshot.value = next
    const target = next.roleplays[0]
    if (target) setDraft(target.definition, target.definition.id)
    else createRoleplay()
    showNotice('success', 'Roleplay deleted', `${id} was removed from the catalog.`)
  } catch (error) {
    showNotice('error', 'Delete rejected', String(error))
  } finally {
    busy.value = false
  }
}

async function previewRoleplay() {
  if (!originalRoleplayId.value || !canPreview.value) return
  await router.push({ path: '/game', query: { previewRoleplay: originalRoleplayId.value, authoring: '1' } })
}

function addDimension() {
  if (!draft.value) return
  draft.value.score_dimensions.push(createRoleplayScoreDimension(draft.value.score_dimensions.map(item => item.id)))
}

function removeDimension(index: number) {
  if (!draft.value || draft.value.score_dimensions.length === 1) return
  const [removed] = draft.value.score_dimensions.splice(index, 1)
  for (const node of draft.value.nodes) {
    node.score_rules = node.score_rules.filter(rule => rule.dimension_id !== removed.id)
    for (const transition of node.transitions) {
      transition.conditions = transition.conditions.filter(condition =>
        !('dimension_id' in condition) || condition.dimension_id !== removed.id)
    }
  }
}

function addNode() {
  if (!draft.value) return
  const existing = new Set(draft.value.nodes.map(node => node.id))
  let suffix = draft.value.nodes.length + 1
  while (existing.has(`scene_${suffix}`)) suffix += 1
  const node = createSceneRoleplayNodeDraft(
    `scene_${suffix}`,
    scenes.value[0]?.id || '',
    characters.value[0]?.id || '',
    endings.value[0]?.id || draft.value.exhaustion_ending_id,
  )
  node.score_rules = draft.value.score_dimensions.slice(0, 1).map(dimension => ({
    dimension_id: dimension.id,
    guidance: 'Evaluate concrete progress toward the active scene goal.',
    max_delta_per_turn: 1,
  }))
  draft.value.nodes.push(node)
  selectedNodeId.value = node.id
  tab.value = 'nodes'
}

function removeSelectedNode() {
  if (!draft.value || !selectedNode.value || draft.value.nodes.length === 1) return
  const id = selectedNode.value.id
  draft.value.nodes = draft.value.nodes.filter(node => node !== selectedNode.value)
  for (const node of draft.value.nodes) {
    node.transitions = node.transitions.filter(route => route.target.kind !== 'node' || route.target.node_id !== id)
    if (node.timeout_target.kind === 'node' && node.timeout_target.node_id === id) {
      node.timeout_target = createRoleplayTarget('ending', draft.value.exhaustion_ending_id)
    }
  }
  if (draft.value.start_node_id === id) draft.value.start_node_id = draft.value.nodes[0].id
  selectedNodeId.value = draft.value.nodes[0].id
}

function synchronizeNodeId(event: Event) {
  if (!draft.value || !selectedNode.value) return
  const previous = selectedNodeId.value
  const next = inputValue(event).trim()
  if (!next || previous === next) return
  for (const node of draft.value.nodes) {
    for (const route of node.transitions) if (route.target.kind === 'node' && route.target.node_id === previous) route.target.node_id = next
    if (node.timeout_target.kind === 'node' && node.timeout_target.node_id === previous) node.timeout_target.node_id = next
  }
  if (draft.value.start_node_id === previous) draft.value.start_node_id = next
  selectedNodeId.value = next
}

function addScoreRule() {
  if (!draft.value || !selectedNode.value) return
  const dimension = draft.value.score_dimensions.find(item => !selectedNode.value!.score_rules.some(rule => rule.dimension_id === item.id))
    || draft.value.score_dimensions[0]
  if (dimension) selectedNode.value.score_rules.push({ dimension_id: dimension.id, guidance: 'Describe the bounded evaluation rule.', max_delta_per_turn: 1 })
}

function addEvidenceRule() {
  if (!selectedNode.value) return
  const existing = new Set(selectedNode.value.evidence_rules.map(rule => rule.id))
  let suffix = selectedNode.value.evidence_rules.length + 1
  while (existing.has(`evidence_${suffix}`)) suffix += 1
  selectedNode.value.evidence_rules.push({ id: `evidence_${suffix}`, description: 'Describe the exact player statement that proves this evidence.' })
}

function addTransition() {
  if (!draft.value || !selectedNode.value) return
  selectedNode.value.transitions.push({
    id: `route_${selectedNode.value.transitions.length + 1}`,
    priority: selectedNode.value.transitions.length + 1,
    target: createRoleplayTarget('ending', endings.value[0]?.id || draft.value.exhaustion_ending_id),
    conditions: [],
  })
}

function setTransitionTargetKind(index: number, kind: 'node' | 'ending') {
  if (!draft.value || !selectedNode.value) return
  selectedNode.value.transitions[index].target = createRoleplayTarget(
    kind,
    kind === 'node' ? draft.value.nodes[0]?.id || '' : endings.value[0]?.id || draft.value.exhaustion_ending_id,
  )
}

function setTimeoutTargetKind(kind: 'node' | 'ending') {
  if (!draft.value || !selectedNode.value) return
  selectedNode.value.timeout_target = createRoleplayTarget(
    kind,
    kind === 'node' ? draft.value.nodes[0]?.id || '' : endings.value[0]?.id || draft.value.exhaustion_ending_id,
  )
}

function addCondition(transitionIndex: number) {
  if (!draft.value || !selectedNode.value) return
  selectedNode.value.transitions[transitionIndex].conditions.push(createRoleplayCondition('node_turns_at_least', draft.value, selectedNode.value))
}

function replaceCondition(transitionIndex: number, conditionIndex: number, kind: RoleplayCondition['kind']) {
  if (!draft.value || !selectedNode.value) return
  selectedNode.value.transitions[transitionIndex].conditions[conditionIndex] = createRoleplayCondition(kind, draft.value, selectedNode.value)
}

function toggleSupportingCharacter(characterId: string) {
  if (!selectedNode.value || characterId === selectedNode.value.character_id) return
  const selected = selectedNode.value.supporting_character_ids
  selectedNode.value.supporting_character_ids = selected.includes(characterId)
    ? selected.filter(id => id !== characterId)
    : [...selected, characterId]
  selectedNode.value.participant_goals ||= {}
  if (!selectedNode.value.supporting_character_ids.includes(characterId)) delete selectedNode.value.participant_goals[characterId]
}

function nodeHasSupportingCharacter(characterId: string): boolean {
  return selectedNode.value?.supporting_character_ids.includes(characterId) || false
}

function setParticipantGoal(characterId: string, goal: string) {
  if (!selectedNode.value) return
  selectedNode.value.participant_goals ||= {}
  selectedNode.value.participant_goals[characterId] = goal
}

function toggleKnowledge(id: string) {
  if (!selectedNode.value) return
  selectedNode.value.knowledge_refs = selectedNode.value.knowledge_refs.includes(id)
    ? selectedNode.value.knowledge_refs.filter(item => item !== id)
    : [...selectedNode.value.knowledge_refs, id]
}

function toggleRelationshipRule() {
  if (!selectedNode.value) return
  selectedNode.value.relationship_rule = selectedNode.value.relationship_rule
    ? null
    : { guidance: 'Reward trust-building conduct and penalize boundary violations.', max_delta_per_turn: 0.1 }
}

function toggleIntrusionResponse() {
  if (!selectedNode.value) return
  selectedNode.value.intrusion_response = selectedNode.value.intrusion_response ? null : {
    reality_anchors: ['The visible scene remains unchanged.'],
    interpretations: ['That sounded as if you were addressing something outside this place.'],
    redirects: ['Look at what is here and tell me what you actually observe.'],
  }
}

function toggleResponseGuard() {
  if (!selectedNode.value) return
  selectedNode.value.response_guard = selectedNode.value.response_guard ? null : {
    forbidden_markers: ['system prompt', 'language model', 'score'],
    grounding_markers: ['scene'],
    min_grounding_matches: 1,
    max_characters: 320,
    max_sentences: 3,
    recoveries: ['The scene is still here. Tell me what you can actually see.'],
  }
}

function toggleFallbackEvaluation() {
  if (!selectedNode.value) return
  selectedNode.value.fallback_evaluation = selectedNode.value.fallback_evaluation ? null : {
    score_signals: [],
    evidence_signals: [],
  }
}

function addFallbackScore() {
  if (!selectedNode.value?.fallback_evaluation) return
  const rule = selectedNode.value.score_rules[0]
  if (rule) selectedNode.value.fallback_evaluation.score_signals.push({
    dimension_id: rule.dimension_id, positive_markers: ['verified'], negative_markers: ['assume'], delta: Math.min(1, rule.max_delta_per_turn),
  })
}

function addFallbackEvidence() {
  if (!selectedNode.value?.fallback_evaluation) return
  const rule = selectedNode.value.evidence_rules[0]
  if (rule) selectedNode.value.fallback_evaluation.evidence_signals.push({ evidence_id: rule.id, markers: ['verified'] })
}

function setIntrusionList(key: keyof RoleplayIntrusionResponse, value: string) {
  if (selectedNode.value?.intrusion_response) selectedNode.value.intrusion_response[key] = lineList(value)
}

function setGuardList(key: 'forbidden_markers' | 'grounding_markers' | 'recoveries', value: string) {
  const guard = selectedNode.value?.response_guard
  if (guard) guard[key] = lineList(value)
}

function hydrateNewDefinitionReferences(definition: SceneRoleplayDefinition) {
  definition.exhaustion_ending_id = endings.value[0]?.id || ''
  const node = definition.nodes[0]
  node.scene_id = scenes.value[0]?.id || ''
  node.character_id = characters.value[0]?.id || ''
  node.timeout_target = createRoleplayTarget('ending', definition.exhaustion_ending_id)
}

function referenceIssues(definition: SceneRoleplayDefinition): string[] {
  const issues: string[] = []
  const sceneIds = new Set(scenes.value.map(item => item.id))
  const characterIds = new Set(characters.value.map(item => item.id))
  const endingIds = new Set(endings.value.map(item => item.id))
  const knowledgeIds = new Set(knowledge.value.map(item => item.id))
  if (!endingIds.has(definition.exhaustion_ending_id)) issues.push(`Exhaustion ending "${definition.exhaustion_ending_id}" does not exist.`)
  for (const node of definition.nodes) {
    if (!sceneIds.has(node.scene_id)) issues.push(`Node "${node.id}" references missing scene "${node.scene_id}".`)
    for (const id of [node.character_id, ...node.supporting_character_ids]) {
      if (!characterIds.has(id)) issues.push(`Node "${node.id}" references missing character "${id}".`)
    }
    for (const id of node.knowledge_refs) if (!knowledgeIds.has(id)) issues.push(`Node "${node.id}" references missing knowledge "${id}".`)
    for (const target of [node.timeout_target, ...node.transitions.map(route => route.target)]) {
      if (target.kind === 'ending' && !endingIds.has(target.ending_id)) issues.push(`Node "${node.id}" references missing ending "${target.ending_id}".`)
    }
  }
  return [...new Set(issues)]
}

function sceneName(id: string) { return scenes.value.find(item => item.id === id)?.name || id }
function characterName(id: string) { return characters.value.find(item => item.id === id)?.name || id }
function inputValue(event: Event) { return (event.target as HTMLInputElement | HTMLTextAreaElement).value }
function selectValue(event: Event) { return (event.target as HTMLSelectElement).value }
function lineList(value: string) { return value.split(/\r?\n/).map(item => item.trim()).filter(Boolean) }
function commaList(value: string) { return value.split(',').map(item => item.trim()).filter(Boolean) }
function showNotice(type: 'success' | 'error', title: string, message: string) { notice.value = { type, title, message } }
function confirmDiscard() { return !dirty.value || window.confirm('Discard unsaved roleplay changes?') }

function handleBeforeUnload(event: BeforeUnloadEvent) {
  if (!dirty.value) return
  event.preventDefault()
  event.returnValue = ''
}
function handleKeydown(event: KeyboardEvent) {
  if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 's') {
    event.preventDefault()
    void saveRoleplay()
  }
}

onBeforeRouteLeave(() => confirmDiscard())
onMounted(async () => {
  window.addEventListener('beforeunload', handleBeforeUnload)
  window.addEventListener('keydown', handleKeydown)
  await loadCatalog()
})
onUnmounted(() => {
  window.removeEventListener('beforeunload', handleBeforeUnload)
  window.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.roleplay-editor { height: calc(100svh - 56px); min-height: 0; display: flex; flex-direction: column; overflow: hidden; background: var(--surface-0); color: var(--text-primary); }
.editor-header { display: flex; flex: 0 0 auto; align-items: flex-start; justify-content: space-between; gap: 20px; padding: 16px 20px 12px; border-bottom: 1px solid var(--border); }
.editor-header h1 { margin: 2px 0 3px; font-size: 22px; letter-spacing: 0; }
.editor-header p { margin: 0; color: var(--text-secondary); font-size: 12px; }
.eyebrow { color: var(--text-tertiary); font-size: 9px; font-weight: 700; letter-spacing: 0; text-transform: uppercase; }
.header-actions { display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 6px; }
.metrics-strip { display: flex; flex: 0 0 auto; flex-wrap: wrap; gap: 18px; padding: 7px 20px; border-bottom: 1px solid var(--border); background: var(--surface-1); color: var(--text-tertiary); font-size: 10px; }
.metrics-strip strong { color: var(--text-primary); font-family: var(--font-mono); }
.dirty-indicator { color: var(--warning); }
.mobile-roleplay-picker { display: none; }
.notice { display: grid; grid-template-columns: auto minmax(0, 1fr) auto; align-items: center; gap: 10px; padding: 8px 18px; border-bottom: 1px solid var(--border); font-size: 11px; }
.notice.success { background: color-mix(in srgb, var(--success) 8%, var(--surface-1)); }.notice.error { background: color-mix(in srgb, var(--danger) 8%, var(--surface-1)); }
.editor-grid { display: grid; min-height: 0; flex: 1; grid-template-columns: 230px minmax(0, 1fr) 220px; }
.catalog-pane, .inspector-pane { min-height: 0; overflow: auto; background: var(--surface-1); }
.catalog-pane { border-right: 1px solid var(--border); }.inspector-pane { border-left: 1px solid var(--border); padding: 12px; }
.search-field { display: flex; align-items: center; gap: 7px; margin: 10px; padding: 7px 9px; border: 1px solid var(--border); border-radius: 5px; background: var(--surface-0); }
.search-field input { min-width: 0; flex: 1; border: 0; outline: 0; background: transparent; color: inherit; font-size: 11px; }
.catalog-list { display: grid; gap: 2px; padding: 0 7px 12px; }
.catalog-row { display: grid; gap: 3px; padding: 9px 10px; border: 1px solid transparent; border-radius: 5px; background: transparent; color: inherit; text-align: left; cursor: pointer; }
.catalog-row:hover { background: var(--surface-2); }.catalog-row.active { border-color: var(--accent); background: color-mix(in srgb, var(--accent) 8%, var(--surface-1)); }
.catalog-row strong { overflow: hidden; font-size: 11px; text-overflow: ellipsis; white-space: nowrap; }.catalog-row span, .catalog-row small { overflow: hidden; color: var(--text-tertiary); font: 9px var(--font-mono); text-overflow: ellipsis; white-space: nowrap; }
.empty-list, .empty-state { padding: 18px; color: var(--text-tertiary); font-size: 11px; text-align: center; }
.work-pane { display: flex; min-width: 0; min-height: 0; flex-direction: column; }
.editor-tabs { display: flex; flex: 0 0 auto; gap: 2px; padding: 7px 12px; border-bottom: 1px solid var(--border); background: var(--surface-1); }
.editor-tabs button { display: inline-flex; align-items: center; gap: 6px; padding: 7px 10px; border: 0; border-radius: 5px; background: transparent; color: var(--text-secondary); font-size: 10px; cursor: pointer; }
.editor-tabs button.active { background: var(--surface-3); color: var(--text-primary); }
.work-scroll { min-height: 0; overflow: auto; padding: 14px; }
.issues-panel { display: flex; gap: 10px; margin-bottom: 12px; padding: 10px; border: 1px solid color-mix(in srgb, var(--danger) 35%, var(--border)); border-radius: 6px; background: color-mix(in srgb, var(--danger) 6%, var(--surface-1)); color: var(--danger); font-size: 10px; }
.issues-panel p { margin: 3px 0 0; color: var(--text-secondary); }
.editor-section { display: grid; gap: 14px; max-width: 1180px; margin: 0 auto 14px; }
.section-heading { display: flex; align-items: center; justify-content: space-between; gap: 12px; }
.section-heading h2 { margin: 2px 0 0; font-size: 15px; letter-spacing: 0; }.section-heading code { max-width: 50%; overflow: hidden; color: var(--text-tertiary); font-size: 9px; text-overflow: ellipsis; white-space: nowrap; }
.form-grid { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 10px; }.form-grid.four { grid-template-columns: repeat(4, minmax(0, 1fr)); }.form-field.wide, .form-field.full { grid-column: 1 / -1; }
.form-field { display: grid; min-width: 0; gap: 5px; }.form-field span { color: var(--text-secondary); font-size: 9px; font-weight: 600; }
.form-field input, .form-field select, .form-field textarea, .compact-row input, .compact-row select, .route-head input, .route-head select, .condition-row input, .condition-row select, .fallback-row input, .fallback-row select, .timeout-row select, .participant-goal input { width: 100%; min-width: 0; padding: 7px 8px; border: 1px solid var(--border); border-radius: 5px; outline: 0; background: var(--surface-1); color: var(--text-primary); font-size: 10px; }
.form-field textarea { resize: vertical; line-height: 1.5; }.numeric { max-width: 180px; }
.pipeline { display: flex; align-items: center; justify-content: center; gap: 8px; padding: 10px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-1); color: var(--text-secondary); font-size: 9px; }
.pipeline span { padding: 5px 7px; border-radius: 4px; background: var(--surface-2); }
.repeat-list { display: grid; gap: 7px; }.repeat-row { display: grid; align-items: end; gap: 7px; padding: 9px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-1); }
.dimension-row { grid-template-columns: 130px 140px minmax(180px, 1fr) 70px 70px 70px 30px; }.dimension-row .description { grid-column: auto; }.dimension-row .numeric { max-width: none; }
.node-layout { display: grid; min-height: 540px; grid-template-columns: 160px minmax(0, 1fr); gap: 12px; }
.node-list { align-self: start; overflow: hidden; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-1); }.node-list-head { display: flex; align-items: center; justify-content: space-between; padding: 8px; border-bottom: 1px solid var(--border); font-size: 10px; }
.node-list > button { display: grid; width: 100%; gap: 2px; padding: 8px; border: 0; border-bottom: 1px solid var(--border); background: transparent; color: inherit; text-align: left; cursor: pointer; }.node-list > button.active { background: color-mix(in srgb, var(--accent) 9%, var(--surface-1)); }.node-list strong { overflow: hidden; font-size: 10px; text-overflow: ellipsis; }.node-list span { overflow: hidden; color: var(--text-tertiary); font-size: 8px; text-overflow: ellipsis; white-space: nowrap; }
.node-work { min-width: 0; }.subsection { display: grid; gap: 8px; padding-top: 12px; border-top: 1px solid var(--border); }.subsection-head { display: flex; align-items: center; justify-content: space-between; gap: 10px; font-size: 10px; }.subsection-head small { color: var(--text-tertiary); font-size: 8px; font-weight: 400; }
.check-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 5px; }.check-row { display: flex; min-width: 0; align-items: center; gap: 6px; padding: 6px; border: 1px solid var(--border); border-radius: 4px; background: var(--surface-1); font-size: 9px; }.check-row span { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.participant-goal { display: grid; grid-template-columns: 120px minmax(0, 1fr); align-items: center; gap: 8px; font-size: 9px; }
.compact-row { display: grid; grid-template-columns: 150px minmax(0, 1fr) 80px 30px; gap: 6px; }.compact-row.evidence-row { grid-template-columns: 180px minmax(0, 1fr) 30px; }.relationship-row { grid-template-columns: minmax(0, 1fr) 90px; }
.short-input { max-width: 90px; }.toggle-row { display: flex; align-items: center; gap: 9px; padding: 9px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-1); font-size: 10px; }.toggle-row span { display: grid; gap: 2px; }.toggle-row small { color: var(--text-tertiary); font-size: 8px; }
.route-block { display: grid; gap: 8px; padding: 9px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-1); }.route-head { display: grid; grid-template-columns: minmax(120px, 1fr) 110px 90px 160px 30px; gap: 6px; }.route-head label { display: flex; align-items: center; gap: 5px; color: var(--text-tertiary); font-size: 8px; }.route-head label input { width: 60px; }
.condition-list { display: grid; gap: 5px; padding-left: 12px; border-left: 2px solid var(--border); }.condition-row { display: grid; grid-template-columns: 180px minmax(130px, 1fr) 90px 30px; gap: 6px; }
.timeout-row { display: grid; grid-template-columns: minmax(0, 1fr) 100px 190px; align-items: center; gap: 8px; padding: 10px; border: 1px solid var(--border); border-radius: 6px; background: var(--surface-1); }.timeout-row div { display: grid; gap: 2px; font-size: 10px; }.timeout-row small { color: var(--text-tertiary); font-size: 8px; }
.safety-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 8px; }.fallback-row { display: grid; grid-template-columns: 160px minmax(140px, 1fr) minmax(140px, 1fr) 70px 30px; gap: 6px; }.fallback-row.evidence { grid-template-columns: 180px minmax(0, 1fr) 30px; }
.inspector-pane section { display: grid; gap: 7px; padding: 10px 0; border-bottom: 1px solid var(--border); }.metric-row { display: flex; justify-content: space-between; gap: 8px; color: var(--text-secondary); font-size: 9px; }.metric-row strong { color: var(--text-primary); font-family: var(--font-mono); }.inspector-pane p, .inspector-pane small { margin: 0; color: var(--text-secondary); font-size: 9px; line-height: 1.5; }.delete-roleplay { width: 100%; margin-top: 12px; }
.btn-xs { min-height: 26px; padding: 4px 7px; font-size: 9px; }.icon-button.danger { color: var(--danger); }
@media (max-width: 1100px) { .editor-grid { grid-template-columns: 200px minmax(0, 1fr); }.inspector-pane { display: none; }.form-grid.four { grid-template-columns: repeat(2, minmax(0, 1fr)); }.dimension-row { grid-template-columns: repeat(3, minmax(0, 1fr)); }.safety-grid { grid-template-columns: 1fr; } }
@media (max-width: 860px) { .roleplay-editor { height: calc(100svh - 114px - env(safe-area-inset-bottom, 0px)); } }
@media (max-width: 760px) { .editor-header { align-items: stretch; flex-direction: column; }.header-actions { justify-content: flex-start; }.mobile-roleplay-picker { display: grid; flex: 0 0 auto; grid-template-columns: auto minmax(0, 1fr); align-items: center; gap: 8px; padding: 7px 12px; border-bottom: 1px solid var(--border); background: var(--surface-1); color: var(--text-secondary); font-size: 9px; }.mobile-roleplay-picker select { min-width: 0; width: 100%; padding: 6px 8px; border: 1px solid var(--border); border-radius: 5px; background: var(--surface-0); color: var(--text-primary); font-size: 10px; }.editor-grid { grid-template-columns: 1fr; }.catalog-pane { display: none; }.work-scroll { padding: 8px; }.node-layout { grid-template-columns: 1fr; }.node-list { display: flex; overflow-x: auto; }.node-list-head { min-width: 90px; }.node-list > button { min-width: 120px; border-right: 1px solid var(--border); border-bottom: 0; }.form-grid, .form-grid.four, .check-grid { grid-template-columns: 1fr; }.pipeline { align-items: stretch; flex-direction: column; }.pipeline svg { align-self: center; transform: rotate(90deg); }.route-head, .condition-row, .fallback-row { grid-template-columns: 1fr; }.timeout-row { grid-template-columns: 1fr; } }
</style>
