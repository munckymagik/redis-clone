use env_logger::{self, Env};
use redis_clone;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    env_logger::from_env(Env::default().default_filter_or("info")).init();

    redis_clone::server::serve()?;

    Ok(())
}
