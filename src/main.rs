use std::net::TcpListener;
use std::thread::spawn;
use tungstenite::accept;
use crossbeam_channel::unbounded;


fn main () {
    let (payload_sender, payload_receiver) = unbounded();
    let initial_payload:Vec<String> = vec![];
    payload_sender.send(initial_payload);


    let server = TcpListener::bind("0.0.0.0:9001").unwrap();
    for stream in server.incoming() {
        spawn ({
            let payload_sender = payload_sender.clone();
            let payload_receiver = payload_receiver.clone();
            move || {
                let mut websocket = accept(stream.unwrap()).unwrap();
                loop {
                    let msg = websocket.read_message().unwrap();//_or(tungstenite::Message::Text("closed".to_string()));
                    let mut payload = payload_receiver.recv().expect("Couldn't get payload from other thread");

                    if msg.to_string() != "RequestData".to_string() && msg.to_string() != "".to_string(){
                        payload.push(msg.to_string());
                        println!("{}, message is {}", payload.join(":"),msg.to_string());
                    }

                    if msg.is_binary() || msg.is_text() {
                        websocket.write_message(tungstenite::Message::Text(payload.join("<br>")));
                    }

                    payload_sender.send(payload);
                }
        }});
    }
}