use anyhow::{anyhow, Result};
use base64::{Engine as _, engine::general_purpose};
use chrono::{Local, TimeZone};
use reqwest::{Client, header, RequestBuilder, Response};
use serde_json;
use url::Url;
use dtparse::parse;

use crate::{Config, types::{Project, ProjectData, Task, TokenResponse}};

pub struct TickTickClient {
    client: Client,
    pub access_token: Option<String>,
    client_id: String,
    client_secret: String,
    redirect_uri: String,
}

impl TickTickClient {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            client: Client::new(),
            access_token: None,
            client_id,
            client_secret,
            redirect_uri,
        }
    }

    // Debug helper to log HTTP requests and responses
    async fn debug_request(&self, request: RequestBuilder, description: &str) -> Result<Response> {
        let request = request.build()?;
        
        println!("🔗 HTTP {} {}", request.method(), request.url());
        
        // Log headers (excluding sensitive auth data)
        for (name, value) in request.headers() {
            if name.as_str().to_lowercase() == "authorization" {
            //     println!("   {}: [REDACTED]", name);
            // } else {
                println!("   {}: {:?}", name, value);
            }
        }
        
        // Log body if present
        if let Some(body) = request.body() {
            if let Some(bytes) = body.as_bytes() {
                if let Ok(body_str) = std::str::from_utf8(bytes) {
                    if body_str.contains("client_secret") {
                        println!("   Body: [CONTAINS SENSITIVE DATA - REDACTED]");
                    } else {
                        println!("   Body: {}", body_str);
                    }
                }
            }
        }
        
        println!("   📤 Sending {} request...", description);
        
        let response = self.client.execute(request).await?;
        
        println!("   📥 Response: {} {}", response.status().as_u16(), response.status().canonical_reason().unwrap_or(""));
        
        // Log response headers
        for (name, value) in response.headers() {
            println!("   Response {}: {:?}", name, value);
        }
        
        Ok(response)
    }

    pub fn get_authorization_url(&self, state: &str) -> String {
        let mut url = Url::parse("https://ticktick.com/oauth/authorize").unwrap();
        url.query_pairs_mut()
            .append_pair("client_id", &self.client_id)
            .append_pair("scope", "tasks:read")
            .append_pair("state", state)
            .append_pair("redirect_uri", &self.redirect_uri)
            .append_pair("response_type", "code");
        url.to_string()
    }

    pub async fn exchange_code_for_token(&mut self, code: &str, config: &mut Config) -> Result<()> {
        let auth_header = general_purpose::STANDARD.encode(format!("{}:{}", self.client_id, self.client_secret));
        
        let params = [
            ("client_id", self.client_id.as_str()),
            ("client_secret", self.client_secret.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
            ("scope", "tasks:read"),
            ("redirect_uri", self.redirect_uri.as_str()),
        ];

        let request = self.client
            .post("https://ticktick.com/oauth/token")
            .header(header::AUTHORIZATION, format!("Basic {}", auth_header))
            .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
            .form(&params);

        let response = self.debug_request(request, "OAuth token exchange").await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            println!("   📥 Response body: {}", response_text);
            
            let token_response: TokenResponse = serde_json::from_str(&response_text)?;
            self.access_token = Some(token_response.access_token.clone());
            
            // Save token to config file
            config.ticktick.access_token = Some(token_response.access_token);
            config.save()?;
            
            println!("✅ Successfully obtained and saved access token!");
            Ok(())
        } else {
            let error_text = response.text().await?;
            println!("   📥 Error response body: {}", error_text);
            Err(anyhow!("Failed to exchange code for token: {}", error_text))
        }
    }

    fn get_auth_header(&self) -> Result<String> {
        match &self.access_token {
            Some(token) => Ok(format!("Bearer {}", token)),
            None => Err(anyhow!("No access token available. Please authenticate first.")),
        }
    }

    pub async fn get_projects(&self) -> Result<Vec<Project>> {
        let auth_header = self.get_auth_header()?;
        
        let request = self.client
            .get("https://api.ticktick.com/open/v1/project")
            .header(header::AUTHORIZATION, auth_header);

        let response = self.debug_request(request, "Get projects").await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            println!("   📥 Response body: {}", response_text);
            
            let projects: Vec<Project> = serde_json::from_str(&response_text)?;
            Ok(projects)
        } else {
            let error_text = response.text().await?;
            println!("   📥 Error response body: {}", error_text);
            Err(anyhow!("Failed to get projects: {}", error_text))
        }
    }

    async fn get_project_data(&self, project_id: &str) -> Result<ProjectData> {
        let auth_header = self.get_auth_header()?;
        
        let url = format!("https://api.ticktick.com/open/v1/project/{}/data", project_id);
        let request = self.client
            .get(&url)
            .header(header::AUTHORIZATION, auth_header);

        let response = self.debug_request(request, &format!("Get project data for {}", project_id)).await?;

        if response.status().is_success() {
            let response_text = response.text().await?;
            println!("   📥 Response body: {}", response_text);
            
            let project_data: ProjectData = serde_json::from_str(&response_text)?;
            Ok(project_data)
        } else {
            let error_text = response.text().await?;
            println!("   📥 Error response body: {}", error_text);
            Err(anyhow!("Failed to get project data: {}", error_text))
        }
    }

    fn is_task_due_today(&self, task: &Task) -> bool {
        let today = Local::now().date_naive();
        
        // Check due date
        if let Some(due_date_str) = &task.due_date {
            if let Ok((due_date, _)) = parse(due_date_str) {
                // dtparse returns a NaiveDateTime, so we need to assume it's in local timezone
                let due_date_local = Local.from_local_datetime(&due_date)
                    .single()
                    .map(|dt| dt.date_naive())
                    .unwrap_or_else(|| due_date.date());
                    
                if due_date_local == today {
                    return true;
                }
            }
        }
        
        // Check start date for today
        if let Some(start_date_str) = &task.start_date {
            if let Ok((start_date, _)) = parse(start_date_str) {
                // dtparse returns a NaiveDateTime, so we need to assume it's in local timezone
                let start_date_local = Local.from_local_datetime(&start_date)
                    .single()
                    .map(|dt| dt.date_naive())
                    .unwrap_or_else(|| start_date.date());
                    
                if start_date_local == today {
                    return true;
                }
            }
        }
        
        false
    }

    pub async fn get_todays_tasks(&self) -> Result<Vec<Task>> {
        let projects = self.get_projects().await?;
        // dbg!(&projects);
        let mut todays_tasks = Vec::new();

        // Check inbox first
        println!("📥 Checking inbox for today's tasks...");

        println!("📋 Checking {} projects for today's tasks...", projects.len());
        
        for project in projects {
            println!("  🔍 Checking project: {}", project.name);
            
            match self.get_project_data(&project.id).await {
                Ok(project_data) => {
                    for task in project_data.tasks {
                        // Only include uncompleted tasks (status 0 = Normal)
                        if task.status == 0 && self.is_task_due_today(&task) {
                            todays_tasks.push(task);
                        }
                    }
                }
                Err(e) => {
                    println!("  ❌ Failed to get data for project {}: {}", project.name, e);
                }
            }
        }
        
        Ok(todays_tasks)
    }
}
