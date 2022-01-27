#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum File {
    Binary(Vec<u8>),
    Text(String),
}

impl File {
    pub(crate) fn read_from(
        path: &std::path::Path,
        binary: Option<bool>,
    ) -> Result<Self, crate::Error> {
        let data = match binary {
            Some(true) => {
                let data = std::fs::read(&path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                Self::Binary(data)
            }
            Some(false) => {
                let data = std::fs::read_to_string(&path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                let data = normalize_text(&data);
                Self::Text(data)
            }
            None => {
                let data = std::fs::read(&path)
                    .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
                Self::Binary(data).try_utf8()
            }
        };
        Ok(data)
    }

    pub(crate) fn write_to(&self, path: &std::path::Path) -> Result<(), crate::Error> {
        std::fs::write(path, self.as_bytes())
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e).into())
    }

    pub(crate) fn replace_lines(
        &mut self,
        line_nums: std::ops::Range<usize>,
        text: &str,
    ) -> Result<(), crate::Error> {
        let mut output_lines = String::new();

        let s = self
            .as_str()
            .ok_or("Binary file can't have lines replaced")?;
        for (line_num, line) in crate::lines::LinesWithTerminator::new(s)
            .enumerate()
            .map(|(i, l)| (i + 1, l))
        {
            if line_num == line_nums.start {
                output_lines.push_str(text);
                if !text.is_empty() && !text.ends_with('\n') {
                    output_lines.push('\n');
                }
            }
            if !line_nums.contains(&line_num) {
                output_lines.push_str(line);
            }
        }

        *self = Self::Text(output_lines);
        Ok(())
    }

    pub(crate) fn map_text(self, op: impl FnOnce(&str) -> String) -> Self {
        match self {
            Self::Binary(data) => Self::Binary(data),
            Self::Text(data) => Self::Text(op(&data)),
        }
    }

    pub(crate) fn utf8(&mut self) -> Result<(), std::str::Utf8Error> {
        match self {
            Self::Binary(data) => {
                let data = String::from_utf8(data.clone()).map_err(|e| e.utf8_error())?;
                let data = normalize_text(&data);
                *self = Self::Text(data);
                Ok(())
            }
            Self::Text(_) => Ok(()),
        }
    }

    pub(crate) fn try_utf8(self) -> Self {
        match self {
            Self::Binary(data) => {
                if is_binary(&data) {
                    Self::Binary(data)
                } else {
                    match String::from_utf8(data) {
                        Ok(data) => {
                            let data = normalize_text(&data);
                            Self::Text(data)
                        }
                        Err(err) => {
                            let data = err.into_bytes();
                            Self::Binary(data)
                        }
                    }
                }
            }
            Self::Text(data) => Self::Text(data),
        }
    }

    pub(crate) fn into_utf8(self) -> Result<String, std::str::Utf8Error> {
        match self {
            Self::Binary(data) => {
                let data = String::from_utf8(data).map_err(|e| e.utf8_error())?;
                let data = normalize_text(&data);
                Ok(data)
            }
            Self::Text(data) => Ok(data),
        }
    }

    pub(crate) fn as_str(&self) -> Option<&str> {
        match self {
            Self::Binary(_) => None,
            Self::Text(data) => Some(data.as_str()),
        }
    }

    pub(crate) fn as_bytes(&self) -> &[u8] {
        match self {
            Self::Binary(data) => data,
            Self::Text(data) => data.as_bytes(),
        }
    }
}

pub(crate) fn normalize_text(data: &str) -> String {
    normalize_line_endings::normalized(data.chars())
        // Also help out with Windows paths
        .map(|c| if c == '\\' { '/' } else { c })
        .collect()
}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Binary(data) => String::from_utf8_lossy(data).fmt(f),
            Self::Text(data) => data.fmt(f),
        }
    }
}

#[cfg(not(feature = "filesystem"))]
fn is_binary(data: &[u8]) -> bool {
    false
}

