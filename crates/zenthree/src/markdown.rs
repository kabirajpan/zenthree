use pulldown_cmark::{Event, HeadingLevel, Parser, Tag, TagEnd};
use zenthra::prelude::*;
use zenthra_widgets::container::Wrap;
use crate::theme::{ThemeColors, ThemeMode};

#[derive(Debug, Clone, PartialEq)]
pub enum InlineItem {
    Text(String),
    Bold(String),
    Italic(String),
    Code(String),
    Link { text: String, url: String },
}

#[derive(Debug, Clone, PartialEq)]
pub enum MarkdownBlock {
    Heading {
        level: u32,
        inlines: Vec<InlineItem>,
    },
    Paragraph {
        inlines: Vec<InlineItem>,
    },
    CodeBlock {
        lang: String,
        code: String,
    },
    List {
        ordered: bool,
        items: Vec<Vec<InlineItem>>,
    },
    BlockQuote {
        inlines: Vec<InlineItem>,
    },
    Rule,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SpanStyle {
    Normal,
    Bold,
    Italic,
    Code,
    Link(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordToken {
    pub text: String,
    pub style: SpanStyle,
}

pub fn parse_markdown(input: &str) -> Vec<MarkdownBlock> {
    let parser = Parser::new(input);
    let mut blocks = Vec::new();

    let mut current_inlines: Vec<InlineItem> = Vec::new();
    let mut in_strong = false;
    let mut in_emphasis = false;
    let mut current_link_url: Option<String> = None;
    let mut in_code_block: Option<String> = None;
    let mut code_block_buf = String::new();
    let mut in_list: Option<bool> = None;
    let mut list_items: Vec<Vec<InlineItem>> = Vec::new();
    let mut in_item = false;
    let mut heading_level: Option<u32> = None;

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading { level, .. } => {
                    let lvl = match level {
                        HeadingLevel::H1 => 1,
                        HeadingLevel::H2 => 2,
                        HeadingLevel::H3 => 3,
                        HeadingLevel::H4 => 4,
                        HeadingLevel::H5 => 5,
                        HeadingLevel::H6 => 6,
                    };
                    heading_level = Some(lvl);
                    current_inlines.clear();
                }
                Tag::Paragraph => {
                    if !in_item {
                        current_inlines.clear();
                    }
                }
                Tag::CodeBlock(kind) => {
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => lang.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                    in_code_block = Some(lang);
                    code_block_buf.clear();
                }
                Tag::List(start) => {
                    in_list = Some(start.is_some());
                    list_items.clear();
                }
                Tag::Item => {
                    in_item = true;
                    current_inlines.clear();
                }
                Tag::BlockQuote(_) => {
                    current_inlines.clear();
                }
                Tag::Strong => {
                    in_strong = true;
                }
                Tag::Emphasis => {
                    in_emphasis = true;
                }
                Tag::Link { dest_url, .. } => {
                    current_link_url = Some(dest_url.to_string());
                }
                _ => {}
            },
            Event::End(tag_end) => match tag_end {
                TagEnd::Heading(_) => {
                    if let Some(lvl) = heading_level.take() {
                        blocks.push(MarkdownBlock::Heading {
                            level: lvl,
                            inlines: std::mem::take(&mut current_inlines),
                        });
                    }
                }
                TagEnd::Paragraph => {
                    if !in_item && !current_inlines.is_empty() {
                        blocks.push(MarkdownBlock::Paragraph {
                            inlines: std::mem::take(&mut current_inlines),
                        });
                    }
                }
                TagEnd::CodeBlock => {
                    if let Some(lang) = in_code_block.take() {
                        let code = std::mem::take(&mut code_block_buf);
                        blocks.push(MarkdownBlock::CodeBlock {
                            lang,
                            code: code.trim_end().to_string(),
                        });
                    }
                }
                TagEnd::Item => {
                    in_item = false;
                    list_items.push(std::mem::take(&mut current_inlines));
                }
                TagEnd::List(_) => {
                    if let Some(ordered) = in_list.take() {
                        blocks.push(MarkdownBlock::List {
                            ordered,
                            items: std::mem::take(&mut list_items),
                        });
                    }
                }
                TagEnd::BlockQuote(_) => {
                    if !current_inlines.is_empty() {
                        blocks.push(MarkdownBlock::BlockQuote {
                            inlines: std::mem::take(&mut current_inlines),
                        });
                    }
                }
                TagEnd::Strong => {
                    in_strong = false;
                }
                TagEnd::Emphasis => {
                    in_emphasis = false;
                }
                TagEnd::Link => {
                    current_link_url = None;
                }
                _ => {}
            },
            Event::Text(text) => {
                let s = text.to_string();
                if in_code_block.is_some() {
                    code_block_buf.push_str(&s);
                } else if let Some(ref url) = current_link_url {
                    current_inlines.push(InlineItem::Link {
                        text: s,
                        url: url.clone(),
                    });
                } else if in_strong {
                    current_inlines.push(InlineItem::Bold(s));
                } else if in_emphasis {
                    current_inlines.push(InlineItem::Italic(s));
                } else {
                    current_inlines.push(InlineItem::Text(s));
                }
            }
            Event::Code(code) => {
                current_inlines.push(InlineItem::Code(code.to_string()));
            }
            Event::Rule => {
                blocks.push(MarkdownBlock::Rule);
            }
            Event::SoftBreak | Event::HardBreak => {
                if in_code_block.is_some() {
                    code_block_buf.push('\n');
                } else {
                    current_inlines.push(InlineItem::Text(" ".to_string()));
                }
            }
            _ => {}
        }
    }

