export type SceneDialoguePosition = 'bottom' | 'top' | 'left' | 'right'
export type SceneCharacterAnchor = 'left' | 'center' | 'right'
export type ScenePresentationPreset = 'classic_bottom' | 'split_left' | 'split_right' | 'focus'

export interface ScenePresentation {
  dialogue_position: SceneDialoguePosition
  dialogue_width_percent: number
  dialogue_height_percent: number
  dialogue_inset_percent: number
  character_anchor: SceneCharacterAnchor
  character_width_percent: number
  character_height_percent: number
  character_offset_x_percent: number
  character_offset_y_percent: number
  background_dim_percent: number
}

export interface ScenePresentationStyles {
  stage: Record<string, string>
  character: Record<string, string>
  dialogue: Record<string, string>
}

export const DEFAULT_SCENE_PRESENTATION: Readonly<ScenePresentation> = Object.freeze({
  dialogue_position: 'bottom',
  dialogue_width_percent: 92,
  dialogue_height_percent: 38,
  dialogue_inset_percent: 4,
  character_anchor: 'center',
  character_width_percent: 46,
  character_height_percent: 76,
  character_offset_x_percent: 0,
  character_offset_y_percent: 0,
  background_dim_percent: 24,
})

const PRESETS: Record<ScenePresentationPreset, ScenePresentation> = {
  classic_bottom: { ...DEFAULT_SCENE_PRESENTATION },
  split_left: {
    ...DEFAULT_SCENE_PRESENTATION,
    dialogue_position: 'right',
    dialogue_width_percent: 44,
    dialogue_height_percent: 80,
    dialogue_inset_percent: 3,
    character_anchor: 'left',
    character_width_percent: 50,
    character_height_percent: 82,
    background_dim_percent: 18,
  },
  split_right: {
    ...DEFAULT_SCENE_PRESENTATION,
    dialogue_position: 'left',
    dialogue_width_percent: 44,
    dialogue_height_percent: 80,
    dialogue_inset_percent: 3,
    character_anchor: 'right',
    character_width_percent: 50,
    character_height_percent: 82,
    background_dim_percent: 18,
  },
  focus: {
    ...DEFAULT_SCENE_PRESENTATION,
    dialogue_width_percent: 72,
    dialogue_height_percent: 26,
    dialogue_inset_percent: 3,
    character_width_percent: 58,
    character_height_percent: 88,
    background_dim_percent: 10,
  },
}

export function scenePresentationPreset(preset: ScenePresentationPreset): ScenePresentation {
  return { ...PRESETS[preset] }
}

export function normalizeScenePresentation(
  value: Partial<ScenePresentation> | null | undefined,
): ScenePresentation {
  return {
    dialogue_position: dialoguePosition(value?.dialogue_position),
    dialogue_width_percent: integer(value?.dialogue_width_percent, 30, 96, DEFAULT_SCENE_PRESENTATION.dialogue_width_percent),
    dialogue_height_percent: integer(value?.dialogue_height_percent, 20, 90, DEFAULT_SCENE_PRESENTATION.dialogue_height_percent),
    dialogue_inset_percent: integer(value?.dialogue_inset_percent, 0, 12, DEFAULT_SCENE_PRESENTATION.dialogue_inset_percent),
    character_anchor: characterAnchor(value?.character_anchor),
    character_width_percent: integer(value?.character_width_percent, 20, 80, DEFAULT_SCENE_PRESENTATION.character_width_percent),
    character_height_percent: integer(value?.character_height_percent, 35, 96, DEFAULT_SCENE_PRESENTATION.character_height_percent),
    character_offset_x_percent: integer(value?.character_offset_x_percent, -30, 30, DEFAULT_SCENE_PRESENTATION.character_offset_x_percent),
    character_offset_y_percent: integer(value?.character_offset_y_percent, -10, 30, DEFAULT_SCENE_PRESENTATION.character_offset_y_percent),
    background_dim_percent: integer(value?.background_dim_percent, 0, 80, DEFAULT_SCENE_PRESENTATION.background_dim_percent),
  }
}

export function scenePresentationStyles(
  value: Partial<ScenePresentation> | null | undefined,
): ScenePresentationStyles {
  const layout = normalizeScenePresentation(value)
  const inset = `${layout.dialogue_inset_percent}%`
  const dialogue: Record<string, string> = {
    width: `${layout.dialogue_width_percent}%`,
    height: `${layout.dialogue_height_percent}%`,
    maxWidth: 'none',
    margin: '0',
  }
  if (layout.dialogue_position === 'bottom' || layout.dialogue_position === 'top') {
    dialogue.left = '50%'
    dialogue.transform = 'translateX(-50%)'
    dialogue[layout.dialogue_position] = inset
  } else {
    dialogue.top = '50%'
    dialogue.transform = 'translateY(-50%)'
    dialogue[layout.dialogue_position] = inset
  }

  const anchorPosition = layout.character_anchor === 'left'
    ? 25
    : layout.character_anchor === 'right' ? 75 : 50
  return {
    stage: {
      '--scene-background-dim': `${layout.background_dim_percent / 100}`,
    },
    character: {
      left: `${anchorPosition + layout.character_offset_x_percent}%`,
      bottom: `${layout.character_offset_y_percent}%`,
      width: `${layout.character_width_percent}%`,
      height: `${layout.character_height_percent}%`,
      transform: 'translateX(-50%)',
    },
    dialogue,
  }
}

function dialoguePosition(value: unknown): SceneDialoguePosition {
  return value === 'top' || value === 'left' || value === 'right' ? value : 'bottom'
}

function characterAnchor(value: unknown): SceneCharacterAnchor {
  return value === 'left' || value === 'right' ? value : 'center'
}

function integer(value: unknown, min: number, max: number, fallback: number): number {
  const number = typeof value === 'number' && Number.isFinite(value) ? Math.round(value) : fallback
  return Math.min(max, Math.max(min, number))
}