#[cfg(feature = "filesystem")]
fn is_binary(data: &[u8]) -> bool {
    match content_inspector::inspect(data) {
        content_inspector::ContentType::BINARY |
        // We don't support these
        content_inspector::ContentType::UTF_16LE |
        content_inspector::ContentType::UTF_16BE |
        content_inspector::ContentType::UTF_32LE |
        content_inspector::ContentType::UTF_32BE => {
            true
        },
        content_inspector::ContentType::UTF_8 |
        content_inspector::ContentType::UTF_8_BOM => {
            false
        },
    }
}

#[derive(Debug)]
pub(crate) enum FilesystemContext {
    Default,
    Path(std::path::PathBuf),
    SandboxPath(std::path::PathBuf),
    #[cfg(feature = "filesystem")]
    SandboxTemp {
        temp: tempfile::TempDir,
        path: std::path::PathBuf,
    },
}

impl FilesystemContext {
    #[cfg_attr(not(feature = "filesystem"), allow(unused_variables))]
    pub(crate) fn new(
        path: &std::path::Path,
        cwd: Option<&std::path::Path>,
        sandbox: bool,
        mode: &crate::Mode,
    ) -> Result<Self, std::io::Error> {
        if sandbox {
            #[cfg(feature = "filesystem")]
            match mode {
                crate::Mode::Dump(root) => {
                    let target = root.join(path.with_extension("out").file_name().unwrap());
                    let _ = std::fs::remove_dir_all(&target);
                    std::fs::create_dir_all(&target)?;
                    if let Some(cwd) = cwd {
                        debug!("Initializing {} from {}", target.display(), cwd.display());
                        copy_dir(cwd, &target)?;
                    }
                    Ok(Self::SandboxPath(target))
                }
                crate::Mode::Fail | crate::Mode::Overwrite => {
                    let temp = tempfile::tempdir()?;
                    // We need to get the `/private` prefix on Mac so variable substitutions work
                    // correctly
                    let path = canonicalize(temp.path())?;
                    if let Some(cwd) = cwd {
                        debug!("Initializing {} from {}", path.display(), cwd.display());
                        copy_dir(cwd, &path)?;
                    }
                    Ok(Self::SandboxTemp { temp, path })
                }
            }
            #[cfg(not(feature = "filesystem"))]
            Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Sandboxing is disabled",
            ))
        } else {
            Ok(cwd.map(|p| Self::Path(p.to_owned())).unwrap_or_default())
        }
    }

    pub(crate) fn is_sandbox(&self) -> bool {
        match self {
            Self::Default | Self::Path(_) => false,
            Self::SandboxPath(_) => true,
            #[cfg(feature = "filesystem")]
            Self::SandboxTemp { .. } => true,
        }
    }

    pub(crate) fn path(&self) -> Option<&std::path::Path> {
        match self {
            Self::Default => None,
            Self::Path(path) => Some(path.as_path()),
            Self::SandboxPath(path) => Some(path.as_path()),
            #[cfg(feature = "filesystem")]
            Self::SandboxTemp { path, .. } => Some(path.as_path()),
        }
    }

    pub(crate) fn close(self) -> Result<(), std::io::Error> {
        match self {
            Self::Default | Self::Path(_) | Self::SandboxPath(_) => Ok(()),
            #[cfg(feature = "filesystem")]
            Self::SandboxTemp { temp, .. } => temp.close(),
        }
    }
}

impl Default for FilesystemContext {
    fn default() -> Self {
        Self::Default
    }
}

#[cfg(feature = "filesystem")]
pub(crate) struct Iterate {
    inner: walkdir::IntoIter,
}

#[cfg(feature = "filesystem")]
impl Iterate {
    pub(crate) fn new(path: &std::path::Path) -> Self {
        Self {
            inner: walkdir::WalkDir::new(path).into_iter(),
        }
    }
}

#[cfg(feature = "filesystem")]
impl Iterator for Iterate {
    type Item = Result<std::path::PathBuf, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(entry) = self.inner.next().map(|e| {
            e.map(walkdir::DirEntry::into_path)
                .map_err(std::io::Error::from)
        }) {
            if entry.as_ref().ok().and_then(|e| e.file_name())
                != Some(std::ffi::OsStr::new(".keep"))
            {
                return Some(entry);
            }
        }
        None
    }
}

#[cfg(not(feature = "filesystem"))]
pub(crate) struct Iterate {}

