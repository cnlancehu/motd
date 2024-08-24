mod cli;
use crate::cli::parse;

use base64::prelude::*;
use colored::{ColoredString, Colorize};
use gamedig::{
    minecraft::{self, BedrockResponse, JavaResponse},
    protocols::types::CommonResponse,
};
use regex::Regex;
use serde::Deserialize;
use serde_json::{from_str, to_string, Map, Value};
use unicode_width::UnicodeWidthStr;
use viuer::{print, terminal_size, Config};

use std::net::ToSocketAddrs;
use std::{collections::HashMap, error::Error};
use std::{net::IpAddr, process::exit, thread, time::Duration};

#[derive(Debug, Deserialize)]
struct JavaDescription {
    extra: Option<Vec<JavaDescription>>,
    text: String,
    color: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
}

fn main() {
    #[cfg(windows)]
    colored::control::set_virtual_terminal(true).unwrap();

    let addr = parse();

    let ip: IpAddr = if Regex::new("^(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])\\.(25[0-5]|2[0-4][0-9]|1[0-9]{2}|[1-9]?[0-9])$").unwrap().is_match(&addr.0) {
        addr.0.parse().unwrap_or_else(|_| {
            println!("{}", "Motd 获取失败\n无法解析IP地址".bright_red().bold());
            exit(1);
        })
    } else {
        match format!("{}:1", addr.0).to_socket_addrs() {
            Ok(mut addrs) => addrs.next().unwrap().ip(),
            Err(e) => {
                println!("{}\n{}", "Motd 获取失败".bright_red().bold(), e.to_string().bright_red());
                exit(1);
            }
        }
    };
    let java_req = thread::spawn(move || minecraft::query_java(&ip, addr.1, None));
    let bedrock_req = thread::spawn(move || minecraft::query_bedrock(&ip, addr.1));
    let mut waiting_count: usize = 0;

    while !java_req.is_finished() && !bedrock_req.is_finished() {
        if waiting_count >= 200 {
            println!("{}", "Motd 获取失败\n连接超时".bright_red().bold());
        }
        thread::sleep(Duration::from_millis(100));
        waiting_count += 1;
    }
    if bedrock_req.is_finished() {
        let bedrock_result = bedrock_req.join().unwrap();
        match bedrock_result {
            Ok(bedrock) => {
                print_bedrock_motd(bedrock);
            }
            Err(_) => {
                let mut waiting_count: usize = 0;
                while !java_req.is_finished() {
                    if waiting_count >= 200 {
                        println!("{}", "Motd 获取失败\n连接超时".bright_red().bold());
                    }
                    thread::sleep(Duration::from_millis(100));
                    waiting_count += 1;
                }
                let java_result = java_req.join().unwrap();
                match java_result {
                    Ok(java) => {
                        print_java_motd(java);
                    }
                    Err(_) => {
                        println!("{}", "Motd 获取失败\n连接超时".bright_red().bold());
                    }
                }
            }
        }
    } else if java_req.is_finished() {
        let java_result = java_req.join().unwrap();
        match java_result {
            Ok(java) => {
                print_java_motd(java);
            }
            Err(_) => {
                let mut waiting_count: usize = 0;
                while !bedrock_req.is_finished() {
                    if waiting_count >= 200 {
                        println!("{}", "Motd 获取失败\n连接超时".bright_red().bold());
                    }
                    thread::sleep(Duration::from_millis(100));
                    waiting_count += 1;
                }
                let bedrock_result = bedrock_req.join().unwrap();
                match bedrock_result {
                    Ok(bedrock) => {
                        print_bedrock_motd(bedrock);
                    }
                    Err(_) => {
                        println!("{}", "Motd 获取失败\n连接超时".bright_red().bold());
                    }
                }
            }
        }
    }
}

