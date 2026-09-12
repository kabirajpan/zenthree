use zenthra::{Color, ImageSource};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ThemeMode {
    Dark,
    Light,
}

#[derive(Clone, Copy)]
pub struct ThemeColors {
    pub bg_base: Color,
    pub bg_sidebar: Color,
    pub bg_editor: Color,
    pub bg_gutter: Color,
    pub bg_active_tab: Color,
    pub bg_inactive_tab: Color,
    pub border: Color,
    pub text_primary: Color,
    pub text_muted: Color,
    pub text_dim: Color,
    pub accent: Color,
    pub highlight: Color,
}

pub const ACCENT_COLOR_OPTIONS: &[(&str, &str)] = &[
    ("blue", "Blue"),
    ("yellow", "Yellow"),
    ("green", "Green"),
    ("red", "Red"),
    ("orange", "Orange"),
    ("purple", "Purple"),
    ("pink", "Pink"),
    ("turquoise", "Turquoise"),
    ("gray", "Gray"),
];

pub fn named_color(name: &str, mode: ThemeMode) -> Color {
    match name {
        "yellow" => Color::rgb(255.0 / 255.0, 214.0 / 255.0, 0.0 / 255.0),
        "blue" => Color::rgb(99.0 / 255.0, 208.0 / 255.0, 223.0 / 255.0),
        "green" => Color::rgb(46.0 / 255.0, 204.0 / 255.0, 113.0 / 255.0),
        "red" => Color::rgb(252.0 / 255.0, 60.0 / 255.0, 56.0 / 255.0),
        "orange" => Color::rgb(253.0 / 255.0, 151.0 / 255.0, 31.0 / 255.0),
        "purple" => Color::rgb(155.0 / 255.0, 89.0 / 255.0, 182.0 / 255.0),
        "pink" => Color::rgb(236.0 / 255.0, 100.0 / 255.0, 165.0 / 255.0),
        "turquoise" => Color::rgb(26.0 / 255.0, 188.0 / 255.0, 156.0 / 255.0),
        "gray" => Color::rgb(136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0),
        _ => match mode {
            ThemeMode::Dark => Color::rgb(99.0 / 255.0, 208.0 / 255.0, 223.0 / 255.0),
            ThemeMode::Light => Color::rgb(41.0 / 255.0, 128.0 / 255.0, 185.0 / 255.0),
        },
    }
}

impl ThemeColors {
    #[allow(dead_code)]
    pub fn get(mode: ThemeMode) -> Self {
        Self::get_custom(mode, "blue")
    }

    pub fn get_custom(mode: ThemeMode, accent_name: &str) -> Self {
        let accent = named_color(accent_name, mode);
        match mode {
            ThemeMode::Dark => Self {
                bg_base: Color::rgb(0.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0),
                bg_sidebar: Color::rgb(1.0 / 255.0, 1.0 / 255.0, 1.0 / 255.0),
                bg_editor: Color::rgb(2.0 / 255.0, 2.0 / 255.0, 2.0 / 255.0),
                bg_gutter: Color::rgb(2.0 / 255.0, 2.0 / 255.0, 2.0 / 255.0),
                bg_active_tab: Color::rgb(2.0 / 255.0, 2.0 / 255.0, 2.0 / 255.0),
                bg_inactive_tab: Color::rgb(1.0 / 255.0, 1.0 / 255.0, 1.0 / 255.0),
                border: Color::rgb(3.0 / 255.0, 3.0 / 255.0, 3.0 / 255.0),
                text_primary: Color::rgb(224.0 / 255.0, 224.0 / 255.0, 224.0 / 255.0),
                text_muted: Color::rgb(136.0 / 255.0, 136.0 / 255.0, 136.0 / 255.0),
                text_dim: Color::rgb(55.0 / 255.0, 55.0 / 255.0, 55.0 / 255.0),
                accent,
                highlight: Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.04),
            },
            ThemeMode::Light => Self {
                bg_base: Color::rgb(240.0 / 255.0, 240.0 / 255.0, 242.0 / 255.0),
                bg_sidebar: Color::rgb(230.0 / 255.0, 230.0 / 255.0, 234.0 / 255.0),
                bg_editor: Color::WHITE,
                bg_gutter: Color::WHITE,
                bg_active_tab: Color::WHITE,
                bg_inactive_tab: Color::rgb(230.0 / 255.0, 230.0 / 255.0, 234.0 / 255.0),
                border: Color::rgb(210.0 / 255.0, 210.0 / 255.0, 215.0 / 255.0),
                text_primary: Color::rgb(17.0 / 255.0, 17.0 / 255.0, 17.0 / 255.0),
                text_muted: Color::rgb(90.0 / 255.0, 90.0 / 255.0, 95.0 / 255.0),
                text_dim: Color::rgb(190.0 / 255.0, 190.0 / 255.0, 195.0 / 255.0),
                accent,
                highlight: Color::rgba(0.0, 0.0, 0.0, 0.08),
            },
        }
    }
}

