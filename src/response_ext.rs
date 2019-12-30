use crate::response::Response;
use std::convert::TryInto;

pub trait ResponseExt {
    fn add_reply_help(&mut self, command: &str, help: &[&str]);
}

impl ResponseExt for Response {
    fn add_reply_help(&mut self, command: &str, help: &[&str]) {
        self.add_array_len((help.len() + 1).try_into().unwrap());

        let command = command.to_uppercase();

        let lead = format!(
            "{} <subcommand> arg arg ... arg. Subcommands are:",
            command,
        );

        self.add_simple_string(&lead);

        for line in help {
            self.add_simple_string(line);
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_reply_help() {
        let mut response = Response::new();

        response.add_reply_help("cmd", &["abc", "xyz"]);

        let expected = "\
                *3\r\n\
                +CMD <subcommand> arg arg ... arg. Subcommands are:\r\n\
                +abc\r\n\
                +xyz\r\n";

        assert_eq!(response.as_string(), expected);
    }
}
