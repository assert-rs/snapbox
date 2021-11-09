use std::io::prelude::*;

/// If `input`, write it to `child`'s stdin while also reading `child`'s
/// stdout and stderr, then wait on `child` and return its status and output.
///
/// This was lifted from `std::process::Child::wait_with_output` and modified
/// to also write to stdin.
pub(crate) fn wait_with_input_output(
    mut child: std::process::Child,
    input: Option<&[u8]>,
    timeout: Option<std::time::Duration>,
) -> std::io::Result<std::process::Output> {
    let input = input.map(|b| b.to_owned());
    let stdin = input.and_then(|i| {
        child
            .stdin
            .take()
            .map(|mut stdin| std::thread::spawn(move || stdin.write_all(&i)))
    });
    fn read<R>(mut input: R) -> std::thread::JoinHandle<std::io::Result<Vec<u8>>>
    where
        R: Read + Send + 'static,
    {
        std::thread::spawn(move || {
            let mut ret = Vec::new();
            input.read_to_end(&mut ret).map(|_| ret)
        })
    }
    let stdout = child.stdout.take().map(read);
    let stderr = child.stderr.take().map(read);

    // Finish writing stdin before waiting, because waiting drops stdin.
    stdin.and_then(|t| t.join().unwrap().ok());
    let status = if let Some(timeout) = timeout {
        wait_timeout::ChildExt::wait_timeout(&mut child, timeout)
            .transpose()
            .unwrap_or_else(|| {
                let _ = child.kill();
                child.wait()
            })
    } else {
        child.wait()
    }?;

    let stdout = stdout
        .and_then(|t| t.join().unwrap().ok())
        .unwrap_or_default();
    let stderr = stderr
        .and_then(|t| t.join().unwrap().ok())
        .unwrap_or_default();

    Ok(std::process::Output {
        status,
        stdout,
        stderr,
    })
}
