use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::oneshot;
use warp::Filter;

use crate::client::TickTickClient;
use crate::config::{Config, TickTickConfig, default_redirect_uri};

pub async fn interactive_auth() -> Result<(TickTickClient, Config)> {
    println!("🎯 TickTick Today's Tasks");
    println!("========================");
    println!();
    
    // Try to load configuration from TOML file
    let mut config = match Config::load() {
        Ok(config) => {
            println!("✅ Loaded configuration from ~/.ticktick.toml");
            config
        }
        Err(e) => {
            println!("❌ {}", e);
            println!();
            print!("Would you like to create an example configuration file? (y/N): ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes" {
                Config::create_example()?;
                return Err(anyhow!("Please edit the configuration file with your credentials and run the program again."));
            }
            
            println!();
            println!("Manual setup:");
            println!("To get started, you need to set up your TickTick API credentials:");
            println!("1. Visit the TickTick Developer Center to register your application");
            println!("2. Get your client_id and client_secret");
            println!("3. Set redirect_uri to: http://localhost:8080/callback");
            println!();

            print!("Enter your client_id: ");
            io::stdout().flush()?;
            let mut client_id = String::new();
            io::stdin().read_line(&mut client_id)?;
            let client_id = client_id.trim().to_string();

            print!("Enter your client_secret: ");
            io::stdout().flush()?;
            let mut client_secret = String::new();
            io::stdin().read_line(&mut client_secret)?;
            let client_secret = client_secret.trim().to_string();

            Config {
                ticktick: TickTickConfig {
                    client_id,
                    client_secret,
                    redirect_uri: default_redirect_uri(),
                    access_token: None,
                }
            }
        }
    };

    let port = 8080;
    let mut client = TickTickClient::new(
        config.ticktick.client_id.clone(), 
        config.ticktick.client_secret.clone(), 
        config.ticktick.redirect_uri.clone()
    );

    println!();
    println!("🔗 Please visit this URL to authorize the application:");
    println!("{}", client.get_authorization_url("state123"));
    println!();
    println!("🌐 Waiting for authorization callback...");
    println!("   (A browser window should open automatically, or copy the URL above)");
    
    // Try to open the URL in the default browser
    let auth_url = client.get_authorization_url("state123");
    let _ = std::process::Command::new("nu")
        .args(&["-c", &("start ".to_owned() + &auth_url)])
        .spawn();

    // Start the callback server and wait for the code
    let code = start_callback_server(port).await?;
    
    println!("✅ Received authorization code, exchanging for access token...");
    client.exchange_code_for_token(&code, &mut config).await?;
    Ok((client, config))
}

pub async fn start_callback_server(port: u16) -> Result<String> {
    let (tx, rx) = oneshot::channel();
    let tx = Arc::new(tokio::sync::Mutex::new(Some(tx)));

    // Create callback handler
    let callback = warp::path("callback")
        .and(warp::query::<HashMap<String, String>>())
        .and(warp::any().map(move || tx.clone()))
        .and_then(|params: HashMap<String, String>, tx: Arc<tokio::sync::Mutex<Option<oneshot::Sender<String>>>>| async move {
            if let Some(code) = params.get("code") {
                let mut sender = tx.lock().await;
                if let Some(sender) = sender.take() {
                    let _ = sender.send(code.clone());
                }
                Ok(warp::reply::html(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>TickTick Authorization</title>
                        <style>
                            body { font-family: Arial, sans-serif; text-align: center; padding: 50px; background: #f5f5f5; }
                            .container { background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); max-width: 500px; margin: 0 auto; }
                            .success { color: #4CAF50; font-size: 24px; margin-bottom: 20px; }
                            .code { background: #f0f0f0; padding: 10px; border-radius: 5px; font-family: monospace; word-break: break-all; }
                        </style>
                    </head>
                    <body>
                        <div class="container">
                            <div class="success">✅ Authorization Successful!</div>
                            <p>You have successfully authorized the TickTick application.</p>
                            <p>You can now close this browser window and return to the terminal.</p>
                        </div>
                    </body>
                    </html>
                    "#
                ))
            } else if let Some(error) = params.get("error") {
                let mut sender = tx.lock().await;
                if let Some(sender) = sender.take() {
                    let _ = sender.send(format!("ERROR:{}", error));
                }
                Ok(warp::reply::html(
                    r#"
                    <!DOCTYPE html>
                    <html>
                    <head>
                        <title>TickTick Authorization Error</title>
                        <style>
                            body { font-family: Arial, sans-serif; text-align: center; padding: 50px; background: #f5f5f5; }
                            .container { background: white; padding: 30px; border-radius: 10px; box-shadow: 0 2px 10px rgba(0,0,0,0.1); max-width: 500px; margin: 0 auto; }
                            .error { color: #f44336; font-size: 24px; margin-bottom: 20px; }
                        </style>
                    </head>
                    <body>
                        <div class="container">
                            <div class="error">❌ Authorization Failed</div>
                            <p>There was an error during authorization. Please try again.</p>
                            <p>You can close this browser window and return to the terminal.</p>
                        </div>
                    </body>
                    </html>
                    "#
                ))
            } else {
                Err(warp::reject::not_found())
            }
        });

    // Start the server
    let server = warp::serve(callback)
        .bind(([127, 0, 0, 1], port));

    println!("🌐 Started local callback server on http://localhost:{}/callback", port);
    
    // Run server in background
    tokio::spawn(server);
    
    // Wait for the callback
    match rx.await {
        Ok(code) => {
            if code.starts_with("ERROR:") {
                Err(anyhow!("Authorization error: {}", &code[6..]))
            } else {
                Ok(code)
            }
        }
        Err(_) => Err(anyhow!("Failed to receive authorization code")),
    }
}

pub async fn perform_oauth_flow(client: &mut TickTickClient, config: &mut Config) -> Result<()> {
    // Perform OAuth flow
    println!("🔗 Please visit this URL to authorize the application:");
    println!("{}", client.get_authorization_url("state123"));
    println!();
    println!("🌐 Waiting for authorization callback...");
    println!("   (A browser window should open automatically, or copy the URL above)");
    
    // Try to open the URL in the default browser
    let auth_url = client.get_authorization_url("state123");
    let _ = std::process::Command::new("nu")
        .args(&["-c", &("start ".to_owned() + &auth_url)])
        .spawn();

    // Start the callback server and wait for the code
    let code = start_callback_server(8080).await?;
    
    println!("✅ Received authorization code, exchanging for access token...");
    client.exchange_code_for_token(&code, config).await?;
    Ok(())
}