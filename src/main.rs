use std::net::TcpListener;
use native_tls::{Identity, TlsAcceptor, TlsStream};
use std::thread::{sleep, spawn};
use std::time;
use tungstenite::{accept_hdr,buffer};
use crossbeam_channel::unbounded;
use tungstenite::buffer::ReadBuffer;
use tungstenite::handshake::server::{Request, Response};
use std::sync::Arc;
use std::fs::File;
use std::io::Read;

fn main () {
    let (payload_sender, payload_receiver) = unbounded();
    let initial_payload:Vec<String> = vec![];
    payload_sender.send(initial_payload);

    // Load SSL certificate
    let mut cert_file = File::open("/etc/letsencrypt/live/frhs.tech/fullchain.pem").unwrap();
    let mut cert = vec![];
    cert_file.read_to_end(&mut cert).unwrap();

    // Load SSL key
    let mut key_file = File::open("/etc/letsencrypt/live/frhs.tech/privkey.pem").unwrap();
    let mut key = vec![];
    key_file.read_to_end(&mut key).unwrap();

    // Use cert and key to make identity
    let identity = Identity::from_pkcs8(&cert,&key).unwrap();

    // Open a server and bind to port 9001
    let server = TcpListener::bind("0.0.0.0:9001").unwrap();

    // Finish creating the acceptor
    let acceptor = TlsAcceptor::new(identity).unwrap();
    let acceptor = Arc::new(acceptor);

    for mut stream in server.incoming() {
        spawn ({
            let payload_sender = payload_sender.clone();
            let payload_receiver = payload_receiver.clone();
            let acceptor = acceptor.clone();
            move || {
                let stream = acceptor.accept(stream.unwrap()).unwrap();

                let callback = |req: &Request, mut response: Response| {
                    println!("Received a new ws handshake");
                    println!("The request's path is: {}", req.uri().path());
                    println!("The request's headers are:");
                    for (ref header, _value) in req.headers() {
                        println!("* {}", header);
                    }

                    let _headers = response.headers_mut();

                    Ok(response)
                };

                let mut websocket = accept_hdr(stream,callback).unwrap();

                loop { // Wait for messages
                    let msg = websocket.read_message().unwrap();

                    let mut payload = payload_receiver.recv().expect("Couldn't get payload from other thread");

                    if msg.to_string() != "RequestData".to_string() && msg.to_string() != "".to_string(){
                        // If the purpose of the socket is to send data
                        payload.push(msg.to_string());
                        println!("{}, message is {}", payload.join(":"),msg.to_string());
                    } else{
                        // If this is the every 30 seconds socket
                        payload_sender.send(payload);
                        loop{
                            let payload = payload_receiver.recv().expect("Couldn't get payload from other thread");
                            websocket.write_message(tungstenite::Message::Text(payload.join("<br>")));
                            payload_sender.send(payload);
                            sleep(time::Duration::from_millis(30000));
                        }
                    }

                    if msg.is_binary() || msg.is_text() {
                        websocket.write_message(tungstenite::Message::Text(payload.join("<br>")));
                    }

                    payload_sender.send(payload);
                    break;
                }
        }});
    }
}