pub const ICON_THEME_OPTIONS: &[(&str, &str)] = &[
    ("neovim_mini", "Neovim (Mini.icons)"),
    ("neovim", "Neovim (Devicons)"),
    ("material", "Material Icons"),
    ("catppuccin", "Catppuccin Mocha"),
    ("gruvbox", "Gruvbox Plus"),
];

#[derive(Clone)]
pub enum FileIcon {
    Glyph {
        glyph: &'static str,
        color: Color,
    },
    Image(ImageSource),
}

impl FileIcon {
    pub fn show(&self, ui: &mut zenthra::Ui, size: f32) {
        let box_w = (size + 3.0).max(15.0);
        ui.container()
            .width(box_w)
            .valign(zenthra::Align::Center)
            .halign(zenthra::Align::Center)
            .show(|ui| {
                match self {
                    FileIcon::Glyph { glyph, color } => {
                        ui.text(glyph)
                            .size(size)
                            .color(*color)
                            .show();
                    }
                    FileIcon::Image(source) => {
                        ui.image(source.clone())
                            .size(size + 2.0, size + 2.0)
                            .fit(zenthra::ObjectFit::Contain)
                            .show();
                    }
                }
            });
    }
}

#[derive(Debug, serde::Deserialize)]
struct VsCodeIconDef {
    #[serde(rename = "iconPath", default)]
    icon_path: String,
}

#[derive(Debug, serde::Deserialize)]
struct VsCodeThemeJson {
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    folder: Option<String>,
    #[serde(rename = "folderExpanded", default)]
    folder_expanded: Option<String>,
    #[serde(rename = "fileExtensions", default)]
    file_extensions: std::collections::HashMap<String, String>,
    #[serde(rename = "fileNames", default)]
    file_names: std::collections::HashMap<String, String>,
    #[serde(rename = "folderNames", default)]
    folder_names: std::collections::HashMap<String, String>,
    #[serde(rename = "folderNamesExpanded", default)]
    folder_names_expanded: std::collections::HashMap<String, String>,
    #[serde(rename = "iconDefinitions", default)]
    icon_definitions: std::collections::HashMap<String, VsCodeIconDef>,
}

pub struct ParsedThemeData {
    pub theme_name: String,
    pub default_file_icon: String,
    pub default_folder_icon: String,
    pub default_folder_expanded_icon: String,
    pub file_extensions: std::collections::HashMap<String, String>,
    pub file_names: std::collections::HashMap<String, String>,
    pub folder_names: std::collections::HashMap<String, String>,
    pub folder_names_expanded: std::collections::HashMap<String, String>,
    pub icon_definitions: std::collections::HashMap<String, String>,
}

impl ParsedThemeData {
    pub fn get_folder_icon_path(&self, folder_name: &str, is_expanded: bool) -> String {
        let name_lower = folder_name.to_lowercase();

        let icon_id = if is_expanded {
            self.folder_names_expanded
                .get(&name_lower)
                .or_else(|| self.folder_names.get(&name_lower))
                .map(|s| s.as_str())
                .unwrap_or(&self.default_folder_expanded_icon)
        } else {
            self.folder_names
                .get(&name_lower)
                .map(|s| s.as_str())
                .unwrap_or(&self.default_folder_icon)
        };

        self.resolve_icon_path(icon_id)
    }