    if !current_inlines.is_empty() {
        blocks.push(MarkdownBlock::Paragraph {
            inlines: current_inlines,
        });
    }

    blocks
}

/// Tokenizes inline elements into word tokens for fluid line wrapping with rich preview styling
pub fn tokenize_inlines(inlines: &[InlineItem]) -> Vec<WordToken> {
    let mut tokens: Vec<WordToken> = Vec::new();

    for item in inlines {
        match item {
            InlineItem::Text(s) => {
                let words: Vec<&str> = s.split_whitespace().collect();
                for (idx, &w) in words.iter().enumerate() {
                    // If leading punctuation like : or , attach to preceding word if available
                    if idx == 0 && (w == ":" || w == "," || w == "." || w == ";" || w == "!" || w == "?") {
                        if let Some(prev) = tokens.last_mut() {
                            if prev.style != SpanStyle::Code {
                                prev.text.push_str(w);
                                continue;
                            }
                        }
                    }
                    tokens.push(WordToken {
                        text: w.to_string(),
                        style: SpanStyle::Normal,
                    });
                }
            }
            InlineItem::Bold(s) => {
                let words: Vec<&str> = s.split_whitespace().collect();
                for w in words {
                    tokens.push(WordToken {
                        text: w.to_string(),
                        style: SpanStyle::Bold,
                    });
                }
            }
            InlineItem::Italic(s) => {
                let words: Vec<&str> = s.split_whitespace().collect();
                for w in words {
                    tokens.push(WordToken {
                        text: w.to_string(),
                        style: SpanStyle::Italic,
                    });
                }
            }
            InlineItem::Code(code) => {
                tokens.push(WordToken {
                    text: code.clone(),
                    style: SpanStyle::Code,
                });
            }
            InlineItem::Link { text, url } => {
                let words: Vec<&str> = text.split_whitespace().collect();
                for w in words {
                    tokens.push(WordToken {
                        text: w.to_string(),
                        style: SpanStyle::Link(url.clone()),
                    });
                }
            }
        }
    }

    tokens
}

