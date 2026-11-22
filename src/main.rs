use rust_course_project_rd::PreviewStats;
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("preview") => {
            let path = &args[2..].concat();

            if let Ok(contents) = fs::read_to_string(path) {
                let preview: PreviewStats = PreviewStats::new_stats(&contents);
                println!("{}", preview);
            } else {
                eprintln!("Failed to open file");
                print_help();
                std::process::exit(1);
            }
        }
        _ => {
            print_help();
        }
    };
}

fn print_help() {
    println!("Usage:");
    println!("job-queue preview path/to/file   Preview tasks import from file");
}