    pub fn get_file_icon_path(&self, file_name: &str, path: Option<&std::path::Path>) -> String {
        let name_lower = file_name.to_lowercase();

        // 1. Exact filename check (e.g. "cargo.toml", "package.json", ".gitignore", "dockerfile")
        if let Some(id) = self.file_names.get(&name_lower) {
            return self.resolve_icon_path(id);
        }

        // 2. Compound extension check (e.g. "test.tsx", "spec.ts", "d.ts")
        let parts: Vec<&str> = name_lower.split('.').collect();
        if parts.len() > 2 {
            for i in 1..parts.len() - 1 {
                let compound_ext = parts[i..].join(".");
                if let Some(id) = self.file_extensions.get(&compound_ext) {
                    return self.resolve_icon_path(id);
                }
            }
        }

        // 3. Single extension check
        let ext = path
            .and_then(|p| p.extension())
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .or_else(|| {
                if parts.len() > 1 {
                    Some(parts.last().unwrap().to_string())
                } else {
                    None
                }
            });

        if let Some(ref ext_str) = ext {
            if let Some(id) = self.file_extensions.get(ext_str) {
                return self.resolve_icon_path(id);
            }
        }

        // 4. Default fallback icon
        self.resolve_icon_path(&self.default_file_icon)
    }

    fn resolve_icon_path(&self, icon_id: &str) -> String {
        let file_name = self.icon_definitions.get(icon_id)
            .map(|s| s.as_str())
            .unwrap_or(icon_id);
        let mut fname = file_name.to_string();
        if !fname.ends_with(".svg") {
            fname.push_str(".svg");
        }
        format!("themes/{}/icons/{}", self.theme_name, fname)
    }
}

static THEME_CACHE: std::sync::OnceLock<std::sync::RwLock<std::collections::HashMap<String, std::sync::Arc<ParsedThemeData>>>> =
    std::sync::OnceLock::new();

fn get_or_load_theme_data(theme_name: &str) -> Option<std::sync::Arc<ParsedThemeData>> {
    let cache = THEME_CACHE.get_or_init(|| std::sync::RwLock::new(std::collections::HashMap::new()));

    if let Ok(read_guard) = cache.read() {
        if let Some(data) = read_guard.get(theme_name) {
            return Some(std::sync::Arc::clone(data));
        }
    }

    let theme_json_path = format!("themes/{}/theme.json", theme_name);
    let asset = crate::assets::Assets::get(&theme_json_path)?;
    let parsed: VsCodeThemeJson = serde_json::from_slice(&asset.data).ok()?;

    let mut icon_definitions = std::collections::HashMap::new();
    for (id, def) in parsed.icon_definitions {
        let file_name = def
            .icon_path
            .rsplit('/')
            .next()
            .unwrap_or(&def.icon_path)
            .to_string();
        icon_definitions.insert(id, file_name);
    }

    let default_file = parsed.file.unwrap_or_else(|| "file".to_string());
    let default_folder = parsed.folder.unwrap_or_else(|| "folder".to_string());
    let default_folder_expanded = parsed.folder_expanded.unwrap_or_else(|| "folder-open".to_string());

    let theme_data = std::sync::Arc::new(ParsedThemeData {
        theme_name: theme_name.to_string(),
        default_file_icon: default_file,
        default_folder_icon: default_folder,
        default_folder_expanded_icon: default_folder_expanded,
        file_extensions: parsed.file_extensions,
        file_names: parsed.file_names,
        folder_names: parsed.folder_names,
        folder_names_expanded: parsed.folder_names_expanded,
        icon_definitions,
    });

    if let Ok(mut write_guard) = cache.write() {
        write_guard.insert(theme_name.to_string(), std::sync::Arc::clone(&theme_data));
    }

    Some(theme_data)
}

