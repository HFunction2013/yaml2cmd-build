use std::fmt::Write;
use std::fs;
use std::path::Path;

#[derive(serde::Deserialize, Debug)]
struct Config {
    commands: Vec<CommandDef>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct CommandDef {
    name: String,
    about: String,
    args: Option<Vec<ArgDef>>,
    hidden: Option<bool>,
    subcommands: Option<Vec<Self>>,
    multiple_values: Option<bool>,

    #[cfg(not(debug_assertions))]
    #[serde(default)]
    debug_only: bool,

    aliases: Option<Vec<String>>,
}

#[derive(serde::Deserialize, Debug, Clone)]
struct ArgDef {
    name: String,
    short: Option<char>,
    long: Option<String>,
    help: Option<String>,
    value_name: Option<String>,
    required: Option<bool>,
    num_args: Option<String>,
    action: Option<String>,
    default_value: Option<String>,
    conflicts_with: Option<String>,
    value_parser: Option<String>,
}

/// 映射action字符串到clap枚举
fn map_action(s: &str) -> &'static str {
    match s {
        "set_true" => "clap::ArgAction::SetTrue",
        "append" => "clap::ArgAction::Append",
        "set_false" => "clap::ArgAction::SetFalse",
        "count" => "clap::ArgAction::Count",
        _ => "clap::ArgAction::Set",
    }
}

/// 解析 `num_args` 范围字符串，转为rust合法代码
fn parse_num_args(s: &str) -> String {
    match s {
        "0.." => "0..".to_string(),
        "1.." => "1..".to_string(),
        "0..1" => "0..1".to_string(),
        "0..=" => "0..=".to_string(),
        val if val.parse::<usize>().is_ok() => val.to_string(),
        _ => panic!("invalid num_args range: {s}"),
    }
}

/// 转义字符串
fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// 递归构建Command代码字符串
fn build_command(c: &CommandDef) -> String {
    let mut code = String::new();
    let about = escape_str(&c.about);
    write!(code, "Command::new(\"{}\").about(\"{}\")", escape_str(&c.name), about).unwrap();

    if let Some(aliases) = &c.aliases {
        for alias in aliases {
            write!(code, ".alias(\"{}\")", escape_str(alias)).unwrap();
        }
    }

    if c.hidden == Some(true) {
        code.push_str(".hide(true)");
    }

    if c.multiple_values == Some(false)
        && let Some(args) = &c.args
    {
        let mut group =
            format!("clap::ArgGroup::new(\"{}_exclusive\").multiple(false)", escape_str(&c.name));
        for arg in args {
            write!(group, ".arg(\"{}\")", escape_str(&arg.name)).unwrap();
        }
        write!(code, ".group({group})").unwrap();
    }

    if let Some(args) = &c.args {
        for a in args {
            let mut arg = format!("Arg::new(\"{}\")", escape_str(&a.name));

            if let Some(c) = a.short {
                write!(arg, ".short('{c}')").unwrap();
            }
            if let Some(l) = &a.long {
                write!(arg, ".long(\"{}\")", escape_str(l)).unwrap();
            }
            if let Some(h) = &a.help {
                write!(arg, ".help(\"{}\")", escape_str(h)).unwrap();
            }
            if let Some(vn) = &a.value_name {
                write!(arg, ".value_name(\"{}\")", escape_str(vn)).unwrap();
            }
            if a.required == Some(true) {
                arg.push_str(".required(true)");
            }
            if let Some(n) = &a.num_args {
                let range = parse_num_args(n);
                write!(arg, ".num_args({range})").unwrap();
            }
            if let Some(act) = &a.action {
                write!(arg, ".action({})", map_action(act)).unwrap();
            }
            if let Some(dv) = &a.default_value {
                write!(arg, ".default_value(\"{}\")", escape_str(dv)).unwrap();
            }

            if let Some(vp) = &a.value_parser {
                write!(arg, ".value_parser(clap::value_parser!({vp}))").unwrap();
            }

            if let Some(cf) = &a.conflicts_with {
                write!(arg, ".conflicts_with(\"{}\")", escape_str(cf)).unwrap();
            }

            write!(code, ".arg({arg})").unwrap();
        }
    }

    if let Some(mut subcommands) = c.subcommands.clone() {
        subcommands.sort_by(|a, b| a.name.cmp(&b.name));
        for sub in subcommands {
            let sub_code = build_command(&sub);
            write!(code, ".subcommand({sub_code})").unwrap();
        }
    }

    code
}

/// 从 YAML 生成 clap 命令代码
///
/// # Panics
///
/// 如果 `OUT_DIR` 环境变量未设置，或 YAML 解析失败，或文件写入失败。
pub fn build_with_yaml(yaml: &str) {
    let out_dir = std::env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let dest = Path::new(&out_dir).join("command.rs");
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).expect("Failed create command dir");
    }

    // let yaml_path = Path::new("./commands.yaml");
    // let yaml = fs::read_to_string(yaml_path).expect("Cannot read ./commands.yaml, file missing");
    let config: Config =
        serde_yaml::from_str(yaml).expect("commands.yaml yaml parse failed, check syntax");

    let mut commands = config.commands;

    #[cfg(not(debug_assertions))]
    commands.retain(|c| !c.debug_only);

    commands.sort_by(|a, b| a.name.cmp(&b.name));

    let mut code = String::new();
    code.push_str("use clap::{Arg, Command};\n");
    code.push_str("pub fn add_commands(mut cmd: Command) -> Command {\n");

    for c in commands {
        let cmd_code = build_command(&c);
        writeln!(code, "    cmd = cmd.subcommand({cmd_code});").unwrap();
    }

    code.push_str("    cmd\n");
    code.push_str("}\n");

    let need_write = if dest.exists() {
        fs::read_to_string(&dest).ok().is_none_or(|old| old != code)
    } else {
        true
    };

    if need_write {
        fs::write(&dest, code).expect("Failed write generated cmd code");
    }

    // println!("cargo:rerun-if-changed={}", yaml_path.display());
}
