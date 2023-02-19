use std::collections::HashMap;
use clap::Parser;
use json::JsonValue;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use std::error::Error;
use std::net::{SocketAddr, IpAddr};
use tokio::net::TcpSocket;
use std::ops::RangeInclusive;


type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Parser)]
#[command(name = "Rups")]
#[command(author = "Mikhail Vasilchyk <mmishkin747@gmail.com>")]
#[command(version = "0.1")]
#[command(about = "Rust check state UPS via telnet, extertal script for zabbix")]
struct Cli {
    /// User's name for connecting ups
    #[arg(short, long)]
    user: Option<String>,
    /// Password for connecting ups
    #[arg(short, long)]
    password: Option<String>,
    /// Network ipv4 address server
    ipaddr: IpAddr,
    /// Network port to use
    #[arg( long, value_parser = port_in_range, default_value_t = 2001)]
    port: u16,
}

#[derive(Debug)]
pub struct Config <'a> {
    addr: IpAddr,
    port: u16,
    user: String,
    passw: String,
    commands: HashMap<&'a str, &'a str>
}

pub fn get_args() -> MyResult<Config<'static>>{
    let cli = Cli::parse();
    let mut user = String::new();
    let mut passw = String::new();
    if let Some(ref user_v) = cli.user {
        if let Some(ref passw_v) = cli.password {
            user = user_v.to_string();
            passw = passw_v.to_string();
        }
    }
    //this is hashmap stores commands for ups
    let commands = HashMap::from([
        ("main_voltage", "O"),
        ("load", "P"),
        ("temperature", "C"),
        ("charge_battaries", "0"),
        ("workin_hour", "j"),
        
    ]);

    Ok(Config {
        addr: cli.ipaddr,
        port: cli.port,
        user,
        passw,
        commands,

    })
}



#[tokio::main]
async fn main() {
    let config = get_args().unwrap();
    dbg!(&config);
    let remote_addr: SocketAddr = SocketAddr::new(config.addr, config.port);
    
    let socket = TcpSocket::new_v4().unwrap();
    const MAX_DATAGRAM_SIZE: usize = 10;
    
    let mut strem = socket.connect(remote_addr).await.unwrap();
    

    //dbg!(&socket);
    let mut json_data = JsonValue::new_object();
    for (name, command) in config.commands{
            let mut data = vec![0u8; MAX_DATAGRAM_SIZE];
            
            strem.write(command.as_bytes()).await.unwrap();
            //socket.send(command.as_str().as_bytes()).await.unwrap();
            
            let len = strem.read(&mut data).await.unwrap();
            
            let res  = String::from_utf8_lossy(&data[..len]).to_string();
            json_data[name] = res.into();
    }
    
    println!("{}", json_data);
}

/// This func check valid number port
fn port_in_range(s: &str) -> Result<u16, String> {
    let port_range: RangeInclusive<usize> = 1..=65535;
    let port: usize = s
        .parse()
        .map_err(|_| format!("`{}` isn't a port number", s))?;
    if port_range.contains(&port) {
        Ok(port as u16)
    } else {
        Err(format!(
            "Port not in range {}-{}",
            port_range.start(),
            port_range.end()
        ))
    }
}