use std::env;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

use grape_badge_service::server::{self, Server};
use grape_badge_service::store::Store;

fn main() -> std::io::Result<()> {
    let port: u16 = env::var("GRP_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse()
        .unwrap_or(8080);
    let data_dir = env::var_os("GRP_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../data"));
    let server = Server {
        store: Arc::new(Store::new(data_dir)),
    };
    let listener = TcpListener::bind(("0.0.0.0", port))?;
    println!("grapeGram badge service on :{port}");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let server = server.clone();
                thread::spawn(move || {
                    let response = match server::Request::read(&mut stream) {
                        Ok(request) => server.handle(&request),
                        Err(error) => server::error_response(&error),
                    };
                    if let Err(error) = server::write_response(&mut stream, response) {
                        eprintln!("response error: {error}");
                    }
                });
            }
            Err(error) => eprintln!("accept error: {error}"),
        }
    }
    Ok(())
}
