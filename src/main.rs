use rust_course_project_rd::new_job;

fn main() {
    let kind = std::env::args().nth(1).unwrap_or(String::from("noop"));
    let job = new_job(&kind);
    println!("created job {} ({})", job.id, kind);
}
