/**
 * Returns true when the in-app menu bar should be hidden.
 * On macOS the native menu bar takes over; on Windows/Linux the in-app bar is the primary UI.
 */
export function isAppMenuHidden(platform: string): boolean {
  return platform.startsWith("Mac");
}
