//! For `snapbox`s tests only

use std::env;
use std::error::Error;
use std::io;
use std::io::Write;
use std::process;

fn run() -> Result<(), Box<dyn Error>> {
    if let Ok(text) = env::var("stdout") {
        println!("{text}");
    }
    if let Ok(text) = env::var("stderr") {
        eprintln!("{text}");
    }

    if env::var("echo_large").as_deref() == Ok("1") {
        for i in 0..(128 * 1024) {
            println!("{i}");
        }
    }

    if env::var("echo_cwd").as_deref() == Ok("1") {
        if let Ok(cwd) = env::current_dir() {
            eprintln!("{}", cwd.display());
        }
    }

    if let Ok(raw) = env::var("write") {
        let (path, text) = raw.split_once('=').unwrap_or((raw.as_str(), ""));
        std::fs::write(path.trim(), text.trim()).unwrap();
    }

    if let Ok(path) = env::var("cat") {
        let text = std::fs::read_to_string(path).unwrap();
        eprintln!("{text}");
    }

    if let Some(timeout) = env::var("sleep").ok().and_then(|s| s.parse().ok()) {
        std::thread::sleep(std::time::Duration::from_secs(timeout));
    }

    let code = env::var("exit")
        .ok()
        .map(|v| v.parse::<i32>())
        .map(|r| r.map(Some))
        .unwrap_or(Ok(None))?
        .unwrap_or(0);
    process::exit(code);
}

fn main() {
    let code = match run() {
        Ok(_) => 0,
        Err(ref e) => {
            write!(&mut io::stderr(), "{e}").expect("writing to stderr won't fail");
            1
        }
    };
    process::exit(code);
}
