use crate::{
    commands::{self, RedisCommand},
    db::Database,
    errors::{Error, Result},
    protocol::ProtoError,
    request::{self, Request},
    response::Response,
};
use log::{debug, error, info};
use std::{
    fmt::Debug,
    panic::{catch_unwind, AssertUnwindSafe},
};
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream, ToSocketAddrs},
    runtime::Runtime,
    stream::StreamExt,
    sync::mpsc::{self, Sender},
};

#[derive(Debug)]
struct Message {
    request: Request,
    response_sender: Sender<Response>,
}

pub fn serve(address: impl ToSocketAddrs + Debug) -> Result<()> {
    let db = Database::new();

    let mut rt = Runtime::new().unwrap();
    rt.block_on(async move {
        let api = start_api(db);

        start_network(api, address).await
    })
}

fn start_api(mut db: Database) -> Sender<Message> {
    let (sender, mut receiver) = mpsc::channel::<Message>(512);

    tokio::spawn(async move {
        while let Some(mut message) = receiver.next().await {
            let request = message.request;
            let mut response = Response::new();

            if let Some(cmd) = commands::lookup(request.command()) {
                api_handle_command(cmd, &mut db, &request, &mut response);
            } else {
                let msg = format!(
                    "ERR unknown command `{}`, with args beginning with: {}",
                    request.command(),
                    request.argv_to_string()
                );
                response.add_error(&msg);
            }

            if let Err(e) = message.response_sender.send(response).await {
                error!("Client receiver has gone: {:?}", e);
            }
        }
    });

    sender
}

fn api_handle_command(
    cmd: &RedisCommand,
    db: &mut Database,
    request: &Request,
    response: &mut Response,
) {
    let result = catch_unwind(AssertUnwindSafe(|| cmd.execute(db, request, response)));

    match result {
        Ok(Err(e)) => {
            error!(
                "ERROR handling command `{}` with args {}: '{}'",
                cmd.name,
                request.argv_to_string(),
                e
            );
            response.add_error("ERR server error");
        }
        Err(_) => {
            error!(
                "PANIC handling command `{}` with args {}",
                cmd.name,
                request.argv_to_string(),
            );
            response.add_error("ERR server error");
        }
        _ => (),
    }
}

async fn start_network(api: Sender<Message>, address: impl ToSocketAddrs + Debug) -> Result<()> {
    let mut listener = TcpListener::bind(&address).await?;
    let mut incoming = listener.incoming();

    // accept connections and process them serially
    info!("Listening at {:?}", address);
    while let Some(stream) = incoming.next().await {
        let stream = stream?;
        let api = api.clone();
        tokio::spawn(async move {
            if let Err(ref err) = handle_client(stream, api).await {
                error!("Error handling client: {}", err);
            }
        });
    }
    Ok(())
}

async fn handle_client(mut stream: TcpStream, mut api: Sender<Message>) -> Result<()> {
    let (read_half, mut out_stream) = stream.split();
    let mut reader = BufReader::new(read_half);
    let (response_sender, mut response_receiver) = mpsc::channel(1);

    loop {
        let request = match request::parse(&mut reader).await {
            Ok(request) => request,
            Err(Error::Proto(ProtoError::ConnectionClosed)) => {
                debug!("Client closed connection");
                break;
            }
            Err(Error::EmptyRequest) => {
                // Redis ignores this and continues to await a valid command
                continue;
            }
            Err(ref err) => {
                let msg = format!("ERR {}", err);
                error!("{}", msg);
                let mut response = Response::new();
                response.add_error(&msg);
                out_stream.write_all(&response.as_bytes()).await?;
                break;
            }
        };

        debug!("{:?}", request);

        let message = Message {
            request,
            response_sender: response_sender.clone(),
        };

        if let Err(e) = api.send(message).await {
            let msg = format!("Api receiver has gone: {}", e);
            return Err(msg.into());
        }

        match response_receiver.next().await {
            Some(response) => {
                out_stream.write_all(&response.as_bytes()).await?;
            }
            None => {
                error!("Api sender gone");
                break;
            }
        }
    }

    Ok(())
}
