use crate::response::Response;
use byte_string::ByteStr;
use std::convert::TryInto;

/// Helpers methods that extend Response with facilities more logically
/// related to the command interactions than the protocol
pub trait ResponseExt {
    fn add_reply_help(&mut self, command: ByteStr, help: &[&str]);
    fn add_reply_subcommand_syntax_error(&mut self, command: ByteStr, sub_command: ByteStr);
    fn add_reply_wrong_number_of_arguments(&mut self, command: ByteStr);
    fn add_reply_wrong_type(&mut self);
    fn add_reply_not_a_number(&mut self);
}

impl ResponseExt for Response {
    fn add_reply_help(&mut self, command: ByteStr, help: &[&str]) {
        self.add_array_len((help.len() + 1).try_into().unwrap());

        let command = command.to_uppercase();

        let lead = format!("{} <subcommand> arg arg ... arg. Subcommands are:", command);

        self.add_simple_string(&lead);

        for line in help {
            self.add_simple_string(line);
        }
    }

    fn add_reply_subcommand_syntax_error(&mut self, command: ByteStr, sub_command: ByteStr) {
        let command = command.to_uppercase();

        let message = format!(
            "ERR Unknown subcommand or wrong number of arguments for '{}'. Try {} HELP.",
            sub_command, command,
        );

        self.add_error(&message);
    }

    fn add_reply_wrong_number_of_arguments(&mut self, command: ByteStr) {
        let command = command.to_lowercase();
        let message = format!("ERR wrong number of arguments for '{}' command", command,);

        self.add_error(&message);
    }

    fn add_reply_wrong_type(&mut self) {
        self.add_error("WRONGTYPE Operation against a key holding the wrong kind of value");
    }

    fn add_reply_not_a_number(&mut self) {
        self.add_error("ERR value is not an integer or out of range");
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_reply_help() {
        let mut response = Response::new();

        response.add_reply_help("cmd".into(), &["abc", "xyz"]);

        let expected = "\
                        *3\r\n\
                        +CMD <subcommand> arg arg ... arg. Subcommands are:\r\n\
                        +abc\r\n\
                        +xyz\r\n";

        assert_eq!(response.as_string(), expected);
    }

    #[test]
    fn test_add_reply_subcommand_syntax_error() {
        let mut response = Response::new();

        response.add_reply_subcommand_syntax_error("cmd".into(), "xyz".into());

        let expected =
            "-ERR Unknown subcommand or wrong number of arguments for 'xyz'. Try CMD HELP.\r\n";

        assert_eq!(response.as_string(), expected);
    }

    #[test]
    fn test_add_reply_wrong_number_of_arguments() {
        let mut response = Response::new();

        response.add_reply_wrong_number_of_arguments("CMD".into());

        let expected = "-ERR wrong number of arguments for 'cmd' command\r\n";

        assert_eq!(response.as_string(), expected);
    }
}
