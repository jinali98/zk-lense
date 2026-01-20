use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::thread;
use console::style;

use crate::commands::init::{read_config, DEFAULT_WEB_APP_URL};

pub fn run_view(path: Option<String>) {
    // Determine the project directory
    let project_dir = match path {
        Some(p) => PathBuf::from(p),
        None => match std::env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                eprintln!(
                    "{} Failed to get current directory: {}",
                    style("✖").red().bold(),
                    e
                );
                eprintln!(
                    "  {} Try specifying a path: zkprof view /path/to/project",
                    style("→").dim()
                );
                std::process::exit(1);
            }
        },
    };

    let zkproof_dir = project_dir.join(".zkproof");
    let report_path = zkproof_dir.join("report.json");

    // Check if report.json exists
    if !report_path.exists() {
        eprintln!(
            "{} No report found at {}",
            style("✖").red().bold(),
            style(report_path.display()).yellow()
        );
        eprintln!(
            "  {} Run profiling first to generate a report.",
            style("→").dim()
        );
        std::process::exit(1);
    }

    // Read the report file
    let report_content = match fs::read_to_string(&report_path) {
        Ok(content) => content,
        Err(e) => {
            eprintln!(
                "{} Failed to read report: {}",
                style("✖").red().bold(),
                e
            );
            std::process::exit(1);
        }
    };

    // Validate it's valid JSON
    if serde_json::from_str::<serde_json::Value>(&report_content).is_err() {
        eprintln!(
            "{} Report file is not valid JSON",
            style("✖").red().bold()
        );
        std::process::exit(1);
    }

    // Read web app URL from config, fallback to default
    let web_app_url = match read_config(&project_dir) {
        Ok(config) => config
            .get("web_app_url")
            .cloned()
            .unwrap_or_else(|| DEFAULT_WEB_APP_URL.to_string()),
        Err(_) => DEFAULT_WEB_APP_URL.to_string(),
    };

    // Find an available port
    let listener = match TcpListener::bind("127.0.0.1:0") {
        Ok(l) => l,
        Err(e) => {
            eprintln!(
                "{} Failed to bind to a port: {}",
                style("✖").red().bold(),
                e
            );
            std::process::exit(1);
        }
    };
    let port = listener.local_addr().unwrap().port();

    println!(
        "{} Starting local server on port {}",
        style("◉").cyan().bold(),
        style(port).cyan()
    );

    // Build the web app URL with the port parameter
    let viewer_url = format!("{}?port={}", web_app_url, port);

    println!(
        "{} Opening viewer at {}",
        style("◉").cyan().bold(),
        style(&viewer_url).underlined()
    );

    // Open the browser
    if let Err(e) = webbrowser::open(&viewer_url) {
        eprintln!(
            "{} Failed to open browser: {}",
            style("⚠").yellow().bold(),
            e
        );
        println!(
            "  {} Open this URL manually: {}",
            style("→").dim(),
            viewer_url
        );
    }

    println!(
        "{} Serving report... Press Ctrl+C to stop.",
        style("◉").green().bold()
    );

    // Handle incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let content = report_content.clone();
                thread::spawn(move || {
                    let mut buffer = [0; 1024];
                    if stream.read(&mut buffer).is_err() {
                        return;
                    }

                    let request = String::from_utf8_lossy(&buffer);
                    
                    // Handle CORS preflight
                    if request.starts_with("OPTIONS") {
                        let response = "HTTP/1.1 204 No Content\r\n\
                            Access-Control-Allow-Origin: *\r\n\
                            Access-Control-Allow-Methods: GET, OPTIONS\r\n\
                            Access-Control-Allow-Headers: Content-Type\r\n\
                            \r\n";
                        let _ = stream.write_all(response.as_bytes());
                        return;
                    }

                    // Serve the JSON for any GET request (including /data.json)
                    if request.starts_with("GET") {
                        let response = format!(
                            "HTTP/1.1 200 OK\r\n\
                            Content-Type: application/json\r\n\
                            Access-Control-Allow-Origin: *\r\n\
                            Content-Length: {}\r\n\
                            \r\n\
                            {}",
                            content.len(),
                            content
                        );
                        let _ = stream.write_all(response.as_bytes());
                    }
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
