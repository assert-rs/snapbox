fn main() {
    let mut args = std::env::args();
    let _ = args.next().expect("expects `$ diff <old> <new>`");
    let old_path = args.next().expect("expects `$ diff <old> <new>`");
    let new_path = args.next().expect("expects `$ diff <old> <new>`");
    if args.next().is_some() {
        panic!("expects `$ diff <old> <new>`");
    }

    let old = snapbox::Data::text(std::fs::read_to_string(&old_path).unwrap());
    let new = snapbox::Data::text(std::fs::read_to_string(&new_path).unwrap());

    let mut output = String::new();
    snapbox::report::write_diff(
        &mut output,
        &old,
        &new,
        Some(&old_path),
        Some(&new_path),
        snapbox::report::Palette::color(),
    )
    .unwrap();
    println!("{output}");
}
