use std::{collections::HashMap, error::Error};

use anyhow::Result;
use base64::prelude::*;
use color::{mc_formatting_colors_by_name, to_colored_string};
use colored::{ColoredString, Colorize};
use gamedig::{
    minecraft::{self, BedrockResponse, JavaResponse},
    protocols::types::CommonResponse,
};
use image::{imageops::FilterType, load_from_memory, GenericImageView as _, Rgba};
use serde_json::{from_str, to_string, Map, Value};
use unicode_width::UnicodeWidthStr;

use crate::{color, JavaDescription};

pub fn print_java_motd(java_resp: JavaResponse) {
    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "{} {} {} {}",
        output_field_format("Java版").bright_green(),
        "|".bright_cyan().bold(),
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
        "{} {} {} / {}",
        output_field_format("在线玩家").bright_cyan(),
        "|".bright_cyan().bold(),
        java_resp.players_online,
        java_resp.players_maximum
    ));

    if let Some(map) = java_resp.map() {
        lines.push(format!(
            "{} {} {}",
            output_field_format("地图").bright_cyan(),
            "|".bright_cyan().bold(),
            to_colored_string(map)
        ));
    };
    if let Some(gamemode) = java_resp.game_mode() {
        lines.push(format!(
            "{} {} {}",
            output_field_format("游戏模式").bright_cyan(),
            "|".bright_cyan().bold(),
            gamemode
        ));
    };
    if let Some(players) = java_resp.players {
        if !players.is_empty() {
            for (i, player) in players.iter().enumerate() {
                if i == 0 {
                    lines.push(format!(
                        "{} {} {}",
                        output_field_format("玩家列表").bright_cyan(),
                        "|".bright_cyan().bold(),
                        to_colored_string(&player.name)
                    ));
                } else {
                    lines.push(format!(
                        "{} {} {}",
                        output_field_format("").bright_cyan(),
                        "|".bright_cyan().bold(),
                        to_colored_string(&player.name)
                    ));
                }
            }
        }
    };
    let lines_len = lines.len();
    for line in lines {
        println!("{}", line);
    }
    if let Some(favicon) = java_resp.favicon {
        let favicon = favicon.replace("data:image/png;base64,", "");
        match BASE64_STANDARD.decode(favicon) {
            Ok(image) => {
                let size = match calc_image_size((13, lines_len as u16 + 1)) {
                    Ok(size) => size,
                    Err(_) => {
                        println!(
                            "{} {} {}",
                            output_field_format("图标").bright_cyan(),
                            "|".bright_cyan().bold(),
                            "请调大控制台窗口的大小".bright_red().bold()
                        );
                        return;
                    }
                };
                match img2lines(&image, size as u32) {
                    Ok(lines) => {
                        println!(
                            "{} {}",
                            output_field_format("").bright_cyan(),
                            "|".bright_cyan().bold(),
                        );
                        for (index, line) in lines.into_iter().enumerate() {
                            if index == 0 {
                                println!(
                                    "{} {} {}",
                                    output_field_format("图标").bright_cyan(),
                                    "|".bright_cyan().bold(),
                                    line
                                );
                            } else {
                                println!(
                                    "{} {} {}",
                                    output_field_format("").bright_cyan(),
                                    "|".bright_cyan().bold(),
                                    line
                                )
                            };
                        }
                    }
                    Err(_) => {
                        println!(
                            "{} {} {}",
                            output_field_format("图标").bright_cyan(),
                            "|".bright_cyan().bold(),
                            "图片输出失败".bright_red().bold()
                        );
                    }
                };
            }
            Err(_) => {
                println!(
                    "{} {} {}",
                    output_field_format("图标").bright_cyan(),
                    "|".bright_cyan().bold(),
                    "图片解码失败".bright_red().bold()
                );
            }
        }
    }
    println!();
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

