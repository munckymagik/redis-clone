use log::{debug, error, info, warn};
use std::sync::{Arc, Mutex};
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
    runtime::Runtime,
    stream::StreamExt,
};

use crate::{
    commands,
    db::Database,
    errors::{Error, Result},
    protocol::RespError,
    request,
    response::Response,
};

pub fn serve() -> Result<()> {
    let db = Arc::new(Mutex::new(Database::new()));
    let address = "127.0.0.1:8080";

    let mut rt = Runtime::new().unwrap();
    rt.block_on(start_network(db, address))
}

async fn start_network(db: Arc<Mutex<Database>>, address: &str) -> Result<()> {
    let mut listener = TcpListener::bind(address).await?;
    let mut incoming = listener.incoming();

    // accept connections and process them serially
    info!("Listening at {}", address);
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let db = Arc::clone(&db);
        tokio::spawn(async move {
            if let Err(ref err) = handle_client(stream, db).await {
                error!("Error handling client: {}", err);
            }
        });
    }
    Ok(())
}

async fn handle_client(mut stream: TcpStream, db: Arc<Mutex<Database>>) -> Result<()> {
    let (read_half, mut out_stream) = stream.split();
    let mut reader = BufReader::new(read_half);

    loop {
        let mut response = Response::new();
        let request = match request::parse(&mut reader).await {
            Ok(request) => request,
            Err(Error::Resp(RespError::ConnectionClosed)) => {
                debug!("Client closed connection");
                break;
            }
            Err(Error::EmptyQuery) => {
                // Redis ignores this and continues to await a valid command
                continue;
            }
            Err(ref err) => {
                let msg = format!("ERR {}", err);
                error!("{}", msg);
                response.add_error(&msg);
                out_stream.write_all(&response.as_bytes()).await?;
                break;
            }
        };

        debug!("{:?}", request);

        if let Some(cmd) = commands::lookup(&request.command) {
            cmd.execute(Arc::clone(&db), &request, &mut response)?;
            out_stream.write_all(&response.as_bytes()).await?;
        } else {
            let msg = format!(
                "ERR unknown command `{}`, with args beginning with: {}",
                request.command,
                request.argv_to_string()
            );
            warn!("{}", msg);
            response.add_error(&msg);
            out_stream.write_all(&response.as_bytes()).await?;
        }
    }

    Ok(())
}
