#[derive(Debug)]
pub(crate) struct RunnerSpec {
    default_bin: Option<Bin>,
    case_specs: Vec<CaseSpec>,
    has_run: bool,
}

impl RunnerSpec {
    pub(crate) fn new() -> Self {
        Self {
            default_bin: None,
            case_specs: Default::default(),
            has_run: false,
        }
    }

    pub(crate) fn default_bin_path(&mut self, path: &std::path::Path) {
        self.default_bin = Some(Bin::Path(path.into()));
    }

    pub(crate) fn default_bin_name(&mut self, name: &str) {
        self.default_bin = Some(Bin::Name(name.into()));
    }

    pub(crate) fn case(&mut self, glob: &std::path::Path) {
        self.case_specs.push(CaseSpec {
            glob: glob.into(),
            expected: None,
        })
    }

    pub(crate) fn pass(&mut self, glob: &std::path::Path) {
        self.case_specs.push(CaseSpec {
            glob: glob.into(),
            expected: Some(Expected::Pass),
        })
    }

    pub(crate) fn fail(&mut self, glob: &std::path::Path) {
        self.case_specs.push(CaseSpec {
            glob: glob.into(),
            expected: Some(Expected::Fail),
        })
    }

    pub(crate) fn run(&mut self) {
        self.has_run = true;
    }

    pub(crate) fn has_run(&self) -> bool {
        self.has_run
    }
}

impl Default for RunnerSpec {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
enum Bin {
    Path(std::path::PathBuf),
    Name(String),
}

#[derive(Debug)]
pub(crate) struct CaseSpec {
    glob: std::path::PathBuf,
    expected: Option<Expected>,
}

#[derive(Copy, Clone, Debug)]
enum Expected {
    Pass,
    Fail,
}
