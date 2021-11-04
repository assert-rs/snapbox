#[derive(Copy, Clone, Debug)]
pub(crate) enum Expected {
    Pass,
    Fail,
}

#[derive(Clone, Debug)]
pub(crate) enum Bin {
    Path(std::path::PathBuf),
    Name(String),
}
