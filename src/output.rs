use colored::Colorize;

pub fn show_banner() {
    show_logo();
    println!();
    println!("  {}", "The open agent skills ecosystem".dimmed());
    println!();
    println!("  {} x-skill <command> [options]", "Usage:".bold());
    println!();
    println!("  {}  Install skills     x-skill add <source>", "add".green().bold());
    println!(
        "  {}  Search for skills  x-skill find [query]",
        "find".green().bold()
    );
    println!(
        "  {}  List installed     x-skill list",
        "list".green().bold()
    );
    println!(
        "  {}  Check updates      x-skill check",
        "check".green().bold()
    );
    println!();
    println!("  Run {} for more info", "x-skill --help".bold());
}

pub fn show_logo() {
    // Simplified ASCII logo; the full 256-color version is added in Phase 12
    let logo = r#"
  ██╗  ██╗      ███████╗██╗  ██╗██╗██╗     ██╗
  ╚██╗██╔╝      ██╔════╝██║ ██╔╝██║██║     ██║
   ╚███╔╝ █████╗███████╗█████╔╝ ██║██║     ██║
   ██╔██╗ ╚════╝╚════██║██╔═██╗ ██║██║     ██║
  ██╔╝ ██╗      ███████║██║  ██╗██║███████╗███████╗
  ╚═╝  ╚═╝      ╚══════╝╚═╝  ╚═╝╚═╝╚══════╝╚══════╝"#;
    println!("{}", logo.dimmed());
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