/// Renders parsed markdown blocks as clean, rich preview widgets
pub fn render_markdown_preview(
    ui: &mut Ui,
    content: &str,
    max_w: f32,
    colors: &ThemeColors,
    theme: ThemeMode,
    copy_action: &mut Option<String>,
    collapsed_blocks: &mut std::collections::HashSet<u64>,
) {
    let blocks = parse_markdown(content);

    let code_badge_bg = match theme {
        ThemeMode::Dark => Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.08),
        ThemeMode::Light => Color::rgba(0.0, 0.0, 0.0, 0.05),
    };

    let block_code_bg = match theme {
        ThemeMode::Dark => Color::rgb(22.0 / 255.0, 25.0 / 255.0, 30.0 / 255.0),
        ThemeMode::Light => Color::rgb(240.0 / 255.0, 243.0 / 255.0, 246.0 / 255.0),
    };

    for block in blocks {
        match block {
            MarkdownBlock::Heading { level, inlines } => {
                let tokens = tokenize_inlines(&inlines);
                let (font_size, font_weight, text_color, pad_top) = match level {
                    1 => (13.5f32, FontWeight::Bold, colors.accent, 6.0f32),
                    2 => (12.0f32, FontWeight::Bold, colors.text_primary, 5.0f32),
                    3 => (11.0f32, FontWeight::Bold, colors.accent, 4.0f32),
                    _ => (10.5f32, FontWeight::Bold, colors.text_muted, 3.0f32),
                };

                ui.container()
                    .full_width()
                    .padding(pad_top, 0.0, 2.0, 0.0)
                    .row()
                    .wrap(Wrap::Wrap)
                    .gap(4.0)
                    .valign(Align::Center)
                    .show(|ui| {
                        for token in tokens {
                            if token.style == SpanStyle::Code {
                                ui.container()
                                    .padding(1.0, 4.0, 1.0, 4.0)
                                    .radius_all(3.0)
                                    .bg(code_badge_bg)
                                    .border(colors.border, 1.0)
                                    .show(|ui| {
                                        ui.text(&token.text)
                                            .size(font_size - 1.0)
                                            .monospace()
                                            .color(colors.accent)
                                            .show();
                                    });
                            } else {
                                ui.text(&token.text)
                                    .size(font_size)
                                    .weight(font_weight)
                                    .color(text_color)
                                    .show();
                            }
                        }
                    });
            }

            MarkdownBlock::Paragraph { inlines } => {
                let tokens = tokenize_inlines(&inlines);
                if tokens.is_empty() {
                    continue;
                }

                ui.container()
                    .full_width()
                    .padding(2.0, 0.0, 3.0, 0.0)
                    .row()
                    .wrap(Wrap::Wrap)
                    .gap(3.5)
                    .valign(Align::Center)
                    .show(|ui| {
                        for token in tokens {
                            render_token_widget(ui, &token, colors, code_badge_bg);
                        }
                    });
            }

            MarkdownBlock::CodeBlock { lang, code } => {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                code.hash(&mut hasher);
                lang.hash(&mut hasher);
                let block_id = hasher.finish();

                let is_collapsed = collapsed_blocks.contains(&block_id);
                let line_count = code.lines().count();
                let is_scrollable = line_count > 10;
                let display_lang = if lang.trim().is_empty() { "code" } else { lang.trim() };
                let block_card_w = (max_w - 4.0).max(60.0);

                ui.container()
                    .width(block_card_w)
                    .padding(5.0, 7.0, 5.0, 7.0)
                    .bg(block_code_bg)
                    .border(colors.border, 1.0)
                    .radius_all(6.0)
                    .column()
                    .gap(4.0)
                    .clip(true)
                    .show(|ui| {
                        // Top toolbar
                        ui.container()
                            .full_width()
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                // Left: Collapse toggle arrow + Language + Line count
                                let arrow_icon = if is_collapsed { "\u{f054}" } else { "\u{f078}" };
                                let toggle_btn = ui.container()
                                    .padding(2.0, 4.0, 2.0, 4.0)
                                    .radius_all(3.0)
                                    .hover_bg(colors.highlight)
                                    .row()
                                    .valign(Align::Center)
                                    .gap(5.0)
                                    .show(|ui| {
                                        ui.container()
                                            .width(12.0)
                                            .valign(Align::Center)
                                            .halign(Align::Center)
                                            .show(|ui| {
                                                ui.text(arrow_icon)
                                                    .size(7.5)
                                                    .color(colors.text_muted)
                                                    .show();
                                            });

                                        ui.text(display_lang)
                                            .size(8.5)
                                            .monospace()
                                            .weight(FontWeight::Bold)
                                            .color(colors.text_primary)
                                            .show();

                                        ui.text(&format!("({} line{})", line_count, if line_count == 1 { "" } else { "s" }))
                                            .size(8.0)
                                            .color(colors.text_muted)
                                            .show();
                                    });

                                if toggle_btn.clicked {
                                    if is_collapsed {
                                        collapsed_blocks.remove(&block_id);
                                    } else {
                                        collapsed_blocks.insert(block_id);
                                    }
                                    ui.needs_redraw = true;
                                }

                                // Right: Copy button
                                let copy_btn = ui.container()
                                    .padding(2.0, 6.0, 2.0, 6.0)
                                    .radius_all(3.0)
                                    .bg(colors.bg_base)
                                    .border(colors.border, 1.0)
                                    .hover_bg(colors.highlight)
                                    .show(|ui| {
                                        ui.text("Copy")
                                            .size(8.0)
                                            .color(colors.text_muted)
                                            .show();
                                    });

                                if copy_btn.clicked {
                                    *copy_action = Some(code.clone());
                                }
                            });

                        // Monospace code display: capped at 10 lines (~155px), scroll_y when >10 lines, clip to prevent overflow
                        if !is_collapsed {
                            let scroll_id = zenthra_core::Id::from_u64(block_id ^ 0xC0DE_5C80_11);
                            let mut code_box = ui.container()
                                .id(scroll_id)
                                .full_width()
                                .padding(2.0, 4.0, 2.0, 4.0)
                                .clip(true);

                            if is_scrollable {
                                code_box = code_box
                                    .height(155.0)
                                    .scroll_y(true)
                                    .scroll_x(true);
                            } else {
                                code_box = code_box.scroll_x(true);
                            }

                            code_box.show(|ui| {
                                ui.text(&code)
                                    .size(10.0)
                                    .line_height(1.35)
                                    .monospace()
                                    .color(colors.text_primary)
                                    .max_width((block_card_w - 20.0).max(40.0))
                                    .wrap(zenthra_text::prelude::TextWrap::None)
                                    .show();
                            });
                        }
                    });
            }

            MarkdownBlock::List { ordered, items } => {
                ui.container()
                    .full_width()
                    .padding(1.0, 0.0, 2.0, 0.0)
                    .column()
                    .gap(2.5)
                    .show(|ui| {
                        for (idx, item_inlines) in items.iter().enumerate() {
                            let tokens = tokenize_inlines(item_inlines);
                            ui.container()
                                .full_width()
                                .row()
                                .no_wrap()
                                .gap(6.0)
                                .valign(Align::Top)
                                .show(|ui| {
                                    if ordered {
                                        ui.text(&format!("{}.", idx + 1))
                                            .size(10.0)
                                            .weight(FontWeight::Bold)
                                            .color(colors.accent)
                                            .show();
                                    } else {
                                        ui.text("•")
                                            .size(10.0)
                                            .color(colors.accent)
                                            .show();
                                    }

                                    ui.container()
                                        .full_width()
                                        .row()
                                        .wrap(Wrap::Wrap)
                                        .gap(3.5)
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            for token in tokens {
                                                render_token_widget(ui, &token, colors, code_badge_bg);
                                            }
                                        });
                                });
                        }
                    });
            }

            MarkdownBlock::BlockQuote { inlines } => {
                let tokens = tokenize_inlines(&inlines);
                ui.container()
                    .full_width()
                    .padding(3.0, 0.0, 3.0, 0.0)
                    .row()
                    .no_wrap()
                    .gap(6.0)
                    .show(|ui| {
                        ui.container()
                            .width(2.5)
                            .height(14.0)
                            .radius_all(1.0)
                            .bg(colors.accent.with_alpha(0.7))
                            .show(|_| {});

                        ui.container()
                            .full_width()
                            .row()
                            .wrap(Wrap::Wrap)
                            .gap(3.5)
                            .valign(Align::Center)
                            .show(|ui| {
                                for token in tokens {
                                    ui.text(&token.text)
                                        .size(10.0)
                                        .italic()
                                        .color(colors.text_muted)
                                        .show();
                                }
                            });
                    });
            }

            MarkdownBlock::Rule => {
                ui.container()
                    .full_width()
                    .padding(4.0, 0.0, 4.0, 0.0)
                    .column()
                    .show(|ui| {
                        ui.container()
                            .full_width()
                            .height(1.0)
                            .bg(colors.border)
                            .show(|_| {});
                    });
            }
        }
    }
}

