use anyhow::Result;
use std::env;

mod auth;
mod client;
mod config;
mod display;
mod types;

use auth::{interactive_auth, perform_oauth_flow};
use client::TickTickClient;
use config::Config;
use display::print_task;

#[tokio::main]
async fn main() -> Result<()> {
    // Try to load from environment variables first
    let client = if let (Ok(client_id), Ok(client_secret), Ok(redirect_uri), Ok(access_token)) = (
        env::var("TICKTICK_CLIENT_ID"),
        env::var("TICKTICK_CLIENT_SECRET"), 
        env::var("TICKTICK_REDIRECT_URI"),
        env::var("TICKTICK_ACCESS_TOKEN")
    ) {
        let mut client = TickTickClient::new(client_id, client_secret, redirect_uri);
        client.access_token = Some(access_token);
        println!("✅ Using credentials from environment variables");
        client
    } else {
        // Try to load from config file
        match Config::load() {
            Ok(mut config) => {
                println!("📁 Found configuration file ~/.ticktick.toml");
                
                // Check if we already have a stored access token
                if let Some(stored_token) = &config.ticktick.access_token {
                    println!("✅ Using stored access token from configuration file");
                    let mut client = TickTickClient::new(
                        config.ticktick.client_id.clone(), 
                        config.ticktick.client_secret.clone(), 
                        config.ticktick.redirect_uri.clone()
                    );
                    client.access_token = Some(stored_token.clone());
                    
                    // Test if the token still works by trying to fetch projects
                    println!("🔍 Verifying stored access token...");
                    println!("🌐 About to make HTTP request to verify token...");
                    match client.get_projects().await {
                        Ok(_) => {
                            println!("✅ Stored access token is valid");
                            client
                        }
                        Err(_) => {
                            println!("❌ Stored access token is invalid or expired, requesting new one...");
                            config.ticktick.access_token = None; // Clear invalid token
                            
                            let client_id = config.ticktick.client_id.clone();
                            let client_secret = config.ticktick.client_secret.clone();
                            let redirect_uri = config.ticktick.redirect_uri.clone();
                            
                            let mut client = TickTickClient::new(client_id, client_secret, redirect_uri);
                            
                            // Perform OAuth flow
                            perform_oauth_flow(&mut client, &mut config).await?;
                            client
                        }
                    }
                } else {
                    println!("🔑 No stored access token found, initiating OAuth flow...");
                    
                    let client_id = config.ticktick.client_id.clone();
                    let client_secret = config.ticktick.client_secret.clone();
                    let redirect_uri = config.ticktick.redirect_uri.clone();
                    
                    let mut client = TickTickClient::new(client_id, client_secret, redirect_uri);
                    
                    // Perform OAuth flow
                    perform_oauth_flow(&mut client, &mut config).await?;
                    client
                }
            }
            Err(_) => {
                // Fallback to interactive authentication
                let (client, _config) = interactive_auth().await?;
                client
            }
        }
    };

    println!();
    println!("🗓️  Fetching today's tasks...");
    println!("🌐 About to make HTTP requests to fetch tasks...");
    println!();

    match client.get_todays_tasks().await {
        Ok(tasks) => {
            if tasks.is_empty() {
                println!("🎉 No tasks due today! You're all caught up!");
            } else {
                println!("📅 You have {} task(s) for today:", tasks.len());
                println!();
                
                // We need to get project names for display
                println!("🌐 Making additional HTTP request to get project names...");
                let projects = client.get_projects().await?;
                let project_map: std::collections::HashMap<String, String> = projects
                    .into_iter()
                    .map(|p| (p.id, p.name))
                    .collect();
                
                for task in &tasks {
                    let project_name = project_map
                        .get(&task.project_id)
                        .map(|s| s.as_str())
                        .unwrap_or("Unknown Project");
                    print_task(task, project_name);
                    println!();
                }
            }
        }
        Err(e) => {
            println!("❌ Error fetching tasks: {}", e);
        }
    }

    Ok(())
}
