# Zenthree

A high-performance, lightweight, and modern code editor built in Rust using the **[Zenthra](https://github.com/kabirajpan/zenthra-v2)** GPU-accelerated GUI framework.

Designed for speed, visual clarity, and responsiveness, Zenthree blends the aesthetic minimalism of modern editors with the responsiveness of native systems programming.

---

## ✨ Features

### ⚡ Editor Engine
- **Tree-sitter Syntax Highlighting**: Grammar-based highlighting for Rust, JavaScript, TypeScript/TSX, Python, HTML, CSS, JSON, YAML, Go, C/C++, Java, Markdown, and Bash/Shell, with regex fallback for other supported file types.
- **Optimized Buffer & Line Cache**: Zero-allocation line retrieval for fluid scrolling and editing.
- **Full Text Editing Suite**: Undo/redo history, multi-tab editing, word-by-word cursor jumping and deletion (`Ctrl+Backspace`, `Ctrl+Delete`), clipboard operations (`Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V`).
- **Interactive Breadcrumb Navigation**: Path trail with clickable directory popups for quick folder navigation.
- **Code Formatting**: Automatic document formatting integration (`Ctrl+Shift+F`).
- **Context Menus**: Right-click in the editor or terminal for editing, file, and panel actions. File operations target the active file; creation uses its folder.

### 📂 File Explorer
- **Clean Minimal Layout**: Zed-inspired minimalist file tree without distracting ASCII branch guide lines.
- **Sub-Pixel Column Alignment**: Fixed 12px chevron slot keeps file and folder icons in alignment.
- **Multiple Explorer Styles**: Choose between **Minimal**, **Framed Box**, and **Indented Tree**.
- **Git Status Badges**: Visual indicators for Modified (`M`) and Untracked (`U`) files.
- **Workspace Actions**: Quick toolbar buttons for creating files, collapsing folders, and refreshing the file tree.

### 🖥️ Native Embedded Terminal
- **Integrated PTY**: Full pseudo-terminal session powered by `portable-pty`.
- **Run Active File**: Launch supported Python, JavaScript, TypeScript (Deno), Go, Ruby, PHP, Lua, shell, and PowerShell files from the title-bar Run menu. JSX/TSX files run the nearest project's `dev`, `start`, or `serve` package script using its detected package manager.
- **Terminal Sessions**: Keep live terminal sessions in a selectable right-side list, create or close sessions, and split active sessions side by side. Drag to select terminal text, then use `Ctrl+C` or `Ctrl+Insert` to copy it; `Ctrl+C` still interrupts commands when there is no selection.
- **Title-bar Terminal Menu**: Create, switch, split, focus, hide, clear, or close terminal sessions. Run the active file or selected editor text, launch workspace tasks, run the default build task, and configure task files from the Terminal menu after Run.
- **Workspace Tasks**: Read and run shell tasks from `.vscode/tasks.json`, including `${workspaceFolder}`, `${file}`, `${fileBasename}`, and `${fileDirname}` substitutions. Configure Tasks opens or creates the workspace task file.
- **Bottom Panel Views**: Switch between Problems (LSP diagnostics, with file navigation), Output (recent IDE status messages), Debug Console (reports when no debugger is active), Terminal, and Ports (local listening sockets with process IDs when available).
- **Direct Inline Console Prompt**: Type directly inside the console (`❯ command█`) with an animated blinking block cursor.
- **Command Syntax Highlighting**: Color command names, flags, quoted arguments, variables, paths, and shell operators on the active command line.
- **Terminal Productivity**: Command history (`Up`/`Down` or `Ctrl+Up`/`Ctrl+Down`), process interrupt (`Ctrl+C`), screen clear (`Ctrl+L`), and tab completion (`Tab`).
- **Resizable Splitter**: Smooth horizontal resizing with a visual accent pill grip handle and `RowResize` cursor feedback.

### 🤖 AI Assistant Panel
- **Integrated Sidecar**: Collapsible side panel for code explanation, debugging, and drafting snippets.
- **Space-Saving Diff Review Bar**: Unified modified files container with compact diff badges (`+N` / `-M`), one-click file navigation, and instant `Accept all` / `Reject all` controls.
- **Structured Output**: Clear separation between thought streams, explanations, and formatted code blocks.
- **Independent Edge Resizing**: Isolated resize splitter with accent grip handle.

### 🧩 Language Support & Extensions
- **Automatic Language Detection & Prompt**: Opening an uninstalled language file (e.g. `.go`, `.ts`, `.cpp`, `.html`, `.lua`, etc.) prompts a floating modal dialog to install full language support, syntax highlighting, and tooling.
- **Smart Startup Prompting**: Shows again on startup if the language pack remains uninstalled, with a `"Don't ask again for <Language>"` checkbox to permanently dismiss only when desired.
- **Extensions Panel**: Dedicated sidebar panel (`Ctrl+Shift+X` or switcher tab) listing all language extensions with search, category tabs (`All`, `Installed`, `Available`), and one-click `[Install]` / `[Uninstall]`.
- **Status Bar Integration**: Displays the active file's language badge with immediate installation prompt or shortcut to the Extensions view.

### 🎨 Visuals & Color Fidelity
- **True Color Pipeline**: Direct linear swapchain presentation (`Rgba8Unorm`/`Bgra8Unorm`) ensuring 1:1 color accuracy with zero sRGB double-gamma washout.
- **Themes & Accents**: Polished Dark and Light themes accompanied by 9 saturated accent palettes (Blue, Yellow, Green, Red, Orange, Purple, Pink, Turquoise, Gray).
- **Icon Themes**:
  - **Minimalist** (Default, based on Mini NerdFont glyphs)
  - **Developer** (Devicons)
  - **Material**
  - **Pastel** (Catppuccin)
  - **Retro** (Gruvbox)

---

## 🏗️ Architecture

Zenthree is organized as a Cargo workspace containing four modular crates:

```
apps/zenthree/
├── Cargo.toml
├── assets/                 # Embedded fonts (Nerd Fonts) and theme icon packs
└── crates/
    ├── zenthree/           # Main application binary, UI layout, PTY terminal & state
    ├── editor/             # Text buffer, line caching, selection & editing engine
    ├── syntax/             # Tree-sitter highlighter & indentation engine
    └── formatter/          # Code formatting integration
```

---

## 🚀 Getting Started

### Prerequisites
- **Rust toolchain** (Rust 1.80+ recommended): [rustup.rs](https://rustup.rs)
- **Vulkan / Metal / DirectX 12** drivers supported by WGPU

### Build & Run

1. Clone the repository:
   ```bash
   git clone git@github.com:kabirajpan/zenthree.git
   cd zenthree
   ```

2. Run in development mode:
   ```bash
   cargo run --bin zenthree
   ```

3. Open a specific project directory or file:
   ```bash
   cargo run --bin zenthree -- /path/to/your/project
   ```

4. Build the optimized release binary:
   ```bash
   cargo build --release --bin zenthree
   ```
   The binary will be available at `target/release/zenthree`.

---

## ⌨️ Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Ctrl + N` | New untitled file |
| `Ctrl + O` | Open file dialog |
| `Ctrl + S` | Save active file |
| `Ctrl + W` | Close active tab |
| `Ctrl + B` | Toggle File Explorer sidebar |
| `Ctrl + ` ` ` (Backtick) | Toggle Terminal panel |
| `Ctrl + A` | Select all text |
| `Ctrl + C` / `Ctrl + X` / `Ctrl + V` | Copy / Cut / Paste |
| `Ctrl + Z` / `Ctrl + Y` | Undo / Redo |
| `Ctrl + Backspace` | Delete previous word |
| `Ctrl + Delete` | Delete next word |
| `Ctrl + Shift + F` | Format document |
| `Ctrl + =` / `Ctrl + -` | Zoom in / Zoom out |
| `Ctrl + 0` | Reset zoom level |

---

## ⚙️ Configuration & State Persistence

Zenthree automatically saves workspace state, including:
- Active tabs and cursor positions
- Open project directories and expanded folders
- Selected themes, accent colors, and icon themes
- Panel dimensions and visibility states

State configuration is stored at:
- **Linux**: `~/.config/zenthree/state.json`
- **macOS**: `~/Library/Application Support/zenthree/state.json`
- **Windows**: `%APPDATA%\zenthree\state.json`

---

## 📄 License

Licensed under the [MIT License](LICENSE) or Apache-2.0 at your option.
