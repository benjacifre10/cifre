use crate::domain::models::Task;
use chrono::{NaiveDate, Utc};
use std::fs;

pub fn cleanup_old_done_tasks() -> anyhow::Result<()> {
    let path = "data/task.json";
    if !std::path::Path::new(path).exists() {
        return Ok(());
    }

    let content = fs::read_to_string(path)?;
    let mut tasks: Vec<Task> = serde_json::from_str(&content)?;
    
    let now = Utc::now().date_naive();
    let one_month_ago = now - chrono::Duration::days(30);
    
    tasks.retain(|task| {
        if task.state != "done" {
            return true;
        }
        
        if let Ok(finish_date) = NaiveDate::parse_from_str(&task.finish_date, "%Y-%m-%d") {
            finish_date > one_month_ago
        } else {
            true
        }
    });
    
    let json = serde_json::to_string_pretty(&tasks)?;
    fs::write(path, json)?;
    
    Ok(())
}
