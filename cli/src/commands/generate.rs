use anyhow::{Context, Result};
use console::style;
use dialoguer::{theme::ColorfulTheme, Input, Select};
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::ui::{self, emoji};

/// Template information with embedded content
struct Template {
    name: &'static str,
    display_name: &'static str,
    content: &'static str,
}

/// Available templates (embedded at compile time)
const TEMPLATES: &[Template] = &[
    Template {
        name: "age_verifier",
        display_name: "Age Verifier - Verify age threshold based on year of birth",
        content: include_str!("../templates/age_verifier.nr"),
    },
    Template {
        name: "merkle_inclusion",
        display_name: "Merkle Inclusion Proof - Prove membership in a Merkle tree",
        content: include_str!("../templates/merkle_inclusion.nr"),
    },
];

/// Run the generate command
pub fn run_generate(name: Option<String>, template: Option<String>) -> Result<()> {
    // Header
    ui::panel_header(
        emoji::SPARKLES,
        "CREATE NEW NOIR PROJECT",
        Some("Generate a new Noir circuit with optional templates"),
    );

    // Get project name
    let project_name = match name {
        Some(n) => n,
        None => {
            Input::<String>::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("{} Project name", emoji::PACKAGE))
                .interact_text()
                .context("Failed to read project name")?
        }
    };
    
    if project_name.is_empty() {
        ui::panel_error("INVALID INPUT", "Project name cannot be empty", None, None);
        return Err(anyhow::anyhow!("Project name cannot be empty"));
    }

    // Build template selection options
    let mut template_options: Vec<String> = vec![format!(
        "{} None - Start with default Noir template",
        emoji::PENDING
    )];
    for t in TEMPLATES {
        template_options.push(format!("{} {}", emoji::FILE, t.display_name));
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
                    let short_name = tmpl.name.to_lowercase();
                    let display_lower = tmpl.display_name.to_lowercase();
                    t_lower == short_name
                        || display_lower.contains(&t_lower)
                })
            }
        }
        None => {
            ui::blank();
            // Interactive selection
            let selection = Select::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("{} Select a template", emoji::FILE))
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

    ui::blank();
    
    // Run nargo new with spinner
    let spinner = ui::spinner(&format!("Creating Noir project '{}'...", project_name));
    
    let nargo_output = Command::new("nargo")
        .args(["new", &project_name])
        .output()
        .context("Failed to execute 'nargo new'. Is Nargo installed and in PATH?")?;
    
    if !nargo_output.status.success() {
        let stderr = String::from_utf8_lossy(&nargo_output.stderr);
        let stdout = String::from_utf8_lossy(&nargo_output.stdout);
        ui::spinner_error(&spinner, "Failed to create project");
        
        ui::panel_error(
            "NARGO FAILED",
            "Failed to create new Noir project",
            Some(&format!("{}\n{}", stdout, stderr)),
            Some(&["Make sure 'nargo' is installed and in PATH"]),
        );
        
        return Err(anyhow::anyhow!(
            "nargo new failed:\n{}\n{}",
            stdout,
            stderr
        ));
    }
    
    ui::spinner_success(&spinner, &format!("Created Noir project: {}", style(&project_name).green().bold()));
    
    // Apply template if selected
    if let Some(tmpl) = selected_template {
        let main_nr_path = Path::new(&project_name).join("src").join("main.nr");
        let template_name = tmpl.display_name.split(" - ").next().unwrap_or(tmpl.name);
        
        let spinner = ui::spinner(&format!("Applying template: {}...", template_name));
        
        fs::write(&main_nr_path, tmpl.content)
            .with_context(|| format!("Failed to write template to: {}", main_nr_path.display()))?;
        
        ui::spinner_success(&spinner, &format!(
            "Applied template: {}",
            style(template_name).cyan()
        ));
    }

    // Success panel
    ui::panel_success(
        "PROJECT CREATED",
        &format!("Noir project '{}' created successfully!", project_name),
    );
    
    // Ask if user wants to run zklense init
    let should_init = ui::confirm_custom(
        "Initialize zklense in this project?",
        &format!("{} Yes, initialize zklense", emoji::CHECKMARK),
        &format!("{} No, skip for now", emoji::CROSSMARK),
    )?;
    
    if should_init {
        ui::blank();
        let project_path = std::env::current_dir()?.join(&project_name);
        super::run_init(Some(project_path.to_string_lossy().to_string()));
    }

    // Next steps
    ui::section(emoji::BULB, "Next Steps");
    println!();
    println!("  {} {}", style("1.").dim(), style(format!("cd {}", project_name)).cyan());
    println!("  {} {}", style("2.").dim(), style("nargo check").cyan().to_string() + &style("    # Verify the project compiles").dim().to_string());
    println!("  {} {}", style("3.").dim(), style("nargo prove").cyan().to_string() + &style("    # Generate a proof").dim().to_string());
    ui::blank();
    
    Ok(())
}
