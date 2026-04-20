use crate::t;
use console::style;

pub fn show_banner() {
    show_logo();
    println!();
    println!("  {}", style(t!("banner_tagline")).dim());
    println!();
    println!(
        "  {} x-skill <command> [options]",
        style(t!("banner_usage")).bold()
    );
    println!();
    println!(
        "  {}  {}  x-skill add <source>",
        style("add").green().bold(),
        t!("banner_add")
    );
    println!(
        "  {}  {}  x-skill find [query]",
        style("find").green().bold(),
        t!("banner_find")
    );
    println!(
        "  {}  {}  x-skill list",
        style("list").green().bold(),
        t!("banner_list")
    );
    println!(
        "  {}  {}  x-skill check",
        style("check").green().bold(),
        t!("banner_check")
    );
    println!();
    println!(
        "  {}",
        t!("banner_more_info", "cmd" => style("x-skill --help").bold())
    );
}

pub fn show_logo() {
    let logo = r#"
  ██╗  ██╗      ███████╗██╗  ██╗██╗██╗     ██╗
  ╚██╗██╔╝      ██╔════╝██║ ██╔╝██║██║     ██║
   ╚███╔╝ █████╗███████╗█████╔╝ ██║██║     ██║
   ██╔██╗ ╚════╝╚════██║██╔═██╗ ██║██║     ██║
  ██╔╝ ██╗      ███████║██║  ██╗██║███████╗███████╗
  ╚═╝  ╚═╝      ╚══════╝╚═╝  ╚═╝╚═╝╚══════╝╚══════╝"#;
    println!("{}", style(logo).dim());
}

#[allow(dead_code)]
pub fn strip_logo(output: &str) -> String {
    output
        .lines()
        .filter(|l| !l.contains("██") && !l.contains("╗") && !l.contains("╚"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[allow(dead_code)]
pub fn has_logo(output: &str) -> bool {
    output.contains("███████╗██╗  ██╗██╗██╗")
}