fn print_java_motd(java_resp: JavaResponse) {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "{} | {} {}",
        output_field_format("Java版").bright_green(),
        if java_resp.game_version.width() < 30 {
            java_resp.game_version.bright_yellow()
        } else {
            format!("{}...", &java_resp.game_version[..30]).bright_yellow()
        },
        format!("({})", java_resp.protocol_version).cyan()
    ));
    match print_java_motd_extra_process(java_resp.description.clone()) {
        Ok(description) => {
            let mut colored_description: ColoredString = String::new().white();
            if let Some(extras) = description.extra {
                let colors = mc_formatting_colors_by_name();
                fn extra_child_process(
                    colored_description: &mut ColoredString,
                    extras: Vec<JavaDescription>,
                    colors: &HashMap<&str, (u8, u8, u8)>,
                ) {
                    for extra in extras {
                        let mut text: ColoredString = extra.text.white();
                        if let Some(color) = extra.color {
                            if let Some((r, g, b)) = colors.get(color.as_str()) {
                                text = text.truecolor(*r, *g, *b);
                            }
                        };
                        if let Some(bold) = extra.bold {
                            if bold {
                                text = text.bold();
                            }
                        };
                        if let Some(italic) = extra.italic {
                            if italic {
                                text = text.italic();
                            }
                        };
                        if let Some(extra) = extra.extra {
                            extra_child_process(&mut text, extra, colors);
                        }
                        *colored_description = format!("{}{}", colored_description, text).into();
                    }
                }
                extra_child_process(&mut colored_description, extras, &colors);
            };
            colored_description = format!(
                "{}{}",
                colored_description,
                to_colored_string(&description.text)
            )
            .into();
            let colored_description: Vec<&str> = colored_description.split("\n").collect();
            for (i, line) in colored_description.iter().enumerate() {
                if i == 0 {
                    lines.push(format!(
                        "{} {} {}",
                        output_field_format("Motd").bright_cyan(),
                        "|".bright_cyan().bold(),
                        line
                    ));
                } else {
                    lines.push(format!(
                        "{} {} {}",
                        output_field_format("").bright_cyan().bold(),
                        "|".bright_cyan().bold(),
                        line
                    ));
                }
            }
        }
        Err(_) => {
            lines.push(format!(
                "{} | {}",
                output_field_format("Motd").bright_cyan(),
                "显示失败".bright_red().bold()
            ));
        }
    };
    lines.push(format!(
        "{} | {} / {}",
        output_field_format("在线玩家").bright_cyan(),
        java_resp.players_online,
        java_resp.players_maximum
    ));

    if let Some(map) = java_resp.map().clone() {
        lines.push(format!(
            "{} | {}",
            output_field_format("地图").bright_cyan(),
            to_colored_string(&map)
        ));
    };
    if let Some(gamemode) = java_resp.game_mode() {
        lines.push(format!(
            "{} | {}",
            output_field_format("游戏模式").bright_cyan(),
            gamemode
        ));
    };
    if let Some(players) = java_resp.players {
        if players.len() > 0 {
            for (i, player) in players.iter().enumerate() {
                if i == 0 {
                    lines.push(format!(
                        "{} {} {}",
                        output_field_format("玩家列表").bright_cyan(),
                        "|".bright_green().bold(),
                        to_colored_string(&player.name)
                    ));
                } else {
                    lines.push(format!(
                        "{} {} {}",
                        output_field_format("").bright_cyan(),
                        "|".bright_green().bold(),
                        to_colored_string(&player.name)
                    ));
                }
            }
        }
    };
    for line in lines {
        println!("{}", line);
    }
    if let Some(favicon) = java_resp.favicon {
        let favicon = favicon.replace("data:image/png;base64,", "");
        match BASE64_STANDARD.decode(favicon) {
            Ok(image) => {
                let image = image::load_from_memory(&image).unwrap();
                let term_size = terminal_size();
                let viuer_config = Config {
                    absolute_offset: true,
                    x: term_size.0 - (term_size.1 as f64 / 3.0 * 4.0).floor() as u16,
                    y: 0,
                    width: Some((term_size.1 as f64 / 3.0 * 4.0).floor() as u32),
                    height: Some((term_size.1 as f64 / 3.0 * 2.0).floor() as u32),
                    transparent: true,
                    restore_cursor: true,
                    ..Default::default()
                };

                match print(&image, &viuer_config) {
                    Ok(_) => {}
                    Err(_) => {
                        println!(
                            "{} {} {}",
                            output_field_format("图标").bright_cyan(),
                            "|".bold(),
                            "图片输出失败".bright_red().bold()
                        );
                    }
                };
            }
            Err(_) => {
                println!(
                    "{} {} {}",
                    output_field_format("图标").bright_cyan(),
                    "|".bold(),
                    "图片解码失败".bright_red().bold()
                );
            }
        }
    }
}

