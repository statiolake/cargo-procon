mod imp;
mod ui;

fn main() {
    match ui::main() {
        Ok(_) => {}
        Err(e) => {
            println!("  error: {}", e);
            for cause in e.iter_causes() {
                println!(" due to: {}", cause);
            }

            std::process::exit(1);
        }
    }
}
