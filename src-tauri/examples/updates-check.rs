//! Read-only GitHub release check; does not access USB or open a browser.
use am8_lab::updates::Updates;

fn main() {
    let updates = Updates::new();
    match updates.check() {
        Ok(info) => println!("{}", serde_json::to_string_pretty(&info).expect("release metadata")),
        Err(error) => {
            eprintln!("Release check failed: {error}");
            std::process::exit(1);
        }
    }
}
