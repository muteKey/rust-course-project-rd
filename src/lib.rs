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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_job_keeps_kind() {
        assert_eq!(new_job("send_email").kind, "send_email");
    }
}
