use crate::types::Task;
use chrono::{Local, TimeZone};
use dtparse::parse;

/// Convert markdown links [text](url) to ANSI escape sequence links with color and underline
fn convert_markdown_links(text: &str) -> String {
    use regex::Regex;

    let re = Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    // ANSI codes: \x1b[4m = underline, \x1b[36m = cyan color, \x1b[0m = reset
    re.replace_all(
        text,
        "\x1b[4m\x1b[36m\x1b]8;;$2\x1b\\$1\x1b]8;;\x1b\\\x1b[0m",
    )
    .to_string()
}

/// Get the appropriate emoji for a task priority
fn get_priority_emoji(priority: Option<i32>) -> &'static str {
    match priority {
        Some(5) => "🔴", // High
        Some(3) => "🟡", // Medium
        Some(1) => "🔵", // Low
        _ => "⚪",       // None
    }
}

/// Format a date string to display in local time format
fn format_time(date_str: &str) -> String {
    if let Ok((datetime, _)) = parse(date_str) {
        // dtparse returns a NaiveDateTime, so we need to assume it's in local timezone
        let local_datetime = Local
            .from_local_datetime(&datetime)
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

/// Print a simplified task (without project info since it's grouped by project)
pub fn print_task_simple(task: &Task) {
    println!("  {} {}", get_priority_emoji(task.priority), task.title);

    if let Some(content) = &task.content {
        if !content.is_empty() {
            let lines: Vec<&str> = content.lines().collect();
            if lines.len() == 1 {
                let converted_content = convert_markdown_links(content);
                println!("    📝 {}", converted_content);
            } else {
                println!("    📝 Content:");
                for line in lines {
                    let converted_line = convert_markdown_links(line);
                    println!("      {}", converted_line);
                }
            }
        }
    }

    if let Some(desc) = &task.desc {
        if !desc.is_empty() {
            println!("    📄 {}", desc);
        }
    }

    if let Some(due_date) = &task.due_date {
        println!("    ⏰ Due: {}", format_time(due_date));
    }

    if let Some(start_date) = &task.start_date {
        println!("    🚀 Start: {}", format_time(start_date));
    }

    // Show subtasks
    if let Some(items) = &task.items {
        if !items.is_empty() {
            println!("    📋 Subtasks:");
            for item in items {
                let status_icon = if item.status == 1 { "✅" } else { "☐" };
                println!("      {} {}", status_icon, item.title);
            }
        }
    }
}

/// Print tasks grouped by project
pub fn print_tasks_grouped(
    tasks: &[Task],
    project_map: &std::collections::HashMap<String, String>,
) {
    use std::collections::HashMap;

    // Group tasks by project
    let mut grouped_tasks: HashMap<String, Vec<&Task>> = HashMap::new();

    for task in tasks {
        grouped_tasks
            .entry(task.project_id.clone())
            .or_insert_with(Vec::new)
            .push(task);
    }

    // Sort project IDs to ensure consistent ordering, with inbox first
    let mut project_ids: Vec<String> = grouped_tasks.keys().cloned().collect();
    project_ids.sort_by(|a, b| {
        match (a.starts_with("inbox"), b.starts_with("inbox")) {
            (true, false) => std::cmp::Ordering::Less, // inbox comes first
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.cmp(b), // alphabetical for the rest
        }
    });

    // Print each project's tasks
    for (i, project_id) in project_ids.iter().enumerate() {
        if i > 0 {
            println!(); // Add spacing between project sections
        }

        let project_name = if project_id.starts_with("inbox") {
            "📥 Inbox"
        } else {
            project_map
                .get(project_id)
                .map(|s| s.as_str())
                .unwrap_or("Unknown Project")
        };

        let project_tasks = &grouped_tasks[project_id];

        println!("╔══════════════════════════════════════════════════");
        println!(
            "║ 📁 {} ({} task{})",
            project_name,
            project_tasks.len(),
            if project_tasks.len() == 1 { "" } else { "s" }
        );
        println!("╚══════════════════════════════════════════════════");

        for task in project_tasks {
            print_task_simple(task);
            println!();
        }
    }
}
