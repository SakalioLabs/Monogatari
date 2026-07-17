import { describe, expect, it, vi } from 'vitest'

import {
  createWorkflowCanvasInteractionController,
  workflowCanvasPoint,
  workflowDraggedNodePosition,
  type WorkflowCanvasEventTarget,
} from '../workflowCanvasInteractions'
import type { WorkflowNode } from '../workflowContract'

function node(id: string, overrides: Partial<WorkflowNode> = {}): WorkflowNode {
  return {
    id,
    node_type: 'dialogue',
    label: id,
    x: 20,
    y: 30,
    config: {},
    connections: [],
    ...overrides,
  }
}

class PointerEventTarget implements WorkflowCanvasEventTarget {
  private readonly listeners = new Map<'mousemove' | 'mouseup', Set<(event: MouseEvent) => void>>()

  addEventListener(type: 'mousemove' | 'mouseup', listener: (event: MouseEvent) => void) {
    const listeners = this.listeners.get(type) ?? new Set()
    listeners.add(listener)
    this.listeners.set(type, listeners)
  }

  removeEventListener(type: 'mousemove' | 'mouseup', listener: (event: MouseEvent) => void) {
    this.listeners.get(type)?.delete(listener)
  }

  dispatch(type: 'mousemove' | 'mouseup', clientX: number, clientY: number) {
    const event = { clientX, clientY } as MouseEvent
    for (const listener of [...(this.listeners.get(type) ?? [])]) listener(event)
  }

  listenerCount(type: 'mousemove' | 'mouseup'): number {
    return this.listeners.get(type)?.size ?? 0
  }
}

function pointer(clientX: number, clientY: number) {
  return { clientX, clientY }
}

describe('workflow canvas interactions', () => {
  it('converts client points and clamps drag positions to the canvas', () => {
    const bounds = { left: 100, top: 50, width: 400, height: 300 }
    expect(workflowCanvasPoint(pointer(145, 95), bounds)).toEqual({ x: 45, y: 45 })
    expect(workflowDraggedNodePosition(
      { x: 20, y: 30 },
      pointer(150, 100),
      pointer(250, 200),
      bounds,
    )).toEqual({ x: 120, y: 130 })
    expect(workflowDraggedNodePosition(
      { x: 20, y: 30 },
      pointer(150, 100),
      pointer(1_000, 1_000),
      bounds,
    )).toEqual({ x: 186, y: 208 })
    expect(workflowDraggedNodePosition(
      { x: 20, y: 30 },
      pointer(150, 100),
      pointer(0, 0),
      bounds,
    )).toEqual({ x: 0, y: 0 })
  })

  it('drags nodes immutably and releases global listeners on mouseup', () => {
    const eventTarget = new PointerEventTarget()
    const original = node('source')
    let nodes = [original]
    const commits: { nodes: WorkflowNode[]; changedNodeId: string }[] = []
    const controller = createWorkflowCanvasInteractionController({
      eventTarget,
      getCanvasBounds: () => ({ left: 100, top: 50, width: 400, height: 300 }),
      getNodes: () => nodes,
      commitNodes(nextNodes, changedNodeId) {
        nodes = nextNodes
        commits.push({ nodes: nextNodes, changedNodeId })
      },
    })

    controller.startNodeDrag(pointer(150, 100), 'source')
    expect(eventTarget.listenerCount('mousemove')).toBe(1)
    expect(eventTarget.listenerCount('mouseup')).toBe(1)
    eventTarget.dispatch('mousemove', 250, 200)

    expect(nodes[0]).toMatchObject({ x: 120, y: 130 })
    expect(nodes[0]).not.toBe(original)
    expect(original).toMatchObject({ x: 20, y: 30 })
    expect(commits).toHaveLength(1)
    expect(commits[0].changedNodeId).toBe('source')

    eventTarget.dispatch('mousemove', 250, 200)
    expect(commits).toHaveLength(1)
    eventTarget.dispatch('mouseup', 250, 200)
    expect(eventTarget.listenerCount('mousemove')).toBe(0)
    expect(eventTarget.listenerCount('mouseup')).toBe(0)
    eventTarget.dispatch('mousemove', 300, 250)
    expect(commits).toHaveLength(1)
  })

  it('connects hit nodes while rejecting self, duplicate, and missing targets', () => {
    const eventTarget = new PointerEventTarget()
    let nodes = [
      node('source', { x: 20, y: 30 }),
      node('target', { x: 300, y: 100 }),
    ]
    const commits: string[] = []
    const controller = createWorkflowCanvasInteractionController({
      eventTarget,
      getCanvasBounds: () => ({ left: 100, top: 50, width: 700, height: 400 }),
      getNodes: () => nodes,
      commitNodes(nextNodes, changedNodeId) {
        nodes = nextNodes
        commits.push(changedNodeId)
      },
    })
    const preventDefault = vi.fn()

    controller.startConnection({ ...pointer(334, 126), preventDefault }, 'source')
    eventTarget.dispatch('mouseup', 410, 160)
    expect(preventDefault).toHaveBeenCalledOnce()
    expect(nodes[0].connections).toEqual(['target'])
    expect(commits).toEqual(['source'])
    expect(eventTarget.listenerCount('mouseup')).toBe(0)

    controller.startConnection({ ...pointer(334, 126), preventDefault }, 'source')
    eventTarget.dispatch('mouseup', 410, 160)
    controller.startConnection({ ...pointer(334, 126), preventDefault }, 'source')
    eventTarget.dispatch('mouseup', 130, 90)
    controller.startConnection({ ...pointer(334, 126), preventDefault }, 'source')
    eventTarget.dispatch('mouseup', 900, 500)
    expect(commits).toEqual(['source'])
  })

  it('cancels prior interactions on reentry, missing state, and disposal', () => {
    const eventTarget = new PointerEventTarget()
    let nodes = [node('source'), node('target', { x: 300, y: 100 })]
    const controller = createWorkflowCanvasInteractionController({
      eventTarget,
      getCanvasBounds: () => ({ left: 0, top: 0, width: 700, height: 400 }),
      getNodes: () => nodes,
      commitNodes(nextNodes) {
        nodes = nextNodes
      },
    })

    controller.startNodeDrag(pointer(20, 30), 'source')
    expect(eventTarget.listenerCount('mousemove')).toBe(1)
    controller.startConnection({ ...pointer(234, 76), preventDefault: vi.fn() }, 'source')
    expect(eventTarget.listenerCount('mousemove')).toBe(0)
    expect(eventTarget.listenerCount('mouseup')).toBe(1)
    controller.dispose()
    expect(eventTarget.listenerCount('mouseup')).toBe(0)

    controller.startNodeDrag(pointer(20, 30), 'missing')
    expect(eventTarget.listenerCount('mousemove')).toBe(0)
    controller.startNodeDrag(pointer(20, 30), 'source')
    nodes = []
    eventTarget.dispatch('mousemove', 40, 50)
    expect(eventTarget.listenerCount('mousemove')).toBe(0)
    expect(eventTarget.listenerCount('mouseup')).toBe(0)
  })
})
