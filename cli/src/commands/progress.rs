use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use std::time::Duration;
use std::thread;

pub fn run_progress() {
    println!("=== Progress Bar Example ===");
    println!();
    
    // Simple progress bar
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
            .unwrap()
            .progress_chars("#>-")
    );
    
    println!("Processing items...");
    for i in 0..100 {
        pb.set_message(format!("Processing item {}", i + 1));
        pb.inc(1);
        thread::sleep(Duration::from_millis(50));
    }
    
    pb.finish_with_message("✅ All items processed!");
    println!();
    
    // Multiple progress bars
    println!("Multiple tasks:");
    let m = MultiProgress::new();
    
    let pb1 = m.add(ProgressBar::new(100));
    pb1.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} Task 1: [{bar:40.cyan/blue}] {pos}/{len}")
            .unwrap()
    );
    
    let pb2 = m.add(ProgressBar::new(100));
    pb2.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.yellow} Task 2: [{bar:40.magenta/red}] {pos}/{len}")
            .unwrap()
    );
    
    // Simulate parallel progress
    for i in 0..100 {
        pb1.inc(1);
        if i % 2 == 0 {
            pb2.inc(2);
        }
        thread::sleep(Duration::from_millis(30));
    }
    
    pb1.finish_with_message("✅ Task 1 complete");
    pb2.finish_with_message("✅ Task 2 complete");
}
