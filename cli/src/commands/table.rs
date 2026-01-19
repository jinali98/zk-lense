use comfy_table::{Table, presets::UTF8_FULL};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_BORDERS_ONLY;

pub fn run_table() {
    println!("=== Table Example ===");
    println!();
    
    // Create a table with UTF8 borders
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(vec!["Name", "Status", "Progress", "Score"]);
    
    // Add rows
    table.add_row(vec![
        "Task 1",
        "✅ Complete",
        "100%",
        "95"
    ]);
    table.add_row(vec![
        "Task 2",
        "🔄 In Progress",
        "75%",
        "80"
    ]);
    table.add_row(vec![
        "Task 3",
        "⏳ Pending",
        "0%",
        "-"
    ]);
    table.add_row(vec![
        "Task 4",
        "❌ Failed",
        "50%",
        "45"
    ]);
    
    println!("{table}");
    println!();
    
    // Another table example - simpler style
    let mut table2 = Table::new();
    table2
        .load_preset(UTF8_BORDERS_ONLY)
        .set_header(vec!["ID", "Command", "Description"]);
    
    table2.add_row(vec!["1", "zkprof emoji", "Show emoji examples"]);
    table2.add_row(vec!["2", "zkprof loading", "Show loading spinner"]);
    table2.add_row(vec!["3", "zkprof table", "Show table example"]);
    table2.add_row(vec!["4", "zkprof progress", "Show progress bar"]);
    
    println!("{table2}");
}