fn get_gruvbox_file_icon(file_name: &str, ext: &str) -> &'static str {
    let name_lower = file_name.to_lowercase();
    let ext_lower = ext.to_lowercase();

    match name_lower.as_str() {
        "cargo.toml" => "text-x-rust.svg",
        "cargo.lock" => "package-x-generic.svg",
        "pom.xml" => "text-x-maven+xml.svg",
        "build.gradle" | "settings.gradle" => "text-x-groovy.svg",
        "package.json" => "text-javascript.svg",
        "package-lock.json" | "yarn.lock" | "pnpm-lock.yaml" | "pnpm-lock.yml" | "bun.lock" | "bun.lockb" => {
            "package-x-generic.svg"
        }
        "tsconfig.json" => "text-typescript.svg",
        "dockerfile" | "dockerfile.dev" | "dockerfile.prod" => "text-dockerfile.svg",
        "docker-compose.yml" | "docker-compose.yaml" => "text-dockerfile.svg",
        "makefile" | "cmakelists.txt" => "text-x-makefile.svg",
        "readme.md" | "readme" | "readme.txt" => "text-x-readme.svg",
        "license" | "licence" | "copying" => "text-x-copying.svg",
        ".gitignore" | ".gitattributes" | ".gitmodules" => "text-x-diff.svg",
        ".env" | ".env.local" | ".env.development" | ".env.production" => "application-x-config.svg",
        _ => match ext_lower.as_str() {
            "rs" => "text-x-rust.svg",
            "py" | "pyw" | "pyi" => "text-x-python.svg",
            "js" | "mjs" | "cjs" => "text-javascript.svg",
            "jsx" => "text-javascript.svg",
            "ts" | "mts" | "cts" => "text-typescript.svg",
            "tsx" => "text-typescript.svg",
            "go" => "text-x-go.svg",
            "java" => "text-x-java.svg",
            "kt" | "kts" => "text-x-kotlin.svg",
            "scala" => "text-x-scala.svg",
            "c" | "h" => "text-x-c.svg",
            "cpp" | "cc" | "cxx" | "hpp" => "text-x-cpp.svg",
            "zig" => "text-x-zig.svg",
            "lua" => "text-x-lua.svg",
            "html" | "htm" => "text-html.svg",
            "css" => "text-css.svg",
            "scss" | "sass" | "less" => "text-scss.svg",
            "json" | "jsonc" | "json5" => "text-json.svg",
            "toml" => "application-toml.svg",
            "yaml" | "yml" => "application-yaml.svg",
            "md" | "markdown" => "text-x-markdown.svg",
            "sh" | "bash" | "zsh" | "fish" => "application-x-executable-script.svg",
            "ps1" => "application-x-powershell.svg",
            "bat" | "cmd" => "application-x-executable-script.svg",
            "sql" => "text-x-sql.svg",
            "php" => "application-x-php.svg",
            "rb" => "text-x-ruby.svg",
            "svg" | "png" | "jpg" | "jpeg" | "gif" | "webp" | "ico" => "image-x-generic.svg",
            "mp3" | "wav" | "ogg" | "flac" => "audio-x-generic.svg",
            "mp4" | "mkv" | "avi" | "webm" | "mov" => "video-x-generic.svg",
            "zip" | "tar" | "gz" | "bz2" | "xz" | "rar" | "7z" => "application-x-archive.svg",
            "pdf" => "application-pdf.svg",
            "log" => "text-x-log.svg",
            _ => "unknown.svg",
        },
    }
}

pub struct IconTheme {
    pub name: String,
    data: Option<std::sync::Arc<ParsedThemeData>>,
}

impl IconTheme {
    pub fn new(name: &str) -> Self {
        let name_clean = if name.is_empty() { "neovim" } else { name };
        let data = get_or_load_theme_data(name_clean);
        Self {
            name: name_clean.to_string(),
            data,
        }
    }

