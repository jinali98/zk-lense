use anyhow::{Context, Result};
use dialoguer::{Confirm, Input, Select};
use std::fs;
use std::path::Path;
use std::process::Command;

/// Template information with embedded content
struct Template {
    display_name: &'static str,
    content: &'static str,
}

/// Available templates (embedded at compile time)
const TEMPLATES: &[Template] = &[
    Template {
        display_name: "Age Verifier - Verify age threshold based on year of birth",
        content: include_str!("../templates/age_verifier.nr"),
    },
    Template {
        display_name: "Merkle Inclusion Proof - Prove membership in a Merkle tree",
        content: include_str!("../templates/merkle_inclusion.nr"),
    },
];

/// Run the generate command
pub fn run_generate(name: Option<String>, template: Option<String>) -> Result<()> {
    // Get project name
    let project_name = match name {
        Some(n) => n,
        None => {
            Input::<String>::new()
                .with_prompt("Enter project name")
                .interact_text()
                .context("Failed to read project name")?
        }
    };
    
    if project_name.is_empty() {
        return Err(anyhow::anyhow!("Project name cannot be empty"));
    }
    
    // Build template selection options
    let mut template_options: Vec<&str> = vec!["None - Start with default Noir template"];
    for t in TEMPLATES {
        template_options.push(t.display_name);
    }
    
    // Get template selection
    let selected_template = match template {
        Some(t) => {
            // Match by template name or display name
            let t_lower = t.to_lowercase();
            if t_lower == "none" {
                None
            } else {
                TEMPLATES.iter().find(|tmpl| {
                    // Extract short name from display name (e.g., "Age Verifier" from "Age Verifier - ...")
                    let short_name = tmpl.display_name.split(" - ").next().unwrap_or("").to_lowercase();
                    let short_name_snake = short_name.replace(' ', "_");
                    t_lower == short_name_snake
                        || short_name.contains(&t_lower)
                        || tmpl.display_name.to_lowercase().contains(&t_lower)
                })
            }
        }
        None => {
            // Interactive selection
            let selection = Select::new()
                .with_prompt("Select a template")
                .items(&template_options)
                .default(0)
                .interact()
                .context("Failed to select template")?;
            
            if selection == 0 {
                None // "None" selected
            } else {
                Some(&TEMPLATES[selection - 1])
            }
        }
    };
    
    // Run nargo new
    println!("\n📦 Creating new Noir project: {}", project_name);
    
    let nargo_output = Command::new("nargo")
        .args(["new", &project_name])
        .output()
        .context("Failed to execute 'nargo new'. Is Nargo installed and in PATH?")?;
    
    if !nargo_output.status.success() {
        let stderr = String::from_utf8_lossy(&nargo_output.stderr);
        let stdout = String::from_utf8_lossy(&nargo_output.stdout);
        return Err(anyhow::anyhow!(
            "nargo new failed:\n{}\n{}",
            stdout,
            stderr
        ));
    }
    
    println!("✅ Created Noir project: {}", project_name);
    
    // Apply template if selected
    if let Some(tmpl) = selected_template {
        let main_nr_path = Path::new(&project_name).join("src").join("main.nr");
        
        println!("📝 Applying template: {}", tmpl.display_name.split(" - ").next().unwrap_or(tmpl.display_name));
        
        fs::write(&main_nr_path, tmpl.content)
            .with_context(|| format!("Failed to write template to: {}", main_nr_path.display()))?;
        
        println!("✅ Applied template to src/main.nr");
    }
    
    println!("\n🎉 Project '{}' created successfully!", project_name);
    
    // Ask if user wants to run zkprof init
    let run_init = Confirm::new()
        .with_prompt("Would you like to initialize zkprof in this project?")
        .default(true)
        .interact()
        .unwrap_or(false);
    
    if run_init {
        let project_path = std::env::current_dir()?.join(&project_name);
        super::run_init(Some(project_path.to_string_lossy().to_string()));
    }
    
    println!("\nNext steps:");
    println!("  cd {}", project_name);
    println!("  nargo check    # Verify the project compiles");
    println!("  nargo prove    # Generate a proof");
    
    Ok(())
}
