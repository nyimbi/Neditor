import { EditorView } from '@codemirror/view'
import { EditorState } from '@codemirror/state'
import type { Extension } from '@codemirror/state'

export function createEditor(el: HTMLElement, opts: { extensions: Extension[]; doc?: string }): EditorView {
  const state = EditorState.create({
    doc: opts.doc ?? '',
    extensions: opts.extensions,
  })
  const view = new EditorView({ state, parent: el })
  return view
}

export interface EditorApi {
  setDoc(text: string): void
  focus(): void
  getSelection(): string
  insertAt(from: number, to: number, text: string): void
  subscribeToChanges(cb: (text: string) => void): () => void
  destroy(): void
}

export function wrapEditorApi(view: EditorView | null): EditorApi {
  const listeners = new Set<(text: string) => void>()

  return {
    setDoc(text: string): void {
      if (!view) return
      const current = view.state.doc.toString()
      if (current === text) return
      view.dispatch({ changes: { from: 0, to: view.state.doc.length, insert: text } })
    },

    focus(): void {
      view?.focus()
    },

    getSelection(): string {
      if (!view) return ''
      const { from, to } = view.state.selection.main
      return view.state.doc.sliceString(from, to)
    },

    insertAt(from: number, to: number, text: string): void {
      if (!view) return
      view.dispatch({ changes: { from, to, insert: text } })
    },

    subscribeToChanges(cb: (text: string) => void): () => void {
      listeners.add(cb)
      return () => { listeners.delete(cb) }
    },

    destroy(): void {
      listeners.clear()
      view?.destroy()
    },
  }
}
