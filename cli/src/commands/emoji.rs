pub fn run_emoji() {
    println!("=== Emoji Examples ===");
    println!();
    
    // Direct Unicode emojis
    println!("✅ Success!");
    println!("❌ Error!");
    println!("⚠️  Warning!");
    println!("ℹ️  Info");
    println!("🚀 Rocket");
    println!("⚡ Fast");
    println!("🔥 Hot");
    println!("💡 Idea");
    println!("🎉 Celebration");
    println!("📊 Chart");
    println!("🔍 Search");
    println!("⚙️  Settings");
    println!();
    
    // Using emojis in formatted strings
    let status = "completed";
    println!("Status: ✅ {}", status);
    
    let count = 42;
    println!("Count: 📦 {}", count);
    
    // Emoji arrays
    let emojis = ["🎯", "🎨", "🎪", "🎭", "🎬"];
    println!("Emojis: {}", emojis.join(" "));
}