fn render_token_widget(
    ui: &mut Ui,
    token: &WordToken,
    colors: &ThemeColors,
    code_badge_bg: Color,
) {
    match &token.style {
        SpanStyle::Normal => {
            ui.text(&token.text)
                .size(10.5)
                .color(colors.text_primary)
                .show();
        }
        SpanStyle::Bold => {
            ui.text(&token.text)
                .size(10.5)
                .weight(FontWeight::Bold)
                .color(colors.text_primary)
                .show();
        }
        SpanStyle::Italic => {
            ui.text(&token.text)
                .size(10.5)
                .italic()
                .color(colors.text_muted)
                .show();
        }
        SpanStyle::Code => {
            ui.container()
                .padding(1.0, 4.0, 1.0, 4.0)
                .radius_all(3.0)
                .bg(code_badge_bg)
                .border(colors.border, 1.0)
                .show(|ui| {
                    ui.text(&token.text)
                        .size(9.5)
                        .monospace()
                        .color(colors.accent)
                        .show();
                });
        }
        SpanStyle::Link(_) => {
            ui.text(&token.text)
                .size(10.5)
                .weight(FontWeight::Medium)
                .color(colors.accent)
                .show();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_headings_and_code() {
        let md = "## Title\n\nThis is **bold** and `code`.\n\n```rust\nfn main() {}\n```\n";
        let blocks = parse_markdown(md);
        assert_eq!(blocks.len(), 3);
        match &blocks[0] {
            MarkdownBlock::Heading { level, inlines } => {
                assert_eq!(*level, 2);
                assert_eq!(inlines, &vec![InlineItem::Text("Title".to_string())]);
            }
            _ => panic!("Expected heading"),
        }
        match &blocks[1] {
            MarkdownBlock::Paragraph { inlines } => {
                assert_eq!(inlines.len(), 5);
                assert_eq!(inlines[0], InlineItem::Text("This is ".to_string()));
                assert_eq!(inlines[1], InlineItem::Bold("bold".to_string()));
                assert_eq!(inlines[2], InlineItem::Text(" and ".to_string()));
                assert_eq!(inlines[3], InlineItem::Code("code".to_string()));
                assert_eq!(inlines[4], InlineItem::Text(".".to_string()));
            }
            _ => panic!("Expected paragraph"),
        }
        match &blocks[2] {
            MarkdownBlock::CodeBlock { lang, code } => {
                assert_eq!(lang, "rust");
                assert_eq!(code, "fn main() {}");
            }
            _ => panic!("Expected code block"),
        }
    }

    #[test]
    fn test_tokenize_inlines_punctuation_and_badges() {
        let inlines = vec![
            InlineItem::Bold("Ecosystem".to_string()),
            InlineItem::Text(": Rust / Cargo".to_string()),
            InlineItem::Code("zenthree".to_string()),
        ];
        let tokens = tokenize_inlines(&inlines);
        assert_eq!(tokens[0].text, "Ecosystem:");
        assert_eq!(tokens[0].style, SpanStyle::Bold);
        assert_eq!(tokens[1].text, "Rust");
        assert_eq!(tokens[1].style, SpanStyle::Normal);
        assert_eq!(tokens[4].text, "zenthree");
        assert_eq!(tokens[4].style, SpanStyle::Code);
    }
}