fn print_java_motd_extra_process_child(extras: &mut [Value]) {
    for extras_ch in extras.iter_mut() {
        if extras_ch.is_string() {
            let mut new_map = Map::new();
            new_map.insert(
                "text".to_string(),
                Value::String(extras_ch.as_str().unwrap().to_string()),
            );
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

pub fn print_bedrock_motd(bedrock_resp: BedrockResponse) {
    println!(
        "{} {} {} {}",
        output_field_format("基岩版").bright_green(),
        "|".bright_cyan().bold(),
        bedrock_resp.version_name.bright_yellow(),
        format!("({})", bedrock_resp.protocol_version).cyan()
    );
    println!(
        "{} {} {}",
        output_field_format("Motd").bright_cyan(),
        "|".bright_cyan().bold(),
        to_colored_string(&bedrock_resp.name)
    );
    println!(
        "{} {} {} / {}",
        output_field_format("在线玩家").bright_cyan(),
        "|".bright_cyan().bold(),
        bedrock_resp.players_online,
        bedrock_resp.players_maximum
    );
    if let Some(map) = bedrock_resp.map.clone() {
        println!(
            "{} {} {}",
            output_field_format("地图").bright_cyan(),
            "|".bright_cyan().bold(),
            to_colored_string(&map)
        );
    };
    if let Some(gamemode) = bedrock_resp.game_mode {
        println!(
            "{} {} {}",
            output_field_format("游戏模式").bright_cyan(),
            "|".bright_cyan().bold(),
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
        if !players.is_empty() {
            println!(
                "{} {} {}",
                output_field_format("玩家列表").bright_cyan(),
                "|".bright_cyan().bold(),
                players[0]
            );
            if players.len() > 1 {
                for player in &players[1..] {
                    println!(
                        "{} {} {}",
                        output_field_format("").bright_cyan(),
                        "|".bright_cyan().bold(),
                        to_colored_string(player)
                    );
                }
            }
        }
    };
    println!();
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

pub fn img2lines(buffer: &[u8], size: u32) -> Result<Vec<String>, Box<dyn Error>> {
    let image = load_from_memory(buffer)?.resize(size, size, FilterType::CatmullRom);
    let pixels = image.pixels().collect::<Vec<_>>();
    let mut pixels_2d: Vec<Vec<Rgba<u8>>> = Vec::new();
    for pixel in pixels {
        let (x, y) = (pixel.0, pixel.1);
        if x == 0 {
            pixels_2d.push(Vec::new());
        };
        pixels_2d.last_mut().unwrap().push(image.get_pixel(x, y));
    }
    let pixel_2d_pairs: Vec<(Vec<Rgba<u8>>, Option<Vec<Rgba<u8>>>)> = pixels_2d
        .chunks(2)
        .map(|chunk| {
            let row1 = chunk[0].clone();
            let row2 = if chunk.len() > 1 {
                Some(chunk[1].clone())
            } else {
                None
            };
            (row1, row2)
        })
        .collect();

    let mut lines: Vec<String> = Vec::new();
    for (row1, row2) in pixel_2d_pairs {
        let mut line = String::new();
        if let Some(row2) = row2 {
            for i in 0..row1.len() - 1 {
                let block: ColoredString = "▀"
                    .truecolor(row1[i][0], row1[i][1], row1[i][2])
                    .on_truecolor(row2[i][0], row2[i][1], row2[i][2]);
                line = format!("{}{}", line, block);
            }
        }
        lines.push(line);
    }
    Ok(lines)
}

fn calc_image_size(base: (u16, u16)) -> Result<usize, Box<dyn Error>> {
    let term_size = crossterm::terminal::size().unwrap_or((80, 24));
    if term_size.0 <= base.0 || term_size.1 <= base.1 {
        return Err("控制台过小，请调大控制台窗口的大小".into());
    }
    let x_max = term_size.0 - base.0;
    let y_max = term_size.1 - base.1;
    if x_max < y_max {
        if x_max < 13 {
            return Err("控制台过小，请调大控制台窗口的大小".into());
        }
        Ok(x_max as usize)
    } else {
        Ok(if (y_max * 2 - 2) > 64 {
            64
        } else {
            y_max * 2 - 2
        } as usize)
    }
}
