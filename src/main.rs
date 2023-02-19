use std::collections::HashMap;
use std::time::Duration;
use clap::Parser;
use json::JsonValue;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time::timeout;
use std::error::Error;
use std::net::{SocketAddr, IpAddr};
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
    //It is hashmap stores commands for ups
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
    if let Err(e) = run().await {
        eprintln!("{}", e);
        std::process::exit(10);
    }
}



async fn run() -> MyResult<()>{
    let config = get_args().unwrap();
    dbg!(&config);
    let remote_addr: SocketAddr = SocketAddr::new(config.addr, config.port);
    
    const MAX_DATAGRAM_SIZE: usize = 5;
    const CONNECTION_TIME: u64 = 5;

    let _ = match tokio::time::timeout(
        Duration::from_secs(CONNECTION_TIME),
        tokio::net::TcpStream::connect(remote_addr)
    )
    .await?
    {
        Ok(mut stream) => {
            dbg!(&stream); 
            

            let mut json_data = JsonValue::new_object();
            for (name, command) in config.commands{
                    let mut data = vec![0u8; MAX_DATAGRAM_SIZE];
                    
                    stream.write(command.as_bytes()).await?;
                    let my_duration = tokio::time::Duration::from_secs(5);
                    while let Ok(len) = timeout(my_duration, stream.read(&mut data)).await? {
                        let res  = String::from_utf8_lossy(&data[..len]).to_string();
                        json_data[name] = res.into();
                        break;
                        }
                                    
                }
    
    println!("{}", json_data);   
            
        }
        Err(e) => panic!("{}", format!("timeout while connecting to server : {}", e)),
    };



    Ok(())
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