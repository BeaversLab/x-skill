use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};
use std::io::{self, Write};

pub struct SearchItem<T: Clone> {
    pub label: String,
    pub value: T,
}

pub struct MultiSelectOptions<T: Clone> {
    pub prompt: String,
    pub items: Vec<SearchItem<T>>,
    pub locked_values: Vec<T>,
    pub locked_labels: Vec<String>,
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

/// Interactive multiselect with substring search filtering.
/// Returns selected values, or Err if cancelled.
pub fn search_multiselect<T: Clone + PartialEq>(
    opts: MultiSelectOptions<T>,
) -> anyhow::Result<Vec<T>> {
    if !atty::is(atty::Stream::Stdin) || std::env::var("CI").is_ok() {
        // Non-interactive: return all items
        let mut result: Vec<T> = opts.locked_values.clone();
        result.extend(opts.items.iter().map(|i| i.value.clone()));
        return Ok(result);
    }

    terminal::enable_raw_mode()?;
    let result = run_interactive(&opts);
    terminal::disable_raw_mode()?;
    // Clear the prompt area
    execute!(io::stdout(), cursor::MoveToColumn(0))?;

    match result {
        Ok(Some(selected)) => {
            let mut values = opts.locked_values.clone();
            values.extend(selected);
            Ok(values)
        }
        Ok(None) => anyhow::bail!("selection cancelled"),
        Err(e) => Err(e),
    }
}

fn run_interactive<T: Clone + PartialEq>(
    opts: &MultiSelectOptions<T>,
) -> anyhow::Result<Option<Vec<T>>> {
    let mut query = String::new();
    let mut cursor_pos: usize = 0;
    let mut selected: Vec<bool> = vec![false; opts.items.len()];
    let mut scroll_offset: usize = 0;

    loop {
        // Filter items by query
        let filtered_indices: Vec<usize> = opts
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if query.is_empty() {
                    return true;
                }
                let q = query.to_lowercase();
                item.label.to_lowercase().contains(&q)
            })
            .map(|(i, _)| i)
            .collect();

        // Clamp cursor
        if !filtered_indices.is_empty() {
            cursor_pos = cursor_pos.min(filtered_indices.len() - 1);
        }

        // Clamp scroll
        if cursor_pos >= scroll_offset + opts.max_visible {
            scroll_offset = cursor_pos + 1 - opts.max_visible;
        }
        if cursor_pos < scroll_offset {
            scroll_offset = cursor_pos;
        }

        // Render
        render(opts, &filtered_indices, &selected, &query, cursor_pos, scroll_offset)?;

        // Handle input
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(KeyEvent {
                code, modifiers, ..
            }) = event::read()?
            {
                match code {
                    KeyCode::Esc => {
                        clear_render(opts, &filtered_indices, scroll_offset)?;
                        return Ok(None);
                    }
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        clear_render(opts, &filtered_indices, scroll_offset)?;
                        return Ok(None);
                    }
                    KeyCode::Enter => {
                        clear_render(opts, &filtered_indices, scroll_offset)?;
                        let result: Vec<T> = selected
                            .iter()
                            .enumerate()
                            .filter(|(_, &s)| s)
                            .map(|(i, _)| opts.items[i].value.clone())
                            .collect();
                        return Ok(Some(result));
                    }
                    KeyCode::Up => {
                        cursor_pos = cursor_pos.saturating_sub(1);
                    }
                    KeyCode::Down => {
                        if !filtered_indices.is_empty() && cursor_pos < filtered_indices.len() - 1 {
                            cursor_pos += 1;
                        }
                    }
                    KeyCode::Char(' ') => {
                        if let Some(&idx) = filtered_indices.get(cursor_pos) {
                            selected[idx] = !selected[idx];
                        }
                    }
                    KeyCode::Backspace => {
                        query.pop();
                        cursor_pos = 0;
                        scroll_offset = 0;
                    }
                    KeyCode::Char(c) if !modifiers.contains(KeyModifiers::CONTROL) => {
                        query.push(c);
                        cursor_pos = 0;
                        scroll_offset = 0;
                    }
                    _ => {}
                }
            }
        }
    }
}

fn render<T: Clone>(
    opts: &MultiSelectOptions<T>,
    filtered: &[usize],
    selected: &[bool],
    query: &str,
    cursor_pos: usize,
    scroll_offset: usize,
) -> io::Result<()> {
    let mut out = io::stdout();

    // Move to start and clear
    execute!(out, cursor::Hide)?;

    let mut lines = Vec::new();

    // Prompt + search
    let search_line = if query.is_empty() {
        format!("  {} (type to filter)", opts.prompt)
    } else {
        format!("  {} > {}", opts.prompt, query)
    };
    lines.push(search_line);

    // Locked items
    for label in &opts.locked_labels {
        lines.push(format!("  ◉ {} (always included)", label));
    }

    // Visible filtered items
    let visible_end = (scroll_offset + opts.max_visible).min(filtered.len());
    for (vi, &idx) in filtered[scroll_offset..visible_end].iter().enumerate() {
        let actual_pos = scroll_offset + vi;
        let prefix = if actual_pos == cursor_pos { "❯" } else { " " };
        let check = if selected[idx] { "◉" } else { "○" };
        lines.push(format!("{prefix} {check} {}", opts.items[idx].label));
    }

    if filtered.is_empty() {
        lines.push("  (no matches)".into());
    }

    let total = lines.len();
    for line in &lines {
        execute!(out, terminal::Clear(ClearType::CurrentLine))?;
        write!(out, "\r{line}")?;
        execute!(out, cursor::MoveDown(1))?;
    }
    // Move back up
    execute!(out, cursor::MoveUp(total as u16))?;
    execute!(out, cursor::Show)?;
    out.flush()?;
    Ok(())
}

fn clear_render<T: Clone>(
    opts: &MultiSelectOptions<T>,
    filtered: &[usize],
    scroll_offset: usize,
) -> io::Result<()> {
    let mut out = io::stdout();
    let visible = (filtered.len() - scroll_offset).min(opts.max_visible);
    let total_lines = 1 + opts.locked_labels.len() + visible.max(1);
    for _ in 0..total_lines {
        execute!(out, terminal::Clear(ClearType::CurrentLine))?;
        execute!(out, cursor::MoveDown(1))?;
    }
    execute!(out, cursor::MoveUp(total_lines as u16))?;
    out.flush()?;
    Ok(())
}