fn print_java_motd_extra_process(json_origin: String) -> Result<JavaDescription, Box<dyn Error>> {
    let mut json: Value = from_str(&json_origin)?;
    if let Some(extra) = json.get_mut("extra") {
        if let Some(extras) = extra.as_array_mut() {
            print_java_motd_extra_process_child(extras);
        }
        return Ok(from_str::<JavaDescription>(&to_string(&json)?)?);
    }

    Ok(from_str::<JavaDescription>(&json_origin)?)
}

fn print_java_motd_extra_process_child(extras: &mut Vec<Value>) {
    for extras_ch in extras.iter_mut() {
        if extras_ch.is_string() {
            let mut new_map = Map::new();
            new_map.insert("text".to_string(), Value::String(extras_ch.as_str().unwrap().to_string()));
            *extras_ch = Value::Object(new_map);
        } else if let Some(extra_map) = extras_ch.as_object_mut() {
            if let Some(nested_extra) = extra_map.get_mut("extra") {
                if let Some(nested_extras) = nested_extra.as_array_mut() {
                    print_java_motd_extra_process_child(nested_extras);
                }
            }
        }
    }
}

fn print_bedrock_motd(bedrock_resp: BedrockResponse) {
    println!(
        "{} | {} {}",
        output_field_format("基岩版").bright_green(),
        bedrock_resp.version_name.bright_yellow(),
        format!("({})", bedrock_resp.protocol_version).cyan()
    );
    println!(
        "{} | {}",
        output_field_format("Motd").bright_cyan(),
        to_colored_string(&bedrock_resp.name)
    );
    println!(
        "{} | {} / {}",
        output_field_format("在线玩家").bright_cyan(),
        bedrock_resp.players_online,
        bedrock_resp.players_maximum
    );
    if let Some(map) = bedrock_resp.map.clone() {
        println!(
            "{} | {}",
            output_field_format("地图").bright_cyan(),
            to_colored_string(&map)
        );
    };
    if let Some(gamemode) = bedrock_resp.game_mode.clone() {
        println!(
            "{} | {}",
            output_field_format("游戏模式").bright_cyan(),
            match gamemode {
                minecraft::GameMode::Survival => "生存",
                minecraft::GameMode::Creative => "创造",
                minecraft::GameMode::Hardcore => "硬核",
                minecraft::GameMode::Spectator => "旁观",
                minecraft::GameMode::Adventure => "冒险",
            }
        );
    };
    if let Some(players) = bedrock_resp.players().map(|p| {
        p.iter()
            .map(|player| player.name().to_string())
            .collect::<Vec<String>>()
    }) {
        if players.len() > 0 {
            println!(
                "{} | {}",
                output_field_format("玩家列表").bright_cyan(),
                players[0]
            );
            if players.len() > 1 {
                for player in &players[1..] {
                    println!(
                        "{} {} {}",
                        output_field_format("").bright_cyan(),
                        "|".bright_green().bold(),
                        to_colored_string(player)
                    );
                }
            }
        }
    };
}

fn output_field_format(field: &str) -> String {
    format!(
        "{}{}",
        " ".repeat(if UnicodeWidthStr::width(field) < 10 {
            10 - UnicodeWidthStr::width(field)
        } else {
            0
        }),
        field
    )
}

fn to_colored_string(text: &str) -> ColoredString {
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

fn mc_formatting_colors_by_ss() -> HashMap<char, (u8, u8, u8)> {
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

fn mc_formatting_colors_by_name() -> HashMap<&'static str, (u8, u8, u8)> {
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

#[derive(Debug, Clone, Copy, PartialEq)]
enum MCFontFormattingStyle {
    Obfuscated,
    Bold,
    Strikethrough,
    Underline,
    Italic,
    Clear,
}

fn mc_formatting_styles() -> HashMap<char, MCFontFormattingStyle> {
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

fn ss() -> char {
    '§'
}
