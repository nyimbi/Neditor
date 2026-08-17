/** Mock for @tauri-apps/api/window */

class MockWebviewWindow {
  label = "main";
  async setTitle(_title: string): Promise<void> {}
  async show(): Promise<void> {}
  async hide(): Promise<void> {}
  async close(): Promise<void> {}
  async maximize(): Promise<void> {}
  async unmaximize(): Promise<void> {}
  async isMaximized(): Promise<boolean> { return false; }
  async minimize(): Promise<void> {}
  async unminimize(): Promise<void> {}
  async setFullscreen(_fullscreen: boolean): Promise<void> {}
  async isFullscreen(): Promise<boolean> { return false; }
  async isVisible(): Promise<boolean> { return true; }
  async center(): Promise<void> {}
  async requestUserAttention(): Promise<void> {}
  async setFocus(): Promise<void> {}
  async setDecorations(_decorations: boolean): Promise<void> {}
  async setAlwaysOnTop(_alwaysOnTop: boolean): Promise<void> {}
  async setResizable(_resizable: boolean): Promise<void> {}
  async setSize(_size: unknown): Promise<void> {}
  async setMinSize(_size: unknown): Promise<void> {}
  async setMaxSize(_size: unknown): Promise<void> {}
  async setPosition(_position: unknown): Promise<void> {}
  async scaleFactor(): Promise<number> { return 1; }
  async innerPosition(): Promise<{ x: number; y: number }> { return { x: 0, y: 0 }; }
  async outerPosition(): Promise<{ x: number; y: number }> { return { x: 0, y: 0 }; }
  async innerSize(): Promise<{ width: number; height: number }> { return { width: 1280, height: 800 }; }
  async outerSize(): Promise<{ width: number; height: number }> { return { width: 1280, height: 800 }; }
  async theme(): Promise<string> { return "light"; }
  async onFileDropEvent(_handler: (event: unknown) => void): Promise<() => void> { return () => {}; }
  async onResized(_handler: (event: unknown) => void): Promise<() => void> { return () => {}; }
  async onMoved(_handler: (event: unknown) => void): Promise<() => void> { return () => {}; }
  async onCloseRequested(_handler: (event: unknown) => void): Promise<() => void> { return () => {}; }
  async onFocusChanged(_handler: (event: unknown) => void): Promise<() => void> { return () => {}; }
  async onScaleChanged(_handler: (event: unknown) => void): Promise<() => void> { return () => {}; }
  async onMenuClicked(_handler: (event: unknown) => void): Promise<() => void> { return () => {}; }
  async listen(_event: string, _handler: (event: unknown) => void): Promise<() => void> { return () => {}; }
  async once(_event: string, _handler: (event: unknown) => void): Promise<() => void> { return () => {}; }
  async emit(_event: string, _payload?: unknown): Promise<void> {}
  async emitTo(_target: string, _event: string, _payload?: unknown): Promise<void> {}
  destroy(): Promise<void> { return Promise.resolve(); }
}

const _win = new MockWebviewWindow();
export function getCurrentWindow(): MockWebviewWindow { return _win; }
export function getAll(): MockWebviewWindow[] { return [_win]; }
export function getByLabel(_label: string): MockWebviewWindow | null { return _win; }
export { MockWebviewWindow as WebviewWindow };
