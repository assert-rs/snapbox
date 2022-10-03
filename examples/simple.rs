use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 2 {
        println!("Hello {}!", args[1]);
    } else {
        eprintln!("Must supply exactly one argument.");
        std::process::exit(1);
    }
}
