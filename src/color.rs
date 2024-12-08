use colored::{ColoredString, Colorize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MCFontFormattingStyle {
    Obfuscated,
    Bold,
    Strikethrough,
    Underline,
    Italic,
    Clear,
}

pub fn to_colored_string(text: &str) -> ColoredString {
    let colors = mc_formatting_colors_by_ss();
    let styles = mc_formatting_styles();
    let mut colored_string: ColoredString = "".to_string().white();
    let mut chars = text.chars().peekable();

    let mut current_color: Option<(u8, u8, u8)> = None;
    let mut current_styles: Vec<MCFontFormattingStyle> = Vec::new();
    let mut buffer = String::new();

    while let Some(c) = chars.next() {
        if c == ss() {
            if let Some(&next_char) = chars.peek() {
                if colors.contains_key(&next_char) || styles.contains_key(&next_char) {
                    if !buffer.is_empty() {
                        let color = current_color.unwrap_or((255, 255, 255));
                        let mut colored_text = buffer.truecolor(color.0, color.1, color.2);

                        for style in &current_styles {
                            match style {
                                MCFontFormattingStyle::Bold => colored_text = colored_text.bold(),
                                MCFontFormattingStyle::Italic => {
                                    colored_text = colored_text.italic()
                                }
                                MCFontFormattingStyle::Underline => {
                                    colored_text = colored_text.underline()
                                }
                                MCFontFormattingStyle::Strikethrough => {
                                    colored_text = colored_text.strikethrough()
                                }
                                MCFontFormattingStyle::Obfuscated => {
                                    colored_text = colored_text.dimmed()
                                }
                                MCFontFormattingStyle::Clear => {
                                    colored_text = colored_text.normal()
                                }
                            }
                        }

                        colored_string = format!("{}{}", colored_string, colored_text).into();
                        buffer.clear();
                    }

                    if let Some(&color) = colors.get(&next_char) {
                        current_color = Some(color);
                    } else if let Some(&style) = styles.get(&next_char) {
                        if style == MCFontFormattingStyle::Clear {
                            current_styles.clear();
                            current_color = None;
                        } else {
                            current_styles.push(style);
                        }
                    }

                    chars.next();
                } else {
                    buffer.push(c);
                    buffer.push(next_char);
                    chars.next();
                }
            } else {
                buffer.push(c);
            }
        } else {
            buffer.push(c);
        }
    }

    if !buffer.is_empty() {
        let color = current_color.unwrap_or((255, 255, 255));
        let mut colored_text = buffer.truecolor(color.0, color.1, color.2);

        // 应用所有的样式
        for style in &current_styles {
            match style {
                MCFontFormattingStyle::Bold => colored_text = colored_text.bold(),
                MCFontFormattingStyle::Italic => colored_text = colored_text.italic(),
                MCFontFormattingStyle::Underline => colored_text = colored_text.underline(),
                MCFontFormattingStyle::Strikethrough => colored_text = colored_text.strikethrough(),
                MCFontFormattingStyle::Obfuscated => colored_text = colored_text.dimmed(),
                MCFontFormattingStyle::Clear => colored_text = colored_text.normal(),
            }
        }

        colored_string = format!("{}{}", colored_string, colored_text).into();
    }

    colored_string
}

pub fn mc_formatting_colors_by_ss() -> HashMap<char, (u8, u8, u8)> {
    [
        ('0', (0, 0, 0)),
        ('1', (0, 0, 170)),
        ('2', (0, 170, 0)),
        ('3', (0, 170, 170)),
        ('4', (170, 0, 0)),
        ('5', (170, 0, 170)),
        ('6', (255, 170, 0)),
        ('7', (170, 170, 170)),
        ('8', (85, 85, 85)),
        ('9', (85, 85, 255)),
        ('a', (85, 255, 85)),
        ('b', (85, 255, 255)),
        ('c', (255, 85, 85)),
        ('d', (255, 85, 255)),
        ('e', (255, 255, 85)),
        ('f', (255, 255, 255)),
        ('g', (221, 214, 5)),
        ('h', (227, 212, 209)),
        ('i', (206, 202, 202)),
        ('j', (68, 58, 59)),
        ('m', (151, 22, 7)),
        ('n', (180, 104, 77)),
        ('p', (222, 177, 45)),
        ('q', (17, 160, 54)),
        ('s', (44, 186, 168)),
        ('t', (33, 73, 123)),
        ('u', (154, 92, 198)),
    ]
    .iter()
    .cloned()
    .collect()
}

pub fn mc_formatting_colors_by_name() -> HashMap<&'static str, (u8, u8, u8)> {
    [
        ("black", (0, 0, 0)),
        ("dark_blue", (0, 0, 170)),
        ("dark_green", (0, 170, 0)),
        ("dark_aqua", (0, 170, 170)),
        ("dark_red", (170, 0, 0)),
        ("dark_purple", (170, 0, 170)),
        ("gold", (255, 170, 0)),
        ("gray", (170, 170, 170)),
        ("dark_gray", (85, 85, 85)),
        ("blue", (85, 85, 255)),
        ("green", (85, 255, 85)),
        ("aqua", (85, 255, 255)),
        ("red", (255, 85, 85)),
        ("light_purple", (255, 85, 255)),
        ("yellow", (255, 255, 85)),
        ("white", (255, 255, 255)),
        ("minecoin_gold", (221, 214, 5)),
        ("material_quartz", (227, 212, 209)),
        ("material_iron", (206, 202, 202)),
        ("material_netherite", (68, 58, 59)),
        ("material_redstone", (151, 22, 7)),
        ("material_copper", (180, 104, 77)),
        ("material_gold", (222, 177, 45)),
        ("material_emerald", (17, 160, 54)),
        ("material_diamond", (44, 186, 168)),
        ("material_lapis", (33, 73, 123)),
        ("material_amethyst", (154, 92, 198)),
    ]
    .iter()
    .cloned()
    .collect()
}

pub fn mc_formatting_styles() -> HashMap<char, MCFontFormattingStyle> {
    [
        ('k', MCFontFormattingStyle::Obfuscated),
        ('l', MCFontFormattingStyle::Bold),
        ('m', MCFontFormattingStyle::Strikethrough),
        ('n', MCFontFormattingStyle::Underline),
        ('o', MCFontFormattingStyle::Italic),
        ('r', MCFontFormattingStyle::Clear),
    ]
    .into_iter()
    .collect()
}

pub fn ss() -> char {
    '§'
}
