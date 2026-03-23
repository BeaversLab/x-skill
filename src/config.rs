use std::io::IsTerminal;
use std::path::PathBuf;

const LANGUAGE_KEY: &str = "LANGUAGE";

/// Load environment variables from `~/.x-skill/.env`.
/// Existing env vars are NOT overridden, giving us the priority:
///   environment variable > .env value > code default
pub fn load_dotenv() {
    let path = dotenv_path();
    let _ = dotenvy::from_path(&path);
}

/// Prompt the user to choose a language if not already configured.
/// Shows the logo first, then the selection prompt.
/// Writes the selection to `~/.x-skill/.env` so it persists.
pub fn ensure_language() {
    if std::env::var(LANGUAGE_KEY).is_ok() {
        return;
    }

    if !std::io::stdin().is_terminal() || std::env::var("CI").is_ok() {
        std::env::set_var(LANGUAGE_KEY, "en");
        return;
    }

    crate::output::show_logo();
    println!();

    let selected: Result<&str, _> = cliclack::select("Language / 语言")
        .item("en", "English", "")
        .item("zh", "中文", "")
        .interact();

    let lang = match selected {
        Ok(v) => v,
        Err(_) => {
            std::env::set_var(LANGUAGE_KEY, "en");
            return;
        }
    };

    std::env::set_var(LANGUAGE_KEY, lang);
    let _ = write_dotenv(LANGUAGE_KEY, lang);
}

pub fn dotenv_path() -> PathBuf {
    dotenv_dir().join(".env")
}

fn dotenv_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/"))
        .join(".x-skill")
}

/// Write or update a key=value pair in `~/.x-skill/.env`.
pub fn write_dotenv(key: &str, value: &str) -> std::io::Result<()> {
    let dir = dotenv_dir();
    std::fs::create_dir_all(&dir)?;

    let path = dir.join(".env");
    let content = std::fs::read_to_string(&path).unwrap_or_default();

    let mut found = false;
    let mut lines: Vec<String> = content
        .lines()
        .map(|line| {
            if line.starts_with(&format!("{key}=")) {
                found = true;
                format!("{key}={value}")
            } else {
                line.to_string()
            }
        })
        .collect();

    if !found {
        lines.push(format!("{key}={value}"));
    }

    let mut output = lines.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }

    std::fs::write(&path, output)
}