    pub fn get_icon_for_folder(&self, folder_name: &str, is_expanded: bool) -> FileIcon {
        if self.name == "neovim_mini" || self.name == "mini" {
            let (glyph, color) = crate::mini_icons::get_mini_folder_icon(folder_name, is_expanded);
            return FileIcon::Glyph { glyph, color };
        }

        if self.name == "neovim" {
            let (glyph, color) = crate::neovim_icons::get_neovim_folder_icon(is_expanded);
            return FileIcon::Glyph { glyph, color };
        }

        if self.name == "gruvbox" {
            let path = "themes/gruvbox/folders/gray.png";
            if let Some(file) = crate::assets::Assets::get(path) {
                return FileIcon::Image(ImageSource::Bytes(std::sync::Arc::from(file.data.into_owned())));
            }
            return FileIcon::Image(ImageSource::Bytes(std::sync::Arc::from(&[][..])));
        }

        if let Some(ref data) = self.data {
            let path = data.get_folder_icon_path(folder_name, is_expanded);
            if let Some(file) = crate::assets::Assets::get(&path) {
                return FileIcon::Image(ImageSource::Bytes(std::sync::Arc::from(file.data.into_owned())));
            }
            // Fallback to default theme folder icon
            let fallback_name = if is_expanded { "folder-open.svg" } else { "folder.svg" };
            let fallback_path = format!("themes/{}/icons/{}", self.name, fallback_name);
            if let Some(file) = crate::assets::Assets::get(&fallback_path) {
                return FileIcon::Image(ImageSource::Bytes(std::sync::Arc::from(file.data.into_owned())));
            }
            let generic_folder = format!("themes/{}/icons/folder.svg", self.name);
            if let Some(file) = crate::assets::Assets::get(&generic_folder) {
                return FileIcon::Image(ImageSource::Bytes(std::sync::Arc::from(file.data.into_owned())));
            }
        }

        // Final fallback: Gruvbox folder
        if let Some(file) = crate::assets::Assets::get("themes/gruvbox/folders/gray.png") {
            FileIcon::Image(ImageSource::Bytes(std::sync::Arc::from(file.data.into_owned())))
        } else {
            FileIcon::Image(ImageSource::Bytes(std::sync::Arc::from(&[][..])))
        }
    }

    pub fn get_icon_for_file(&self, file_name: &str, path: Option<&std::path::Path>) -> FileIcon {
        if self.name == "neovim_mini" || self.name == "mini" {
            let name_lower = file_name.to_lowercase();
            let ext_lower = path
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_else(|| {
                    name_lower.rsplit('.').next().unwrap_or("").to_string()
                });
            let (glyph, color) = crate::mini_icons::get_mini_file_icon(&name_lower, &ext_lower);
            return FileIcon::Glyph { glyph, color };
        }

        if self.name == "neovim" {
            let name_lower = file_name.to_lowercase();
            let ext_lower = path
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_else(|| {
                    name_lower.rsplit('.').next().unwrap_or("").to_string()
                });
            let (glyph, color) = crate::neovim_icons::get_neovim_file_icon(&name_lower, &ext_lower);
            return FileIcon::Glyph { glyph, color };
        }

        if self.name == "gruvbox" {
            let ext = path
                .and_then(|p| p.extension())
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            let icon_file = get_gruvbox_file_icon(file_name, &ext);
            let full_path = format!("themes/gruvbox/Gruvbox-Plus-Dark/mimetypes/scalable/{}", icon_file);
            if let Some(file) = crate::assets::Assets::get(&full_path) {
                return FileIcon::Image(ImageSource::Bytes(std::sync::Arc::from(file.data.into_owned())));
            }
            return FileIcon::Image(ImageSource::Bytes(std::sync::Arc::from(&[][..])));
        }

        if let Some(ref data) = self.data {
            let icon_path = data.get_file_icon_path(file_name, path);
            if let Some(file) = crate::assets::Assets::get(&icon_path) {
                return FileIcon::Image(ImageSource::Bytes(std::sync::Arc::from(file.data.into_owned())));
            }
            // Fallback to theme's default file icon
            let fallback_path = format!("themes/{}/icons/file.svg", self.name);
            if let Some(file) = crate::assets::Assets::get(&fallback_path) {
                return FileIcon::Image(ImageSource::Bytes(std::sync::Arc::from(file.data.into_owned())));
            }
        }

        // Final fallback: Gruvbox unknown icon
        if let Some(file) = crate::assets::Assets::get("themes/gruvbox/Gruvbox-Plus-Dark/mimetypes/scalable/unknown.svg") {
            FileIcon::Image(ImageSource::Bytes(std::sync::Arc::from(file.data.into_owned())))
        } else {
            FileIcon::Image(ImageSource::Bytes(std::sync::Arc::from(&[][..])))
        }
    }

