extern crate http;
extern crate url;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::collections::HashMap;

use url::Url;

use http::server::{Config, Server, Request, ResponseWriter};
use http::headers::content_type::MediaType;

#[allow(dead_code)]
struct SlackCommand {
    channel_name:   String,
    timestamp:      f64,
    username:       String,
    text:           String,
    args:           Vec<String>
}

trait Callable: Send + Sync {
    fn call(&self, cmd: &mut SlackCommand);
}


impl Callable for fn(&mut SlackCommand) {
    fn call(&self, cmd: &mut SlackCommand) {
        (*self)(cmd);
    }
}

impl Callable for Box<Callable + Send + Sync> {
    fn call(&self, cmd: &mut SlackCommand) {
        self.call(cmd);
    }
}

impl Clone for Box<Callable + Send + Sync> {
    fn clone(&self) -> Box<Callable + Send + Sync> {
        self.clone()
    }
}

#[deriving(Clone)]
struct SlackBot {
    port: int,
    commands: HashMap<String, Box<Callable + Send + Sync>>
}

impl SlackBot {
    fn register_command(&mut self, name: &str, func: fn(&mut SlackCommand)) {
        self.commands.insert(name.to_string(), box func);
    }
}

impl Server for SlackBot {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: self.port as u16 } }
    }

    fn handle_request(&self, req: Request, resp: &mut ResponseWriter) {
        resp.headers.content_type = Some(MediaType {
            type_: "application".to_string(),
            subtype: "json".to_string(),
            parameters: vec![]
        });
        
        resp.write(b"{\"text\":\"test post please ignore\"}");

        // yeah... it's bad                 vvvvvvvvvvvvvv
        let url = match Url::parse(format!("http://a.com{}", req.request_uri).as_slice()) {
            Ok(url) => url,
            Err(_) => {
                println!("Could not parse request URL: {}", req.request_uri);
                return
            }
        };
        let query: Vec<(String, String)> = match url.query_pairs() {
            Some(vec) => vec,
            None => {
                println!("URL does not have query string.");
                return
            }
        };

        let map = pairs_to_hashmap(query.clone());
        let text = map.find(& String::from_str("text")).unwrap().clone();
        let args0: Vec<&str> = text.as_slice().split(' ').collect();
        
        // map_in_place doesn't work because elements are not the same size
        let mut args1: Vec<String> = Vec::from_fn(args0.len(), |i| args0[i].to_string());
        args1.remove(0); // shift() is deprecated

        let slack_req = SlackCommand {
            channel_name: map.find(& String::from_str("channel_name")).unwrap().clone(),
            timestamp: from_str(map.find(& String::from_str("timestamp")).unwrap().as_slice()).unwrap(),
            username: map.find(& String::from_str("user_name")).unwrap().clone(),
            text: text.clone(),
            args: args1
        };
    }
}

fn pairs_to_hashmap(pairs: Vec<(String, String)>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for &(ref val0, ref val1) in pairs.iter() {
        map.insert(val0.clone(), val1.clone());
    }
    map
}

fn main() {
    let args = std::os::args();

    if args.len() < 2 {
        println!("No port number supplied.");
        return
    }

    let port = match from_str(args[1].as_slice()) {
        Some(i) => i,
        None => {
            println!("Invalid port number.");
            return
        }
    };

    println!("Starting server on port {}", port);

    let slackbot = SlackBot {
        port: port,
        commands: HashMap::new()
    };

    slackbot.serve_forever();
}
