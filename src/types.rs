use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub closed: Option<bool>,
    #[serde(rename = "groupId")]
    pub group_id: Option<String>,
    #[serde(rename = "viewMode")]
    pub view_mode: Option<String>,
    pub permission: Option<String>,
    pub kind: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChecklistItem {
    pub id: Option<String>,
    pub title: String,
    pub status: i32, // 0 = Normal, 1 = Completed
    #[serde(rename = "completedTime")]
    pub completed_time: Option<String>,
    #[serde(rename = "isAllDay")]
    pub is_all_day: Option<bool>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i64>,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Task {
    pub id: String,
    #[serde(rename = "projectId")]
    pub project_id: String,
    pub title: String,
    #[serde(rename = "isAllDay")]
    pub is_all_day: Option<bool>,
    #[serde(rename = "completedTime")]
    pub completed_time: Option<String>,
    pub content: Option<String>,
    pub desc: Option<String>,
    #[serde(rename = "dueDate")]
    pub due_date: Option<String>,
    pub items: Option<Vec<ChecklistItem>>,
    pub priority: Option<i32>, // 0 = None, 1 = Low, 3 = Medium, 5 = High
    pub reminders: Option<Vec<String>>,
    #[serde(rename = "repeatFlag")]
    pub repeat_flag: Option<String>,
    #[serde(rename = "sortOrder")]
    pub sort_order: Option<i64>,
    #[serde(rename = "startDate")]
    pub start_date: Option<String>,
    pub status: i32, // 0 = Normal, 2 = Completed
    #[serde(rename = "timeZone")]
    pub time_zone: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectData {
    pub project: Project,
    pub tasks: Vec<Task>,
    pub columns: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct InboxData {
    pub tasks: Vec<Task>,
    pub columns: Option<Vec<serde_json::Value>>,
}
