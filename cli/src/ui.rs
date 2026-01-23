//! Unified UI module for consistent CLI styling
//!
//! Provides reusable components for:
//! - Spinners with messages
//! - Interactive selections (replacing Y/N prompts)
//! - Formatted tables
//! - Styled panels (info, success, error, warning)
//! - Multi-step progress tracking
//! - Consistent emoji theme

use comfy_table::{
    Attribute, Cell, Color, ContentArrangement, Table, presets::UTF8_FULL_CONDENSED,
};
use console::{Style, style};
use dialoguer::{Select, theme::ColorfulTheme};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

// ============================================================================
// EMOJI THEME
// ============================================================================

pub mod emoji {
    pub const SUCCESS: &str = "✓";
    pub const ERROR: &str = "✗";
    pub const WARNING: &str = "⚠";
    pub const INFO: &str = "ℹ";
    pub const ROCKET: &str = "🚀";
    pub const PACKAGE: &str = "📦";
    pub const FOLDER: &str = "📁";
    pub const FILE: &str = "📄";
    pub const GEAR: &str = "⚙️";
    pub const GLOBE: &str = "🌐";
    pub const LINK: &str = "🔗";
    pub const CHART: &str = "📊";
    pub const SPARKLES: &str = "✨";
    pub const CHECKMARK: &str = "✅";
    pub const CROSSMARK: &str = "❌";
    pub const LIGHTNING: &str = "⚡";
    pub const SEARCH: &str = "🔍";
    pub const CLOCK: &str = "⏱️";
    pub const MONEY: &str = "💰";
    pub const BULB: &str = "💡";
    pub const PIN: &str = "📌";
    pub const PENDING: &str = "○";
    pub const ACTIVE: &str = "●";
    pub const ARROW_RIGHT: &str = "→";
    pub const TREE_BRANCH: &str = "├──";
    pub const TREE_END: &str = "└──";
}

// ============================================================================
// STYLES
// ============================================================================

pub fn style_success() -> Style {
    Style::new().green().bold()
}

pub fn style_error() -> Style {
    Style::new().red().bold()
}

pub fn style_warning() -> Style {
    Style::new().yellow().bold()
}

pub fn style_info() -> Style {
    Style::new().cyan()
}

pub fn style_dim() -> Style {
    Style::new().dim()
}

pub fn style_bold() -> Style {
    Style::new().bold()
}

pub fn style_header() -> Style {
    Style::new().bold().cyan()
}

// ============================================================================
// SPINNERS
// ============================================================================

/// Create a spinner with a message
pub fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏")
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Finish a spinner with a success message
pub fn spinner_success(pb: &ProgressBar, message: &str) {
    pb.set_style(ProgressStyle::default_spinner().template("{msg}").unwrap());
    pb.finish_with_message(format!(
        "{} {}",
        style(emoji::SUCCESS).green().bold(),
        message
    ));
}

/// Finish a spinner with a success message and duration
pub fn spinner_success_with_duration(pb: &ProgressBar, message: &str, duration_ms: u128) {
    pb.set_style(ProgressStyle::default_spinner().template("{msg}").unwrap());
    pb.finish_with_message(format!(
        "{} {} {}",
        style(emoji::SUCCESS).green().bold(),
        message,
        style(format!("({}ms)", duration_ms)).dim()
    ));
}

/// Finish a spinner with an error message
pub fn spinner_error(pb: &ProgressBar, message: &str) {
    pb.set_style(ProgressStyle::default_spinner().template("{msg}").unwrap());
    pb.finish_with_message(format!(
        "{} {}",
        style(emoji::ERROR).red().bold(),
        style(message).red()
    ));
}

/// Finish a spinner with a warning message
pub fn spinner_warn(pb: &ProgressBar, message: &str) {
    pb.set_style(ProgressStyle::default_spinner().template("{msg}").unwrap());
    pb.finish_with_message(format!(
        "{} {}",
        style(emoji::WARNING).yellow().bold(),
        style(message).yellow()
    ));
}

// ============================================================================
// INTERACTIVE SELECTIONS
// ============================================================================

/// Yes/No selection (replaces Y/N prompts)
pub fn confirm(prompt: &str) -> std::io::Result<bool> {
    let items = vec!["Yes", "No"];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    Ok(selection == 0)
}

/// Yes/No selection with custom labels
pub fn confirm_custom(prompt: &str, yes_label: &str, no_label: &str) -> std::io::Result<bool> {
    let items = vec![yes_label, no_label];
    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&items)
        .default(0)
        .interact()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;
    Ok(selection == 0)
}

