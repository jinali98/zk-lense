use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use std::thread;

pub fn run_loading() {
    println!("=== Loading Spinner Example ===");
    println!();
    
    // Create a spinner with a style
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    );
    
    spinner.set_message("Processing...");
    spinner.enable_steady_tick(Duration::from_millis(100));
    
    // Simulate work
    thread::sleep(Duration::from_secs(3));
    
    spinner.finish_with_message("✅ Done!");
    println!();
    
    // Another example with different style
    let spinner2 = ProgressBar::new_spinner();
    spinner2.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    spinner2.set_message("🚀 Loading data...");
    spinner2.enable_steady_tick(Duration::from_millis(50));
    
    thread::sleep(Duration::from_secs(2));
    
    spinner2.finish_with_message("✨ Complete!");
}
