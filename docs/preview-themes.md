# Preview Themes

NEditor ships six bundled preview themes and supports user-defined CSS themes
that hot-reload while you edit them.

## Theme file structure

A preview theme is a single self-contained `.css` file.  The root selector is:

```css
.preview-pane[data-preview-theme="my-theme"] .preview-document { … }
```

The `data-preview-theme` attribute on `.preview-pane` is set to the theme **id**,
which is the CSS filename without the `.css` extension (e.g. `github-light`).

Comments at the top of the file are used as the description shown in the theme
picker:

```css
/* My custom dark theme for technical writing */
.preview-pane[data-preview-theme="my-theme"] { … }
```

## Bundled themes

| Id | Visual identity |
|---|---|
| `github-light` | GitHub-style light, generous whitespace |
| `github-dark` | GitHub dark, same metrics on dark background |
| `serif-manuscript` | Warm off-white paper, Georgia serif, 18 px justified |
| `newspaper` | Two-column broadsheet, press typography |
| `terminal` | Green-on-black monospace, CRT scanline effect |
| `academic` | LaTeX-inspired two-column, Palatino, booktabs tables |

## Adding a user theme

Drop a `.css` file into the user themes directory for your platform:

| Platform | Path |
|---|---|
| macOS | `~/Library/Application Support/neditor/preview-themes/` |
| Windows | `%APPDATA%\neditor\preview-themes\` |
| Linux | `~/.local/share/neditor/preview-themes/` |

Use **Settings → Open User Themes Folder** (or the `open_user_themes_dir` IPC
command) to reveal the directory and create it if it does not exist yet.

If a user theme has the same id (filename stem) as a bundled theme, the user
theme takes precedence.

## Hot-reload

`watch_preview_theme(id)` installs a file-system watcher on the user CSS file
for the given theme.  When the file changes on disk, NEditor emits a
`preview-theme-changed` event with `{ id, css }` so the frontend can update the
preview pane without a full reload.  Call `unwatch_preview_theme()` to stop
watching.

Only user themes can be watched at runtime (bundled themes are read-only
resources).

## Copying a bundled theme as a starting point

1. Open the user themes directory (Settings → Open User Themes Folder).
2. Copy the bundled CSS file from the app resources:
   - macOS: `NEditor.app/Contents/Resources/themes/preview/<id>.css`
3. Rename the copy to your desired id (e.g. `my-theme.css`).
4. Edit the selector prefix from `data-preview-theme="<id>"` to
   `data-preview-theme="my-theme"` throughout the file.
5. NEditor will pick up the new theme on next launch or after
   `list_preview_themes` is called.
