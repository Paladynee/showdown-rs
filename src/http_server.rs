use crate::components::{get_public_directory, HTTP_PORT};

use std::{
    ffi::OsStr,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    net::{IpAddr, TcpListener, TcpStream},
    path::Path,
    thread,
};

const ERROR_RESPONSE: &[u8; 39] = b"HTTP/1.1 404 NOT FOUND\r\n\r\n404 Not Found";

// -------------------------------------- HTTP SERVER --------------------------------------
pub fn create_http_server(ip_addr: IpAddr) {
    let port = HTTP_PORT;

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
    let public_directory = get_public_directory();

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

    match requested_path.canonicalize() {
        Err(_) => {
            tcp_writer.write_all(ERROR_RESPONSE).unwrap();
        }

        Ok(mut canonical_path) => {
            if !canonical_path.starts_with(&public_directory) {
                return;
            }

            if canonical_path == public_directory {
                canonical_path.push("index.html");
            }

            let file = File::open(&canonical_path);

            match file {
                Err(_) => {
                    tcp_writer.write_all(ERROR_RESPONSE).unwrap();
                }

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

                    let mut file_reader = BufReader::new(&f);
                    let mut file_buffer = [0; 1024];
                    loop {
                        let bytes_read = file_reader.read(&mut file_buffer).unwrap();
                        if bytes_read == 0 {
                            break;
                        }
                        tcp_writer.write_all(&file_buffer[..bytes_read]).unwrap();
                    }
                }
            }
        }
    }
}

// -------------------------------------- CONTENT TYPE --------------------------------------
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