    #[allow(dead_code)]
    pub fn get_icon_source(&self, category: &str, extension: &str) -> ImageSource {
        let file_icon = if category == "folder" {
            self.get_icon_for_folder("", false)
        } else {
            let fake_name = if extension.is_empty() {
                "file".to_string()
            } else if extension.contains('.') {
                extension.to_string()
            } else {
                format!("file.{}", extension)
            };
            self.get_icon_for_file(&fake_name, None)
        };
        match file_icon {
            FileIcon::Image(src) => src,
            FileIcon::Glyph { .. } => ImageSource::Bytes(std::sync::Arc::from(&[][..])),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_neovim_icon_lookups() {
        let theme = IconTheme::new("neovim");
        let rust_icon = theme.get_icon_for_file("main.rs", Some(std::path::Path::new("main.rs")));
        assert!(matches!(rust_icon, FileIcon::Glyph { glyph, .. } if !glyph.is_empty()));

        let cargo_icon = theme.get_icon_for_file("Cargo.toml", Some(std::path::Path::new("Cargo.toml")));
        assert!(matches!(cargo_icon, FileIcon::Glyph { glyph, .. } if !glyph.is_empty()));

        let folder_icon = theme.get_icon_for_folder("src", false);
        assert!(matches!(folder_icon, FileIcon::Glyph { glyph, .. } if !glyph.is_empty()));
    }

    #[test]
    fn test_neovim_mini_icon_lookups() {
        let theme = IconTheme::new("neovim_mini");
        let rust_icon = theme.get_icon_for_file("main.rs", Some(std::path::Path::new("main.rs")));
        assert!(matches!(rust_icon, FileIcon::Glyph { glyph, .. } if !glyph.is_empty()));

        let cargo_icon = theme.get_icon_for_file("Cargo.toml", Some(std::path::Path::new("Cargo.toml")));
        assert!(matches!(cargo_icon, FileIcon::Glyph { glyph, .. } if !glyph.is_empty()));

        let folder_icon = theme.get_icon_for_folder("src", false);
        assert!(matches!(folder_icon, FileIcon::Glyph { glyph, .. } if !glyph.is_empty()));
    }

    #[test]
    fn test_material_icon_lookups() {
        let theme = IconTheme::new("material");
        assert!(theme.data.is_some(), "Material theme should load");

        let rust_icon = theme.get_icon_for_file("main.rs", Some(std::path::Path::new("main.rs")));
        assert!(matches!(rust_icon, FileIcon::Image(ImageSource::Bytes(b)) if !b.is_empty()));

        let cargo_icon = theme.get_icon_for_file("Cargo.toml", Some(std::path::Path::new("Cargo.toml")));
        assert!(matches!(cargo_icon, FileIcon::Image(ImageSource::Bytes(b)) if !b.is_empty()));

        let git_icon = theme.get_icon_for_file(".gitignore", Some(std::path::Path::new(".gitignore")));
        assert!(matches!(git_icon, FileIcon::Image(ImageSource::Bytes(b)) if !b.is_empty()));

        let src_folder = theme.get_icon_for_folder("src", false);
        assert!(matches!(src_folder, FileIcon::Image(ImageSource::Bytes(b)) if !b.is_empty()));
    }

    #[test]
    fn test_catppuccin_icon_lookups() {
        let theme = IconTheme::new("catppuccin");
        assert!(theme.data.is_some(), "Catppuccin theme should load");

        let rust_icon = theme.get_icon_for_file("main.rs", Some(std::path::Path::new("main.rs")));
        assert!(matches!(rust_icon, FileIcon::Image(ImageSource::Bytes(b)) if !b.is_empty()));
    }

    #[test]
    fn test_gruvbox_icon_lookups() {
        let theme = IconTheme::new("gruvbox");
        let rust_icon = theme.get_icon_for_file("main.rs", Some(std::path::Path::new("main.rs")));
        assert!(matches!(rust_icon, FileIcon::Image(ImageSource::Bytes(b)) if !b.is_empty()));
    }
}
