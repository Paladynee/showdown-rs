use crate::{ServerMode, HTTP_PORT, PUBLIC_DIRECTORY};

use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    ffi::OsStr,
    fs::File,
    net::{TcpListener, TcpStream},
    path::Path,
    thread,
};


// -------------------------------------- HTTP SERVER --------------------------------------
pub fn create_http_server(server_mode: ServerMode) {
    let port = HTTP_PORT;
    let ip_addr = match server_mode {
        ServerMode::Development => "127.0.0.1".parse().unwrap(),
        ServerMode::Production => local_ip_address::local_ip().unwrap(),
    };

    let http_listener = TcpListener::bind((ip_addr, port)).unwrap();
    eprintln!("[THREAD::HTTP] http://{}:{} is listening...", ip_addr, port);

    let mut connection_threads = vec![];

    for stream in http_listener.incoming().flatten() {
        let thread = thread::spawn(move || {
            handle_http_connection(stream);
        });

        connection_threads.push(thread);
    }

    for thread in connection_threads {
        thread.join().unwrap();
    }
}


// -------------------------------------- HTTP CONNECTION --------------------------------------
fn handle_http_connection(stream: TcpStream) {
    let public_directory: &Path = PUBLIC_DIRECTORY.as_ref();

    let tcp_reader = BufReader::new(&stream);
    let mut tcp_writer = BufWriter::new(&stream);

    let request_lines = tcp_reader
        .lines()
        .take_while(|line| !line.as_ref().unwrap().is_empty())
        .map(|line| line.unwrap())
        .collect::<Vec<_>>();

    if request_lines.is_empty() {
        return;
    }

    let first_line = &request_lines[0];

    let parts = first_line.split_whitespace().collect::<Vec<_>>();

    let method = parts[0];
    let mut path = parts[1].to_owned();

    if method != "GET" {
        return;
    }

    if path.starts_with("/") {
        path = path.replacen("/", "", 1);
    }

    let requested_path = public_directory.join(path);


    // TODO: refactor file serving logic
    const ERROR_RESPONSE: &[u8] = b"HTTP/1.1 404 NOT FOUND\r\n\r\n";

    match requested_path.canonicalize() {
        Ok(canonical_path) => {
            if !canonical_path.starts_with(public_directory) {
                return;
            }

            let file = File::open(&canonical_path);
            match file {
                Ok(f) => {
                    eprintln!(
                        "[THREAD::HTTP] {}: Serve {}",
                        stream.peer_addr().unwrap(),
                        canonical_path.display()
                    );

                    let file_size = f.metadata().unwrap().len();
                    let content_type = get_content_type(&canonical_path);

                    let response_headers = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n",
                        content_type, file_size
                    );

                    tcp_writer.write_all(response_headers.as_bytes()).unwrap();

                    // Write file
                    io::copy(&mut BufReader::new(&f), &mut tcp_writer).unwrap();
                }
                Err(_) => {
                    tcp_writer.write_all(ERROR_RESPONSE).unwrap();
                }
            }
        }
        Err(_) => {
            tcp_writer.write_all(ERROR_RESPONSE).unwrap();
        }
    }
}


pub fn get_content_type(path: &Path) -> &'static str {
    if let Some(ext) = path.extension().and_then(OsStr::to_str) {
        match ext {
            "html" => "text/html",
            "css" => "text/css",
            "js" => "application/javascript",
            "png" => "image/png",
            "jpg" => "image/jpeg",
            "jpeg" => "image/jpeg",
            "gif" => "image/gif",
            _ => "application/octet-stream",
        }
    } else {
        "application/octet-stream"
    }
}
