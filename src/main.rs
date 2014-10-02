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
    args:           Vec<String>,

    response:       Vec<String>
}

impl SlackCommand {
    fn reply(&mut self, string: &str) {
        self.response.push(string.to_string());
    }
}

struct CommandManager {
    commands: HashMap<String, fn(&mut SlackCommand)>
}

impl CommandManager {
    fn register(&mut self, name: String, func: fn(&mut SlackCommand)) {
        self.commands.insert(name, func);
    }

    fn handle(&mut self, name: String, cmd: &mut SlackCommand) -> Option<Vec<String>> {
        match self.commands.find(& name) {
            Some(func) => (*func)(cmd),
            None => return None
        }

        Some(cmd.response.clone())
    }
}

impl Clone for CommandManager { 
    // Can't clone the command map directly for some reason
    fn clone(&self) -> CommandManager {
        let mut map: HashMap<String, fn(&mut SlackCommand)> = HashMap::new();
        for (string, func) in self.commands.iter() {
            map.insert(string.clone(), *func);
        }
        CommandManager { commands: map }
    }
}


#[deriving(Clone)]
struct SlackBot {
    port: int,
    manager: CommandManager
}

impl Server for SlackBot {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: self.port as u16 } }
    }

    fn handle_request(&self, req: Request, resp: &mut ResponseWriter) {
        // yeah... it's bad                 vvvvvvvvvvvvvv
        let url = match Url::parse(format!("http://placeholder.com?{}", req.body).as_slice()) {
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
        let command = match args1.remove(0) {
            Some(cmd) => cmd,
            None => "".to_string()
        };

        let mut slack_cmd = SlackCommand {
            channel_name: map.find(& String::from_str("channel_name")).unwrap().clone(),
            timestamp: from_str(map.find(& String::from_str("timestamp")).unwrap().as_slice()).unwrap(),
            username: map.find(& String::from_str("user_name")).unwrap().clone(),
            text: text.clone(),
            args: args1,

            response: vec![]
        };

        let response = match self.manager.clone().handle(command.clone(), &mut slack_cmd) {
            Some(v) => v,
            None => vec![format!("Command not found: {}", command.clone()).to_string()]
        };
        
        resp.headers.content_type = Some(MediaType {
            type_: "application".to_string(),
            subtype: "json".to_string(),
            parameters: vec![]
        });

        let bytes = format!("{{\"text\": \"{}\"}}", response.connect("\n")).into_bytes();
        resp.write(bytes.as_slice()).unwrap();
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

    let mut slackbot = SlackBot {
        port: port,
        manager: CommandManager { commands: HashMap::new() }
    };

    fn test_command(cmd: &mut SlackCommand) {
        cmd.reply("It works!");
    }

    slackbot.manager.register("test".to_string(), test_command);

    slackbot.serve_forever();
}
