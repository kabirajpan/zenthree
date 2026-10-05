# Zenthree IDE — Keyboard Shortcuts

> All shortcuts are designed to be mnemonic and VSCode/Neovim friendly.

---

## 🪟 Panels — Smart Toggle

| Shortcut | Panel | Behavior |
|---|---|---|
| `Ctrl+E` | Explorer (file tree) | Hidden → Show + Focus / Focused → Close |
| `Ctrl+T` | Terminal | Hidden → Show + Focus / Focused → Close |
| `Ctrl+Shift+G` | Git | Hidden → Show + Focus / Focused → Close |
| `Ctrl+Shift+A` | AI panel | Hidden → Show + Focus / Focused → Close |
| `F11` | Focus mode | Hides title bar, activity bar, terminal & AI |

---

## 🗂️ Buffer Tabs

| Shortcut | Action |
|---|---|
| `Alt+←` | Previous tab |
| `Alt+→` | Next tab |
| `Alt+W` | Close current tab |

---

## 🧭 Focus Navigation

| Shortcut | Action |
|---|---|
| `Ctrl+1` | Jump to editor (from anywhere) |
| `Escape` | Unfocus panel → back to editor (panel stays open) |

---

## 🌲 File Tree (when focused)

### Navigation

| Vim Key | Arrow / Standard | Action |
|---|---|---|
| `j` | `↓` | Move down |
| `k` | `↑` | Move up |
| `l` or `Enter` | `→` or `Enter` | Open file / expand folder |
| `h` | `←` | Collapse folder / go to parent |

### File Operations

| Key | Action |
|---|---|
| `a` | Open LazyVim dialog (New file / folder / cd) |
| `A` | Open LazyVim dialog for new folder (`mkdir `) |
| `:` / `Shift+;` | Open LazyVim Command Palette |
| `d` / `Delete` | Open Delete Confirmation Modal (type to filter `1. No` / `2. Yes`, press <kbd>Enter</kbd> to confirm) |
| `r` / `F2` | Open Rename Modal (`Rename <name>`) |
| `R` | Refresh tree |
| `q` / `Escape` | Close tree / back to editor |

---

## ⚡ LazyVim Command Palette & Directory Switcher

Open with **`Shift + :`** (or `:` in Normal mode/File tree) or **`Ctrl + Shift + P`**, or press **`a`** in the File Tree.  
Close with **`<Escape>`**, **`Ctrl + C`**, clicking the **`Esc`** badge, or clicking outside the dialog.

### Directory Change Autocomplete & Suggestions
When typing `cd ` or `cd <path>` (e.g. `cd ~/` or `cd `):
- A floating dropdown automatically appears under the input showing matching subdirectories with folder tag icons.
- **`<ArrowDown>`** / **`Ctrl + n`**: Select next directory suggestion.
- **`<ArrowUp>`** / **`Ctrl + p`**: Select previous directory suggestion.
- **`<Tab>`**: Autocomplete selected directory into the path input to continue drill-down.
- **`<Enter>`**: Complete and immediately execute `cd` to the selected directory.
- Clicking any directory in the dropdown autocompletes it into the command line.

| Command | Action | Example |
|---|---|---|
| `cd <path>` / `cd: <path>` | Change workspace directory & terminal cwd (with live autocomplete dropdown) | `cd ~/` or `cd dsa/` |
| `touch <name>` / `file <name>` | Create new file & open in editor | `touch main.rs` or `components/App.tsx` |
| `mkdir <name>` / `<name>/` | Create new folder | `mkdir styles` or `utils/` |
| `!<command>` | Execute shell command in terminal | `!cargo check` or `!git status` |
| `:w` / `w` | Save current file | `:w` |
| `:q` / `q` | Close current tab | `:q` |
| `:wq` / `:x` | Save and close current tab | `:wq` |
| `:term` / `:terminal` | Open/focus integrated terminal | `:term` |
| `:e <path>` / `open <path>` | Open file in editor | `:e Cargo.toml` |

---

## ✏️ Editor