#[cfg(not(feature = "filesystem"))]
impl Iterate {
    pub(crate) fn new(_path: &std::path::Path) -> Self {
        Self {}
    }
}

#[cfg(not(feature = "filesystem"))]
impl Iterator for Iterate {
    type Item = Result<std::path::PathBuf, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

#[cfg(feature = "filesystem")]
fn copy_dir(source: &std::path::Path, dest: &std::path::Path) -> Result<(), std::io::Error> {
    let source = canonicalize(source)?;
    let dest = canonicalize(dest)?;

    for current in Iterate::new(&source) {
        let current = current?;
        let rel = current.strip_prefix(&source).unwrap();
        let target = dest.join(rel);

        shallow_copy(&current, &target)?;
    }

    Ok(())
}

pub(crate) fn shallow_copy(
    source: &std::path::Path,
    dest: &std::path::Path,
) -> Result<(), std::io::Error> {
    let meta = source.symlink_metadata()?;
    if meta.is_dir() {
        std::fs::create_dir_all(dest)?;
    } else if meta.is_file() {
        std::fs::copy(source, dest)?;
    } else if let Ok(target) = std::fs::read_link(source) {
        symlink_to_file(dest, &target)?;
    }

    Ok(())
}

#[cfg(windows)]
fn symlink_to_file(link: &std::path::Path, target: &std::path::Path) -> Result<(), std::io::Error> {
    std::os::windows::fs::symlink_file(target, link)
}

#[cfg(not(windows))]
fn symlink_to_file(link: &std::path::Path, target: &std::path::Path) -> Result<(), std::io::Error> {
    std::os::unix::fs::symlink(target, link)
}

pub(crate) fn resolve_dir(path: std::path::PathBuf) -> Result<std::path::PathBuf, std::io::Error> {
    let meta = std::fs::symlink_metadata(&path)?;
    if meta.is_dir() {
        canonicalize(&path)
    } else if meta.is_file() {
        // Git might checkout symlinks as files
        let target = std::fs::read_to_string(&path)?;
        let target_path = path.parent().unwrap().join(target);
        resolve_dir(target_path)
    } else {
        canonicalize(&path)
    }
}

fn canonicalize(path: &std::path::Path) -> Result<std::path::PathBuf, std::io::Error> {
    #[cfg(feature = "filesystem")]
    {
        dunce::canonicalize(path)
    }
    #[cfg(not(feature = "filesystem"))]
    {
        // Hope for the best
        Ok(strip_trailing_slash(path).to_owned())
    }
}

pub(crate) fn strip_trailing_slash(path: &std::path::Path) -> &std::path::Path {
    path.components().as_path()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn replace_lines_same_line_count() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..3;
        let replacement = "World\n";
        let expected = File::Text("One\nWorld\nThree".into());

        let mut actual = File::Text(input.into());
        actual.replace_lines(line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_lines_grow() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..3;
        let replacement = "World\nTrees\n";
        let expected = File::Text("One\nWorld\nTrees\nThree".into());

        let mut actual = File::Text(input.into());
        actual.replace_lines(line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_lines_shrink() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..3;
        let replacement = "";
        let expected = File::Text("One\nThree".into());

        let mut actual = File::Text(input.into());
        actual.replace_lines(line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_lines_no_trailing() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..3;
        let replacement = "World";
        let expected = File::Text("One\nWorld\nThree".into());

        let mut actual = File::Text(input.into());
        actual.replace_lines(line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn replace_lines_empty_range() {
        let input = "One\nTwo\nThree";
        let line_nums = 2..2;
        let replacement = "World\n";
        let expected = File::Text("One\nWorld\nTwo\nThree".into());

        let mut actual = File::Text(input.into());
        actual.replace_lines(line_nums, replacement).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn strips_trailing_slash() {
        let path = std::path::Path::new("/foo/bar/");
        let rendered = path.display().to_string();
        assert_eq!(rendered.as_bytes()[rendered.len() - 1], b'/');

        let stripped = strip_trailing_slash(path);
        let rendered = stripped.display().to_string();
        assert_eq!(rendered.as_bytes()[rendered.len() - 1], b'r');
    }
}
