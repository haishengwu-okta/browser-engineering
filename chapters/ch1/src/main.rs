use openssl::ssl::{SslConnector, SslMethod};
use std::collections::HashMap;
use std::env;
use std::io::prelude::*;
use std::io::{Error, ErrorKind};
use std::net::TcpStream;

use hw_uri;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        let err: Error = Error::from(ErrorKind::InvalidInput);
        print!("URL is required to run this program\r\n");
        return Err(err);
    }

    for x in &args {
        print!("-- show all arguments --");
        print!("{}\r\n", x);
    }

    let maybe_uri: Option<hw_uri::URI> = hw_uri::parse(&args[1]);

    match maybe_uri {
        None => invalid_uri(&args[1]),
        Some(uri) => fetch_page(uri),
    }
}

fn invalid_uri(uri: &str) -> std::io::Result<()> {
    print!("Invalid uri {}", uri);
    Ok(())
}

fn fetch_page(url: hw_uri::URI) -> std::io::Result<()> {
    let mut buffer = String::new();
    if url.use_tls() {
        read_ssl_stream(url, &mut buffer)?;
    } else {
        read_tcp_stream(url, &mut buffer)?;
    }

    let (status, header, body) = parse_response(&buffer);
    print!("status: {}\r\n", status);
    print!("body: {}\r\n", body);
    print!("headers:\r\n");
    for (k, v) in &header {
        print!("\t{} - {}\r\n", k, v);
    }

    Ok(())
}

fn read_tcp_stream(uri_input: hw_uri::URI, buffer: &mut String) -> std::io::Result<usize> {
    let domain: String = uri_input.get_domain_port();
    let mut stream: TcpStream = TcpStream::connect(domain)?;

    stream.write(format!("GET {} HTTP/1.0\r\n", uri_input.path).as_bytes())?;
    stream.write(format!("Host: {}\r\n\r\n", uri_input.domain).as_bytes())?;
    stream.flush()?;

    stream.read_to_string(buffer)
}

fn read_ssl_stream(uri_input: hw_uri::URI, buffer: &mut String) -> std::io::Result<usize> {
    let domain: String = uri_input.get_domain_port();
    let tcp_stream: TcpStream = TcpStream::connect(domain)?;
    let connector = SslConnector::builder(SslMethod::tls()).unwrap().build();
    let mut stream = connector.connect(&uri_input.domain, tcp_stream).unwrap();

    stream.write(format!("GET {} HTTP/1.0\r\n", uri_input.path).as_bytes())?;
    stream.write(format!("Host: {}\r\n\r\n", uri_input.domain).as_bytes())?;
    stream.flush()?;

    stream.read_to_string(buffer)
}

fn parse_response(resp: &str) -> (&str, HashMap<&str, &str>, &str) {
    // split between body and others
    let xs: Vec<&str> = resp.split("\r\n\r\n").collect();
    // split between status and headers
    let mut ys: Vec<&str> = xs[0].split("\r\n").collect();
    let mut hs = HashMap::new();

    let status = ys.remove(0);
    for hl in ys.iter() {
        let zs: Vec<&str> = hl.split(": ").collect();
        hs.insert(zs[0], zs[1]);
    }
    return (status, hs, xs[1]);
}
