use chrono::{Local, TimeZone};
use dtparse::parse;
use crate::types::Task;

/// Get the appropriate emoji for a task priority
fn get_priority_emoji(priority: Option<i32>) -> &'static str {
    match priority {
        Some(5) => "🔴", // High
        Some(3) => "🟡", // Medium
        Some(1) => "🔵", // Low
        _ => "⚪",        // None
    }
}

/// Format a date string to display in local time format
fn format_time(date_str: &str) -> String {
    if let Ok((datetime, _)) = parse(date_str) {
        // dtparse returns a NaiveDateTime, so we need to assume it's in local timezone
        let local_datetime = Local.from_local_datetime(&datetime)
            .single()
            .unwrap_or_else(|| Local::now());
        
        // Format as "Today HH:MM", "Tomorrow HH:MM", or "MMM DD HH:MM"
        let now = Local::now();
        let today = now.date_naive();
        let datetime_date = local_datetime.date_naive();
        
        if datetime_date == today {
            format!("Today {}", local_datetime.format("%I:%M %p"))
        } else if datetime_date == today + chrono::Days::new(1) {
            format!("Tomorrow {}", local_datetime.format("%I:%M %p"))
        } else {
            local_datetime.format("%b %d %I:%M %p").to_string()
        }
    } else {
        "Invalid time".to_string()
    }
}

/// Print a formatted task with project information
pub fn print_task(task: &Task, project_name: &str) {
    println!("┌─────────────────────────────────────────────────");
    println!("│ {} {}", get_priority_emoji(task.priority), task.title);
    println!("│ 📁 Project: {}", project_name);
    
    if let Some(content) = &task.content {
        if !content.is_empty() {
            println!("│ 📝 Content: {}", content);
        }
    }
    
    if let Some(desc) = &task.desc {
        if !desc.is_empty() {
            println!("│ 📄 Description: {}", desc);
        }
    }
    
    if let Some(due_date) = &task.due_date {
        println!("│ ⏰ Due: {}", format_time(due_date));
        // println!("│ ⏰ Due: {}", due_date);
    }
    
    if let Some(start_date) = &task.start_date {
        println!("│ 🚀 Start: {}", format_time(start_date));
        // println!("│ 🚀 Start: {}", start_date);
    }
    
    // Show subtasks
    if let Some(items) = &task.items {
        if !items.is_empty() {
            println!("│ 📋 Subtasks:");
            for item in items {
                let status_icon = if item.status == 1 { "✅" } else { "☐" };
                println!("│   {} {}", status_icon, item.title);
            }
        }
    }
    
    println!("└─────────────────────────────────────────────────");
}
