use std::net::TcpListener;
use std::thread::{sleep, spawn};
use std::time;
use tungstenite::{accept_hdr,buffer};
use crossbeam_channel::unbounded;
use tungstenite::buffer::ReadBuffer;
use tungstenite::handshake::server::{Request, Response};


fn main () {
    let (payload_sender, payload_receiver) = unbounded();
    let initial_payload:Vec<String> = vec![];
    payload_sender.send(initial_payload);


    let server = TcpListener::bind("0.0.0.0:9001").unwrap();
    for mut stream in server.incoming() {
        spawn ({
            let payload_sender = payload_sender.clone();
            let payload_receiver = payload_receiver.clone();
            move || {
                let callback = |req: &Request, mut response: Response| {
                    println!("Received a new ws handshake");
                    println!("The request's path is: {}", req.uri().path());
                    println!("The request's headers are:");
                    for (ref header, _value) in req.headers() {
                        println!("* {}", header);
                    }

                    // Let's add an additional header to our response to the client.
                    let _headers = response.headers_mut();
                    //headers.append("MyCustomHeader", ":)".parse().unwrap());
                    //headers.append("SOME_TUNGSTENITE_HEADER", "header_value".parse().unwrap());

                    Ok(response)
                };
                let mut websocket = accept_hdr(stream.unwrap(),callback).unwrap();

                /*spawn({ // Make a new thread that sends the payload every 30 seconds
                    let payload_sender = payload_sender.clone();
                    let payload_receiver = payload_receiver.clone();
                    let mut websocket = & mut websocket;
                    move ||{
                        loop{
                            sleep(time::Duration::from_millis(30000));
                            let payload = payload_receiver.recv().expect("Couldn't get payload from other thread");
                            websocket.write_message(tungstenite::Message::Text(payload.join("<br>")));
                            payload_sender.send(payload);
                        }
                    }});*/

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