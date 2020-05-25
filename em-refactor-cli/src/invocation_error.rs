pub(crate) type InvocationResult<T> = Result<T, InvocationError>;

pub(crate) struct InvocationError {
    pub message: String
}

impl InvocationError {
    pub(crate) fn new(message: String) -> Self {
        Self {
            message
        }
    }
    pub(crate) fn from_output(cmd: &std::process::Command, output: &std::process::Output) -> Self {
        Self::new(format!("{:?}\n{}", cmd, std::str::from_utf8(output.stderr.as_slice()).unwrap()))
    }
}

impl From<std::io::Error> for InvocationError {
    fn from(err: std::io::Error) -> Self {
        Self::new(err.to_string())
    }
}
