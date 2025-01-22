mod cli;
mod color;
mod error;
mod net_resolver;
mod print;

use std::thread;

use colored::Colorize as _;
use gamedig::minecraft::{self, RequestSettings};
use print::{print_bedrock_motd, print_java_motd};
use serde::Deserialize;

use crate::{cli::parse, net_resolver::resolve_ip_or_domain};

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

    let ip = match resolve_ip_or_domain(&addr.0, &mut None) {
        Ok(ip) => ip,
        Err(_) => {
            println!("{}", "Motd 获取失败\n无法解析域名".bright_red().bold());
            return;
        }
    };
    let java_req = thread::spawn(move || {
        let result = minecraft::query_java(
            &ip,
            addr.1,
            Some(RequestSettings {
                hostname: addr.0,
                protocol_version: 763, // Java Edition 1.20
            }),
        );
        match result {
            Ok(response) => {
                print_java_motd(response);
                Ok(())
            }
            Err(e) => Err(e),
        }
    });
    let bedrock_req = thread::spawn(move || {
        let result = minecraft::query_bedrock(&ip, addr.1);
        match result {
            Ok(response) => {
                print_bedrock_motd(response);
                Ok(())
            }
            Err(e) => Err(e),
        }
    });

    let java_result = java_req.join().unwrap();
    let bedrock_result = bedrock_req.join().unwrap();
    if let (Err(_), Err(_)) = (java_result, bedrock_result) {
        println!("{}", "Motd 获取失败".bright_red().bold());
    }
}
