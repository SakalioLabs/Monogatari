import {
  connectWorkflowNodes,
  workflowNodeAtPoint,
  WORKFLOW_NODE_HEIGHT,
  WORKFLOW_NODE_WIDTH,
  type WorkflowPoint,
} from './workflowAuthoring'
import type { WorkflowNode } from './workflowContract'

export interface WorkflowCanvasBounds {
  left: number
  top: number
  width: number
  height: number
}

export interface WorkflowPointerPosition {
  clientX: number
  clientY: number
}

export interface WorkflowCanvasEventTarget {
  addEventListener(type: 'mousemove' | 'mouseup', listener: (event: MouseEvent) => void): void
  removeEventListener(type: 'mousemove' | 'mouseup', listener: (event: MouseEvent) => void): void
}

export interface WorkflowCanvasInteractionOptions {
  eventTarget: WorkflowCanvasEventTarget
  getCanvasBounds: () => WorkflowCanvasBounds | null
  getNodes: () => readonly WorkflowNode[]
  commitNodes: (nodes: WorkflowNode[], changedNodeId: string) => void
}

export interface WorkflowCanvasInteractionController {
  startNodeDrag: (event: WorkflowPointerPosition, nodeId: string) => void
  startConnection: (
    event: WorkflowPointerPosition & Pick<MouseEvent, 'preventDefault'>,
    sourceNodeId: string,
  ) => void
  cancel: () => void
  dispose: () => void
}

export function workflowCanvasPoint(
  pointer: WorkflowPointerPosition,
  bounds: WorkflowCanvasBounds,
): WorkflowPoint {
  return {
    x: pointer.clientX - bounds.left,
    y: pointer.clientY - bounds.top,
  }
}

export function workflowDraggedNodePosition(
  startNode: Pick<WorkflowNode, 'x' | 'y'>,
  startPointer: WorkflowPointerPosition,
  currentPointer: WorkflowPointerPosition,
  bounds: Pick<WorkflowCanvasBounds, 'width' | 'height'>,
): WorkflowPoint {
  return {
    x: clamp(
      startNode.x + currentPointer.clientX - startPointer.clientX,
      0,
      Math.max(0, bounds.width - WORKFLOW_NODE_WIDTH),
    ),
    y: clamp(
      startNode.y + currentPointer.clientY - startPointer.clientY,
      0,
      Math.max(0, bounds.height - WORKFLOW_NODE_HEIGHT),
    ),
  }
}

export function createWorkflowCanvasInteractionController(
  options: WorkflowCanvasInteractionOptions,
): WorkflowCanvasInteractionController {
  let removeActiveListeners: (() => void) | null = null

  function cancel() {
    const remove = removeActiveListeners
    removeActiveListeners = null
    remove?.()
  }

  function startNodeDrag(event: WorkflowPointerPosition, nodeId: string) {
    cancel()
    const startNode = options.getNodes().find((node) => node.id === nodeId)
    if (!startNode || !options.getCanvasBounds()) return

    const startPointer = { clientX: event.clientX, clientY: event.clientY }
    const startPosition = { x: startNode.x, y: startNode.y }
    let lastPosition = startPosition

    const onMouseMove = (moveEvent: MouseEvent) => {
      const bounds = options.getCanvasBounds()
      const currentNodes = options.getNodes()
      if (!bounds || !currentNodes.some((node) => node.id === nodeId)) {
        cancel()
        return
      }
      const position = workflowDraggedNodePosition(
        startPosition,
        startPointer,
        moveEvent,
        bounds,
      )
      if (position.x === lastPosition.x && position.y === lastPosition.y) return
      lastPosition = position
      options.commitNodes(
        currentNodes.map((node) => node.id === nodeId ? { ...node, ...position } : node),
        nodeId,
      )
    }
    const onMouseUp = () => cancel()

    options.eventTarget.addEventListener('mousemove', onMouseMove)
    options.eventTarget.addEventListener('mouseup', onMouseUp)
    removeActiveListeners = () => {
      options.eventTarget.removeEventListener('mousemove', onMouseMove)
      options.eventTarget.removeEventListener('mouseup', onMouseUp)
    }
  }

  function startConnection(
    event: WorkflowPointerPosition & Pick<MouseEvent, 'preventDefault'>,
    sourceNodeId: string,
  ) {
    event.preventDefault()
    cancel()
    if (!options.getNodes().some((node) => node.id === sourceNodeId)) return

    const onMouseUp = (upEvent: MouseEvent) => {
      const bounds = options.getCanvasBounds()
      const currentNodes = options.getNodes()
      if (bounds) {
        const target = workflowNodeAtPoint(currentNodes, workflowCanvasPoint(upEvent, bounds))
        if (target) {
          const update = connectWorkflowNodes(currentNodes, sourceNodeId, target.id)
          if (update.changed) options.commitNodes(update.nodes, sourceNodeId)
        }
      }
      cancel()
    }

    options.eventTarget.addEventListener('mouseup', onMouseUp)
    removeActiveListeners = () => {
      options.eventTarget.removeEventListener('mouseup', onMouseUp)
    }
  }

  return {
    startNodeDrag,
    startConnection,
    cancel,
    dispose: cancel,
  }
}

function clamp(value: number, minimum: number, maximum: number): number {
  return Math.min(maximum, Math.max(minimum, value))
}