/// Generic selection from a list of options
pub fn select<T: ToString>(prompt: &str, items: &[T], default: usize) -> std::io::Result<usize> {
    let string_items: Vec<String> = items.iter().map(|i| i.to_string()).collect();
    Select::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .items(&string_items)
        .default(default)
        .interact()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
}

// ============================================================================
// PANELS
// ============================================================================

const PANEL_WIDTH: usize = 65;

fn draw_top_border(title: &str) -> String {
    let title_len = console::measure_text_width(title);
    let padding = if title_len + 4 < PANEL_WIDTH {
        PANEL_WIDTH - title_len - 4
    } else {
        0
    };
    println!();
    format!("┌─ {} {}", title, "─".repeat(padding))
}

fn draw_bottom_border() -> String {
    format!("└{}─", "─".repeat(PANEL_WIDTH - 2))
}

fn draw_divider() -> String {
    format!("├{}┤", "─".repeat(PANEL_WIDTH - 2))
}

fn pad_line(content: &str) -> String {
    let content_len = console::measure_text_width(content);
    let padding = if content_len + 4 < PANEL_WIDTH {
        PANEL_WIDTH - content_len - 4
    } else {
        0
    };
    format!("│  {}{}", content, " ".repeat(padding))
}

/// Print a success panel
pub fn panel_success(title: &str, message: &str) {
    let header = format!("{} {}", emoji::CHECKMARK, title);
    println!();
    println!("{}", style(draw_top_border(&header)).green());
    for line in message.lines() {
        println!("{}", style(pad_line(line)).green());
    }
    println!("{}", style(draw_bottom_border()).green());
    println!();
}

/// Print an error panel with optional suggestions
pub fn panel_error(
    title: &str,
    message: &str,
    details: Option<&str>,
    suggestions: Option<&[&str]>,
) {
    let header = format!("{} {}", emoji::CROSSMARK, title);
    println!();
    println!("{}", style(draw_top_border(&header)).red());
    for line in message.lines() {
        println!("{}", style(pad_line(line)).red());
    }

    if let Some(detail) = details {
        println!("{}", style(pad_line("")).red());
        for line in detail.lines() {
            println!("{}", style(pad_line(&format!("Details: {}", line))).red());
        }
    }

    if let Some(tips) = suggestions {
        println!("{}", style(pad_line("")).red());
        println!(
            "{}",
            style(pad_line(&format!("{} Try:", emoji::BULB))).red()
        );
        for tip in tips {
            println!("{}", style(pad_line(&format!("   • {}", tip))).red());
        }
    }

    println!("{}", style(draw_bottom_border()).red());
    println!();
}

/// Print an info panel
pub fn panel_info(title: &str, message: &str) {
    let header = format!("{} {}", emoji::INFO, title);
    println!();
    println!("{}", style(draw_top_border(&header)).cyan());
    for line in message.lines() {
        println!("{}", style(pad_line(line)).cyan());
    }
    println!("{}", style(draw_bottom_border()).cyan());
    println!();
}

/// Print a warning panel
pub fn panel_warning(title: &str, message: &str) {
    let header = format!("{} {}", emoji::WARNING, title);
    println!();
    println!("{}", style(draw_top_border(&header)).yellow());
    for line in message.lines() {
        println!("{}", style(pad_line(line)).yellow());
    }
    println!("{}", style(draw_bottom_border()).yellow());
    println!();
}

/// Print a header panel (used for command headers)
pub fn panel_header(emoji_icon: &str, title: &str, subtitle: Option<&str>) {
    let header = format!("{} {}", emoji_icon, title);
    println!();
    println!("{}", style(draw_top_border(&header)).cyan().bold());
    if let Some(sub) = subtitle {
        println!("{}", style(pad_line(sub)).cyan());
    }
    println!("{}", style(draw_bottom_border()).cyan().bold());
    println!();
}

// ============================================================================
// TABLES
// ============================================================================

/// Create a styled table with headers
pub fn create_table(headers: &[&str]) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);

    let header_cells: Vec<Cell> = headers
        .iter()
        .map(|h| Cell::new(h).add_attribute(Attribute::Bold).fg(Color::Cyan))
        .collect();
    table.set_header(header_cells);

    table
}

/// Create a key-value table (2 columns)
pub fn create_kv_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL_CONDENSED)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

