#[macro_export]
macro_rules! parse_or_reply_with_err {
    ($arg:expr, $resp:expr) => {
        match $arg.parse() {
            Ok(n) => n,
            Err(_) => {
                $resp.add_reply_not_a_number();
                return Ok(());
            }
        }
    };
}

#[macro_export]
macro_rules! parse_arg_or_reply_with_err {
    ($idx:literal, $req:expr, $resp:expr) => {
        parse_or_reply_with_err!($req.arg($idx)?, $resp)
    };
}
