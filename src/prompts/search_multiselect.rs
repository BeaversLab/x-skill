use crate::t;
use std::io::IsTerminal;

pub struct SearchItem<T: Clone> {
    pub label: String,
    pub value: T,
}

pub struct MultiSelectOptions<T: Clone> {
    pub prompt: String,
    pub items: Vec<SearchItem<T>>,
    pub locked_values: Vec<T>,
    pub locked_labels: Vec<String>,
    #[allow(dead_code)]
    pub max_visible: usize,
}

impl<T: Clone> Default for MultiSelectOptions<T> {
    fn default() -> Self {
        Self {
            prompt: String::new(),
            items: Vec::new(),
            locked_values: Vec::new(),
            locked_labels: Vec::new(),
            max_visible: 15,
        }
    }
}

/// Interactive multiselect using cliclack.
/// Returns selected values (including locked ones), or Err if cancelled.
pub fn search_multiselect<T: Clone + Eq + 'static>(
    opts: MultiSelectOptions<T>,
) -> anyhow::Result<Vec<T>> {
    if !std::io::stdin().is_terminal() || std::env::var("CI").is_ok() {
        let mut result: Vec<T> = opts.locked_values.clone();
        result.extend(opts.items.iter().map(|i| i.value.clone()));
        return Ok(result);
    }

    if !opts.locked_labels.is_empty() {
        let locked_list = opts.locked_labels.join(", ");
        cliclack::log::info(t!("always_included", "list" => locked_list))?;
    }

    if opts.items.is_empty() {
        return Ok(opts.locked_values);
    }

    let mut prompt = cliclack::multiselect(&opts.prompt);
    for item in &opts.items {
        prompt = prompt.item(item.value.clone(), &item.label, "");
    }
    prompt = prompt.required(false);

    let selected: Vec<T> = prompt.interact()?;

    let mut result = opts.locked_values;
    result.extend(selected);
    Ok(result)
}