/// Add a row to a key-value table with emoji prefix
pub fn add_kv_row(table: &mut Table, emoji_icon: &str, key: &str, value: &str) {
    table.add_row(vec![
        Cell::new(format!("{} {}", emoji_icon, key)),
        Cell::new(value),
    ]);
}

/// Print a tree-style list
pub fn print_tree(items: &[(&str, &str)]) {
    let len = items.len();
    for (i, (label, value)) in items.iter().enumerate() {
        let prefix = if i == len - 1 {
            emoji::TREE_END
        } else {
            emoji::TREE_BRANCH
        };
        println!("  {} {:<16} {}", prefix, label, style(value).bold());
    }
}

/// Print a tree-style list with status indicators
pub fn print_tree_with_status(items: &[(&str, &str, bool)]) {
    let len = items.len();
    for (i, (label, value, ok)) in items.iter().enumerate() {
        let prefix = if i == len - 1 {
            emoji::TREE_END
        } else {
            emoji::TREE_BRANCH
        };
        let status = if *ok {
            style(emoji::SUCCESS).green().to_string()
        } else {
            style(emoji::ERROR).red().to_string()
        };
        println!(
            "  {} {:<16} {:<20} {}",
            prefix,
            label,
            style(value).bold(),
            status
        );
    }
}

// ============================================================================
// PROGRESS TRACKING
// ============================================================================

/// Progress step state
#[derive(Clone, Copy, PartialEq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Complete,
    Failed,
}

/// A step in a multi-step progress tracker
pub struct ProgressStep {
    pub name: String,
    pub status: StepStatus,
    pub duration_ms: Option<u128>,
}

impl ProgressStep {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: StepStatus::Pending,
            duration_ms: None,
        }
    }
}

/// Print multi-step progress
pub fn print_progress(steps: &[ProgressStep], current_message: Option<&str>) {
    for (i, step) in steps.iter().enumerate() {
        let step_num = i + 1;
        let total = steps.len();

        let (icon, name_style) = match step.status {
            StepStatus::Pending => (
                style(emoji::PENDING).dim().to_string(),
                style(&step.name).dim(),
            ),
            StepStatus::InProgress => (
                style("⠋").cyan().to_string(),
                style(&step.name).cyan().bold(),
            ),
            StepStatus::Complete => (
                style(emoji::SUCCESS).green().to_string(),
                style(&step.name).green(),
            ),
            StepStatus::Failed => (
                style(emoji::ERROR).red().to_string(),
                style(&step.name).red(),
            ),
        };

        let duration = step.duration_ms.map_or(String::new(), |d| {
            format!(" {}", style(format!("({}ms)", d)).dim())
        });

        let message = if step.status == StepStatus::InProgress {
            current_message.map_or(String::new(), |m| format!(" {}", style(m).dim()))
        } else {
            String::new()
        };

        println!(
            "  [{}] {} {}{}{}",
            style(format!("{}/{}", step_num, total)).dim(),
            icon,
            name_style,
            duration,
            message
        );
    }
}

// ============================================================================
// SECTION HEADERS
// ============================================================================

/// Print a section header with emoji
pub fn section(emoji_icon: &str, title: &str) {
    println!();
    println!("  {} {}", emoji_icon, style(title).bold());
}

/// Print a divider line
pub fn divider() {
    println!("{}", style("─".repeat(65)).dim());
}

/// Print a blank line
pub fn blank() {
    println!();
}

// ============================================================================
// FORMATTED OUTPUT
// ============================================================================

/// Print a labeled value
pub fn print_value(label: &str, value: &str) {
    println!(
        "  {} {}",
        style(format!("{}:", label)).dim(),
        style(value).bold()
    );
}

/// Print a labeled value with emoji
pub fn print_value_with_emoji(emoji_icon: &str, label: &str, value: &str) {
    println!(
        "  {} {} {}",
        emoji_icon,
        style(format!("{}:", label)).dim(),
        style(value).bold()
    );
}

/// Print a success message
pub fn success(message: &str) {
    println!("{} {}", style(emoji::SUCCESS).green().bold(), message);
}

/// Print an error message
pub fn error(message: &str) {
    eprintln!(
        "{} {}",
        style(emoji::ERROR).red().bold(),
        style(message).red()
    );
}

/// Print a warning message
pub fn warn(message: &str) {
    println!(
        "{} {}",
        style(emoji::WARNING).yellow().bold(),
        style(message).yellow()
    );
}

/// Print an info message
pub fn info(message: &str) {
    println!("{} {}", style(emoji::INFO).cyan(), message);
}
