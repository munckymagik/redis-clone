use redis_clone;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    redis_clone::server::serve()
}
