extern crate http;
extern crate url;
extern crate serialize;

use std::io::net::ip::{SocketAddr, Ipv4Addr};
use std::collections::{HashMap, TreeMap};

use url::Url;

use http::server::{Config, Server, Request, ResponseWriter};
use http::headers::content_type::MediaType;

use serialize::json;
use serialize::json::ToJson;

#[allow(dead_code)]
pub struct SlackCommand {
    pub channel_name:   String,
    pub timestamp:      f64,
    pub username:       String,
    pub text:           String,
    pub args:           Vec<String>
}

#[allow(dead_code)]
impl SlackCommand {
    pub fn join_after(&self, index: uint) -> String {
        let mut string = String::new();
        for i in range(index, self.args.len()) {
            string.push_str(self.args[i].as_slice());
        }
        string
    }

    pub fn int_arg(&self, index: uint) -> Option<int> {
        if index >= self.args.len() {
            None
        } else {
            from_str(self.args[index].as_slice())
        }
    }

    pub fn safe_arg(&self, index: uint) -> Option<String> {
        if index >= self.args.len() {
            None
        } else {
            Some(self.args[index].clone())
        }
    }
}

pub struct SlackResponse {
    pub username:   Option<String>,
    pub icon_url:   Option<String>,
    pub icon_emoji: Option<String>,

    pub response:   Vec<String>
}

impl SlackResponse {
    pub fn to_json(&self, bot: &SlackBot) -> String {
        let mut map = TreeMap::new();

        map.insert("text".to_string(), self.response.connect("\n").to_json());
        if self.icon_url.is_some() {
            map.insert("icon_url".to_string(), self.icon_url.to_json());
        } else if bot.icon_url.is_some() {
            map.insert("icon_url".to_string(), bot.icon_url.to_json());
        }

        if self.icon_emoji.is_some() {
            map.insert("icon_emoji".to_string(), self.icon_emoji.to_json());
        } else if bot.icon_emoji.is_some() {
            map.insert("icon_emoji".to_string(), bot.icon_emoji.to_json());
        }

        if self.username.is_some() {
            map.insert("username".to_string(), self.username.to_json());
        } else if bot.username.is_some() {
            map.insert("username".to_string(), bot.username.to_json());
        }

        format!("{}", json::Object(map))
    }

    pub fn reply(&mut self, string: &str) {
        self.response.push(string.to_string());
    }
}

pub struct CommandManager {
    pub commands: HashMap<String, fn(&mut SlackCommand, &mut SlackResponse)>
}

impl CommandManager {
    pub fn register(&mut self, name: String, func: fn(&mut SlackCommand, &mut SlackResponse)) {
        self.commands.insert(name, func);
    }

    pub fn handle(&mut self, name: String, cmd: &mut SlackCommand, resp: &mut SlackResponse) -> Option<Vec<String>> {
        match self.commands.find(& name) {
            Some(func) => {
                println!("Command: \"{}\", user: {}, arguments: {}", name, cmd.username, cmd.args);
                (*func)(cmd, resp)
            },
            None => {
                resp.reply(format!("Command not found: {}", name).as_slice());
                return None
            }
        }

        Some(resp.response.clone())
    }
}

impl Clone for CommandManager { 
    // Can't clone the command map directly for some reason
    fn clone(&self) -> CommandManager {
        let mut map: HashMap<String, fn(&mut SlackCommand, &mut SlackResponse)> = HashMap::new();
        for (string, func) in self.commands.iter() {
            map.insert(string.clone(), *func);
        }
        CommandManager { commands: map }
    }
}


#[deriving(Clone)]
pub struct SlackBot {
    pub port: int,
    pub manager: CommandManager,

    pub username:    Option<String>,
    pub icon_url:    Option<String>,
    pub icon_emoji:  Option<String>
}

impl SlackBot {
    #[allow(dead_code)]
    pub fn new(port: int) -> SlackBot {
        SlackBot {
            port:       port,
            manager:    CommandManager { commands: HashMap::new() },

            username:   None,
            icon_url:   None,
            icon_emoji: None
        }
    }

    pub fn start(self) {
        self.serve_forever();
    }
}

impl Server for SlackBot {
    fn get_config(&self) -> Config {
        Config { bind_address: SocketAddr { ip: Ipv4Addr(127, 0, 0, 1), port: self.port as u16 } }
    }

    fn handle_request(&self, req: Request, resp: &mut ResponseWriter) {
        // `placeholder.com` because Url::parse wants a complete URL.
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
            channel_name:   map.find(&"channel_name".to_string()).unwrap().clone(),
            timestamp:      from_str(map.find(&"timestamp".to_string()).unwrap().as_slice()).unwrap(),
            username:       map.find(&"user_name".to_string()).unwrap().clone(),
            text:           text.clone(),
            args:           args1
        };

        let mut slack_response = SlackResponse {
            response:   vec![],

            username:   None,
            icon_url:   None,
            icon_emoji: None
        };

        self.manager.clone().handle(command.clone(), &mut slack_cmd, &mut slack_response);

        let resp_text = slack_response.to_json(self);

        resp.headers.content_type = Some(MediaType {
            type_: "application".to_string(),
            subtype: "json".to_string(),
            parameters: vec![]
        });

        resp.write(resp_text.into_bytes().as_slice()).unwrap();
    }
}

fn pairs_to_hashmap(pairs: Vec<(String, String)>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for &(ref val0, ref val1) in pairs.iter() {
        map.insert(val0.clone(), val1.clone());
    }
    map
}

#[allow(dead_code)]
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
        manager: CommandManager { commands: HashMap::new() },

        username: None,
        icon_emoji: None,
        icon_url: None
    };

    #[allow(unused_variable)]
    fn version_command(cmd: &mut SlackCommand, resp: &mut SlackResponse) {
        resp.reply("slackbot.rs version 0.1");
    }

    slackbot.manager.register("version".to_string(), version_command);

    slackbot.serve_forever();
}
