use colored::Colorize;
use unicode_width::UnicodeWidthStr;

use std::{env::args, ffi::OsStr, path::Path, process::exit};

pub fn parse() -> (String, Option<u16>) {
    let args: Vec<String> = args().skip(1).collect();
    match args.len() {
        0 => {
            echo_help();
            exit(0);
        }
        1 => {
            let arg = &args[0];

            if arg.contains(":") {
                let ip_port: Vec<&str> = arg.split(":").collect();
                let ip = ip_port[0].to_string();
                let port = match ip_port[1].parse::<u16>() {
                    Ok(port) => Some(port),
                    Err(_) => {
                        input_error(args, 1, "这是一个不合法的端口号");
                        exit(1);
                    }
                };
                (ip, port)
            } else {
                let ip = arg.to_string();
                (ip, None)
            }
        }
        2 => {
            let ip = args[0].to_string();
            let port = match args[1].parse::<u16>() {
                Ok(port) => Some(port),
                Err(_) => {
                    input_error(args, 2, "这是一个不合法的端口号");
                    exit(1);
                }
            };
            (ip, port)
        }
        _ => {
            println!("{}", "您的输入参数过多".bright_red().bold());
            exit(1);
        }
    }
}

fn input_error(args: Vec<String>, error_arg: usize, error_msg: &str) {
    println!("{}", "您的输入有误".bright_red().bold());
    println!(
        "{} {} {} {} {}",
        "-->".bright_cyan().bold(),
        get_current_exe_file_name().bright_yellow(),
        args[..error_arg - 1].join(" "),
        args[error_arg - 1],
        args[error_arg..].join(" ")
    );
    let mut repeat: usize = 2;
    repeat += UnicodeWidthStr::width(get_current_exe_file_name().as_str()) + 1;
    for i in args.iter().take(error_arg - 1) {
        repeat += UnicodeWidthStr::width(i.as_str()) + 1;
    }
    println!(
        " {}{}{} {}",
        "|".bright_cyan().bold(),
        " ".repeat(repeat),
        "^".repeat(UnicodeWidthStr::width(args[error_arg - 1].as_str()))
            .bright_yellow(),
        error_msg.bright_cyan().bold()
    );
    println!(
        " {}  {}",
        "=".bright_cyan().bold(),
        format!(
            "使用 {} -h 查看帮助",
            get_current_exe_file_name().bright_yellow()
        )
    );
}

fn echo_help() {
    let current_exe_file_name = get_current_exe_file_name();
    println!("Motd {}", env!("CARGO_PKG_VERSION").bright_green().bold());
    println!(
        "{}\n",
        "跨平台的 Minecraft 服务器 Motd 测试工具"
            .bright_cyan()
            .bold()
    );
    println!(
        "   食用方法 | {} <IP> <端口>",
        &current_exe_file_name.bright_yellow()
    );
    println!(
        "            | {} <IP:端口>",
        &current_exe_file_name.bright_yellow()
    );
    println!("            |");
    println!(
        "Github Repo | {}",
        "https://github.com/cnlancehu/motd".bright_cyan()
    );
}

fn get_current_exe_file_name() -> String {
    args()
        .next()
        .and_then(|exe_file_name| {
            Path::new(&exe_file_name)
                .file_name()
                .and_then(OsStr::to_str)
                .map(|s| s.to_string())
        })
        .unwrap_or("motd".to_string())
}
