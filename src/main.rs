extern crate http;

use std::io::net::ip::{SocketAddr, Ipv4Addr};

use http::server::{Config, Server, Request, ResponseWriter};

#[deriving(Clone)]
struct SlackBot {
    port: int
}

impl Server for SlackBot {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: self.port as u16 } }
    }

    fn handle_request(&self, req: Request, resp: &mut ResponseWriter) {
        println!("Got request:");
        println!("{}", req.body);
    }
}

fn main() {
    let args = std::os::args();
    let port = from_str(args[1].as_slice()).unwrap();

    println!("Starting server on port {}", port);

    let slackbot = SlackBot {
        port: port
    };

    slackbot.serve_forever();
}
