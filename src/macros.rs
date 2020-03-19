#[macro_export]
macro_rules! parse_arg_or_reply_with_err {
    ($idx:literal, $req:expr, $resp:expr) => {
        match $req.arg($idx)?.parse() {
            Ok(n) => n,
            Err(_) => {
                $resp.add_reply_not_a_number();
                return Ok(());
            }
        };
    };
}
