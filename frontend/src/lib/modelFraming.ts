export type BoundsPoint3 = readonly [number, number, number]
export type ModelPresentation = 'character' | 'scene'

export interface ModelBounds3 {
  min: BoundsPoint3
  max: BoundsPoint3
}

export interface PrimaryModelBoundsSelection {
  bounds: ModelBounds3
  includedIndices: number[]
  excludedIndices: number[]
}

interface BoundsCandidate {
  bounds: ModelBounds3
  diagonal: number
  index: number
}

const DEFAULT_CONNECTIVITY_RATIO = 0.75
const MIN_DIAGONAL = 1e-6

export function calculateModelCameraDistance(
  size: BoundsPoint3,
  verticalFovRadians: number,
  aspect: number,
  presentation: ModelPresentation,
): number {
  const safeVerticalFov = Math.max(verticalFovRadians, 0.01)
  const safeAspect = Math.max(aspect, 0.1)
  const horizontalFov = 2 * Math.atan(Math.tan(safeVerticalFov / 2) * safeAspect)
  const heightDistance = size[1] / Math.max(2 * Math.tan(safeVerticalFov / 2), 0.01)
  const widthDistance = size[0] / Math.max(2 * Math.tan(horizontalFov / 2), 0.01)
  const depthDistance = size[2] * 0.7
  const planarDistance = presentation === 'scene'
    ? Math.min(heightDistance, widthDistance)
    : Math.max(heightDistance, widthDistance)
  const distanceScale = presentation === 'scene' ? 1.25 : 1.55
  return Math.max(planarDistance, depthDistance, 1) * distanceScale
}

export function selectPrimaryModelBounds(
  candidates: readonly ModelBounds3[],
  connectivityRatio = DEFAULT_CONNECTIVITY_RATIO,
): PrimaryModelBoundsSelection | null {
  const ratio = Number.isFinite(connectivityRatio) && connectivityRatio > 0
    ? connectivityRatio
    : DEFAULT_CONNECTIVITY_RATIO
  const validCandidates = candidates
    .map((bounds, index): BoundsCandidate | null => {
      if (!isFiniteBounds(bounds)) return null
      const diagonal = boundsDiagonal(bounds)
      return diagonal > MIN_DIAGONAL ? { bounds, diagonal, index } : null
    })
    .filter((candidate): candidate is BoundsCandidate => candidate !== null)
    .sort((left, right) => right.diagonal - left.diagonal || left.index - right.index)

  if (validCandidates.length === 0) return null

  const primaryBounds = cloneBounds(validCandidates[0].bounds)
  const included = [validCandidates[0]]
  let pending = validCandidates.slice(1)
  let expanded = true

  while (expanded) {
    expanded = false
    const nextPending: BoundsCandidate[] = []
    for (const candidate of pending) {
      const connectionDistance = Math.max(boundsDiagonal(primaryBounds), candidate.diagonal) * ratio
      if (boundsGap(primaryBounds, candidate.bounds) <= connectionDistance) {
        unionBounds(primaryBounds, candidate.bounds)
        included.push(candidate)
        expanded = true
      } else {
        nextPending.push(candidate)
      }
    }
    pending = nextPending
  }

  return {
    bounds: primaryBounds,
    includedIndices: included.map((candidate) => candidate.index).sort((left, right) => left - right),
    excludedIndices: pending.map((candidate) => candidate.index).sort((left, right) => left - right),
  }
}

function isFiniteBounds(bounds: ModelBounds3): boolean {
  return bounds.min.length === 3
    && bounds.max.length === 3
    && [...bounds.min, ...bounds.max].every(Number.isFinite)
    && bounds.min.every((value, index) => value <= bounds.max[index])
}

function cloneBounds(bounds: ModelBounds3): ModelBounds3 {
  return {
    min: [...bounds.min] as [number, number, number],
    max: [...bounds.max] as [number, number, number],
  }
}

function boundsDiagonal(bounds: ModelBounds3): number {
  return Math.hypot(
    bounds.max[0] - bounds.min[0],
    bounds.max[1] - bounds.min[1],
    bounds.max[2] - bounds.min[2],
  )
}

function boundsGap(left: ModelBounds3, right: ModelBounds3): number {
  return Math.hypot(
    axisGap(left.min[0], left.max[0], right.min[0], right.max[0]),
    axisGap(left.min[1], left.max[1], right.min[1], right.max[1]),
    axisGap(left.min[2], left.max[2], right.min[2], right.max[2]),
  )
}

function axisGap(leftMin: number, leftMax: number, rightMin: number, rightMax: number): number {
  return Math.max(leftMin - rightMax, rightMin - leftMax, 0)
}

function unionBounds(target: ModelBounds3, source: ModelBounds3) {
  const targetMin = target.min as [number, number, number]
  const targetMax = target.max as [number, number, number]
  for (let axis = 0; axis < 3; axis += 1) {
    targetMin[axis] = Math.min(targetMin[axis], source.min[axis])
    targetMax[axis] = Math.max(targetMax[axis], source.max[axis])
  }
}
