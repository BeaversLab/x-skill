use crate::output;
use crate::t;

pub fn run() -> anyhow::Result<()> {
    output::show_logo();
    println!();

    let lang: &str = cliclack::select("Language / 语言")
        .item("en", "English", "")
        .item("zh", "中文", "")
        .interact()?;

    std::env::set_var("LANGUAGE", lang);
    crate::config::write_dotenv("LANGUAGE", lang)?;

    cliclack::log::success(t!("language_set", "lang" => lang))?;
    Ok(())
}
