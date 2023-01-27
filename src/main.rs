use env_logger::{self, Env};
use log::{log_enabled, warn};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const REDIS_CLONE_VERSION: &str = env!("CARGO_PKG_VERSION");
const BITS: usize = std::mem::size_of::<usize>() * 8;

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let address = ("127.0.0.1", 8080);

    warn!("oO0OoO0OoO0Oo Redis Clone is starting oO0OoO0OoO0Oo");
    warn!(
        "Redis version={}, bits={}, pid={}, just started",
        REDIS_CLONE_VERSION,
        BITS,
        std::process::id(),
    );
    if log_enabled!(log::Level::Info) {
        println!(
            include_str!("asciiart.in"),
            REDIS_CLONE_VERSION,
            BITS,
            "standalone",
            address.1,
            std::process::id(),
        );
    }

    redis_clone::server::serve(address)?;

    Ok(())
}