| Shortcut | Action |
|---|---|
| `Ctrl+S` | Save |
| `Ctrl+Shift+S` | Save as |
| `Ctrl+Z` | Undo |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Redo |
| `Ctrl+C` | Copy |
| `Ctrl+X` | Cut |
| `Ctrl+V` | Paste |
| `Ctrl+A` | Select all |
| `Ctrl+D` | Select word under cursor |
| `Ctrl+Shift+D` | Duplicate line down |
| `Ctrl+Shift+K` | Delete entire line |
| `Ctrl+Backspace` | Delete word to the left |
| `Ctrl+Delete` | Delete word to the right |
| `Ctrl+←` / `Ctrl+→` | Move word left / right |
| `Ctrl+Shift+←` / `Ctrl+Shift+→` | Select word left / right |
| `Shift+←` / `Shift+→` | Select character left / right |
| `Ctrl+↑` / `Ctrl+↓` | Move up / down by 5 lines |
| `Ctrl+Shift+↑` / `Ctrl+Shift+↓` | Select up / down by 5 lines |
| `Shift+↑` / `Shift+↓` | Select line up / down |
| `Home` / `End` | Move to line start / end |
| `Shift+Home` / `Shift+End` | Select to line start / end |
| `Ctrl+Home` / `Ctrl+End` | Jump to file top / bottom |
| `Ctrl+Shift+Home` / `Ctrl+Shift+End` | Select to file top / bottom |
| `PageUp` / `PageDown` | Move up / down 10 lines |
| `Shift+PageUp` / `Shift+PageDown` | Select up / down 10 lines |
| `Ctrl+F` | Find in file |
| `Ctrl+Shift+F` | Global search / file search |
| `Ctrl+Space` | Trigger autocomplete |
| `Ctrl+,` | Open settings |

---

## 💻 Terminal (when focused)

### Navigation & Word Movement

| Shortcut | Action |
|---|---|
| `Ctrl+←` / `Ctrl+→` | Move backward / forward by word |
| `Ctrl+Shift+←` / `Ctrl+Shift+→` | Move/select word left / right |
| `Ctrl+↑` / `Ctrl+↓` | Multi-line jump up / down |
| `Ctrl+Shift+↑` / `Ctrl+Shift+↓` | Extended jump up / down |
| `Ctrl+Home` / `Ctrl+End` | Jump to buffer start / end |
| `Ctrl+Shift+Home` / `Ctrl+Shift+End` | Extended buffer start / end |
| `Ctrl+A` / `Ctrl+E` | Jump to line start / end |
| `Shift+Tab` | Reverse tab / backtab |

### Scrolling & Scrollback

| Shortcut | Action |
|---|---|
| `Shift+PageUp` / `Shift+PageDown` | Scroll terminal history up / down by 15 lines |
| `Shift+↑` / `Shift+↓` | Scroll terminal history up / down by 1 line |
| `Shift+Home` / `Shift+End` | Scroll to top / bottom of scrollback history |

### Editing & Line Control

| Shortcut | Action |
|---|---|
| `Ctrl+Backspace` | Delete word to the left |
| `Ctrl+Delete` | Delete word to the right |
| `Alt+Backspace` | Backward kill word (readline) |
| `Ctrl+U` | Delete from cursor to start of line |
| `Ctrl+K` | Delete from cursor to end of line |
| `Ctrl+W` | Rubout word backward |

### Clipboard & Process Control

| Shortcut | Action |
|---|---|
| `Ctrl+Shift+C` | Copy terminal output to clipboard |
| `Ctrl+Shift+V` / `Ctrl+V` / `Shift+Insert` | Paste clipboard into terminal |
| `Ctrl+Shift+K` | Clear terminal scrollback & buffer |
| `Ctrl+C` | Interrupt / SIGINT |
| `Ctrl+D` | EOF / Exit session |
| `Ctrl+Z` | Suspend / SIGTSTP |
| `Ctrl+L` | Clear screen |
| `Ctrl+R` | Reverse search history |
| `Ctrl+P` / `Ctrl+N` | Previous / next command history |

---

## 📋 Summary

| Category | Count |
|---|---|
| Panel toggles | 5 |
| Tab navigation | 3 |
| Focus navigation | 2 |
| File tree | 10 |
| Editor | 28 |
| Terminal | 24 |
| **Total** | **72** |

