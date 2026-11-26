use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fmt;
use uuid::Uuid;

pub struct Job {
    pub id: Uuid,
    pub kind: String,
}

pub fn new_job(kind: &str) -> Job {
    Job {
        id: Uuid::new_v4(),
        kind: kind.to_string(),
    }
}

#[derive(Debug)]
pub struct TaskRow {
    pub kind: String,     // "send_email"
    pub priority: u8,     // 5
    pub max_attempts: u8, // 3
}

pub struct PreviewStats {
    pub total_records_count: usize,
    pub bad_lines: Vec<(usize, String)>,
    pub tasks: Vec<TaskRow>,
}

impl PreviewStats {
    pub fn new_stats(contents: &str) -> PreviewStats {
        let (tasks, bad_lines) = parse_tasks(contents);

        PreviewStats {
            total_records_count: tasks.len() + bad_lines.len(),
            bad_lines,
            tasks,
        }
    }
}

impl fmt::Display for PreviewStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for bad_line in &self.bad_lines {
            writeln!(f, "Line {}: {}", bad_line.0 + 1, bad_line.1)?;
        }
        writeln!(f, "Total tasks count: {}", self.total_records_count)?;
        writeln!(f, "Correct tasks count: {}", self.tasks.len())?;
        writeln!(f, "Invalid tasks count: {}", self.bad_lines.len())?;
        Ok(())
    }
}

pub fn parse_line(line: &str) -> Result<TaskRow, String> {
    let fields: Vec<&str> = line.split(',').collect();
    if fields.len() != 3 {
        return Err("Incorrect fields number".to_string());
    }

    let kind = fields[0].trim().to_string();

    if kind.is_empty() {
        return Err("Kind cannot be empty".to_string());
    }

    let priority_result: Option<u8> = fields[1].trim().parse().ok();
    let max_attempts_result: Option<u8> = fields[2].trim().parse().ok();

    match (priority_result, max_attempts_result) {
        (Some(priority), Some(max_attempts)) => Ok(TaskRow {
            kind,
            priority,
            max_attempts,
        }),
        (None, _) => Err("Invalid priority value".to_string()),
        (_, None) => Err("Invalid max attempts value".to_string()),
    }
}

pub fn parse_tasks(text: &str) -> (Vec<TaskRow>, Vec<(usize, String)>) {
    let mut tasks = Vec::new();
    let mut bad_lines = Vec::new();
    let mut unique_kinds = HashSet::new();

    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();
        let kind = fields[0].trim().to_string();
        if kind.is_empty() {
            bad_lines.push((index, "Kind cannot be empty".to_string()));
            continue;
        }
        if !unique_kinds.insert(kind) {
            bad_lines.push((index, "Duplicate value".to_string()));
            continue;
        }

        match parse_line(line) {
            Ok(task) => tasks.push(task),
            Err(reason) => bad_lines.push((index, reason)),
        }
    }
    (tasks, bad_lines)
}

pub fn filter_kind_duplicates(tasks: &[TaskRow]) -> Vec<String> {
    let mut set = HashSet::new();
    for task in tasks {
        set.insert(task.kind.clone());
    }
    set.into_iter().collect()
}

pub fn count_by_kind(tasks: &[TaskRow]) -> BTreeMap<String, u32> {
    let mut counts = BTreeMap::new();
    for task in tasks {
        *counts.entry(task.kind.clone()).or_insert(0) += 1;
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_job_keeps_kind() {
        assert_eq!(new_job("send_email").kind, "send_email");
    }

    #[test]
    fn parse_line_empty_kind() {
        let result = parse_line(",2,1");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err, "Kind cannot be empty");
    }

    #[test]
    fn parse_line_incorrect_priority() {
        let result = parse_line("archive_backup,high,1");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err, "Invalid priority value");
    }

    #[test]
    fn parse_line_priority_overflow() {
        let result = parse_line("archive_backup,256,1");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err, "Invalid priority value");
    }
    #[test]
    fn parse_line_max_attempts_overflow() {
        let result = parse_line("archive_backup,7,256");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err, "Invalid max attempts value");
    }

    #[test]
    fn parse_line_max_attempts_incorrect_value() {
        let result = parse_line("archive_backup,7,2y");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err, "Invalid max attempts value");
    }

    #[test]
    fn parse_line_too_many_items() {
        let result = parse_line("archive_backup,7,256,undefined");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err, "Incorrect fields number");
    }

    #[test]
    fn parse_line_happy_path() {
        let result = parse_line("archive_backup,7,2");
        assert!(result.is_ok());
        let task = result.unwrap();
        assert_eq!(task.kind, "archive_backup");
        assert_eq!(task.priority, 7);
        assert_eq!(task.max_attempts, 2);
    }
    #[test]
    fn parse_line_trim() {
        let result = parse_line("  send_email, 0,4 ");
        assert!(result.is_ok());
        let task = result.unwrap();
        assert_eq!(task.kind, "send_email");
        assert_eq!(task.priority, 0);
        assert_eq!(task.max_attempts, 4);
    }

    #[test]
    fn preview_stats_normal_path() {
        let contents = "email_send,5,3\nreport_generation,8,2";
        let preview = PreviewStats::new_stats(contents);
        assert!(preview.bad_lines.is_empty());
        assert!(preview.tasks.len() == 2);
    }

    #[test]
    fn preview_stats_has_errors() {
        let contents = ",5,3\nreport_generation,8,2\ndata_export,5,2\nvideo_transcode,10,4,NaN";
        let preview = PreviewStats::new_stats(contents);
        assert!(preview.bad_lines.len() == 2);

        let contents = "notification_push,9,5\nreport_generation,2026,2\ndata_export,5,2025\ncleanup_expired,,1";
        let preview = PreviewStats::new_stats(contents);
        assert!(preview.bad_lines.len() == 3);

        let contents = "notification_push,9,5\nnotification_push,2,2\nnotification_push,5,7";
        let preview = PreviewStats::new_stats(contents);
        assert!(preview.bad_lines.len() == 2);
    }
}
