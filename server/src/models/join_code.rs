#[derive(Debug, PartialEq, Eq)]
pub struct JoinCode(String);

impl JoinCode {
    fn new() -> Self {
        JoinCode()
    }
}
