/**
 * Manages the global keydown listener lifecycle and an optional binding
 * registry for future extensibility.
 *
 * The composable owns install/uninstall of the event listener; the actual
 * dispatch function (handleShortcut in App.vue) is passed in at install time
 * because it closes over too many App.vue-local refs to move here cleanly.
 */

export interface BindingSpec {
  id: string;
  /** Human-readable description (for keyboard-shortcuts panel). */
  label: string;
  /** Called by the registry dispatcher when the binding matches. */
  handler: (event: KeyboardEvent) => void;
}

export function useKeybindings() {
  const registry = new Map<string, BindingSpec>();

  let installedTarget: EventTarget | null = null;
  let installedHandler: ((e: KeyboardEvent) => void) | null = null;

  /**
   * Register a named binding.  The id must be unique; re-registering an id
   * replaces the previous spec.
   */
  function registerBinding(spec: BindingSpec): void {
    registry.set(spec.id, spec);
  }

  /** Remove a previously registered binding by id. */
  function unregisterBinding(id: string): void {
    registry.delete(id);
  }

  /**
   * Attach a keydown handler to `target` (defaults to `window`).
   * Call from onMounted; pairs with uninstall().
   */
  function install(
    handler: (e: KeyboardEvent) => void,
    target: EventTarget = window,
  ): void {
    if (installedTarget) uninstall();
    installedTarget = target;
    installedHandler = handler;
    target.addEventListener("keydown", handler);
  }

  /** Remove the previously installed keydown handler. */
  function uninstall(): void {
    if (installedTarget && installedHandler) {
      installedTarget.removeEventListener("keydown", installedHandler);
    }
    installedTarget = null;
    installedHandler = null;
  }

  return {
    registerBinding,
    unregisterBinding,
    install,
    uninstall,
  };
}
