use std::env;
use std::net::{IpAddr, TcpStream};
use std::io::{self, Write};
use std::result::Result;
use std::str::FromStr;
use std::process;
use std::sync::mpsc::{channel, Sender};
use std::thread;


const MAX: u16 = 65535;

#[allow(dead_code)]
struct Arguments {
    flag: String,
    ip_addr: IpAddr,
    threads: u16,
}

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        } else if args.len() > 4 {
            return Err("too many arguments");
        }
        let f = args[1].clone();
        if let Ok(ip_addr) = IpAddr::from_str(&f) {
            return Ok(Arguments {
                flag: String::from(""),
                ip_addr,
                threads: 1,
            });
        } else {
            let flag = args[1].clone();
            if flag.contains("-h") || flag.contains("-help") && args.len() == 2 {
                println!("Usage: -j to specify number of threads you want 
                \r\n       -h or -help to show this help message");
                return Err("help");
            } else if flag.contains("-h") || flag.contains("-help") {
                return Err("too many arguments");
            } else if flag.contains("-j") {
                let ip_addr = match IpAddr::from_str(&args[3]) {
                    Ok(ip_addr) => ip_addr,
                    Err(_) => return Err("not a valid ip address; must be IPv4 or IPv6"),
                };
                let threads = match args[2].parse::<u16>() {
                    Ok(threads) => threads,
                    Err(_) => return Err("Failure to parse thread number"),
                };
                return Ok(Arguments {
                    flag,
                    ip_addr,
                    threads,
                });
            } else {
                return Err("invalid flag");
            }
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port: u16 = start_port + 1;
    loop {
        match TcpStream::connect((addr, port)) {
           Ok(_) => {
               print!(".");
               io::stdout().flush().unwrap();
               tx.send(port).unwrap();
           } 
           Err(_) => {}
        }
        
        if (MAX - port) <= num_threads {
            break;
        }
        port +=  num_threads;
    }
}

fn main() {
    let args: Vec<String> = env::args().collect::<Vec<_>>();
    let program = args[0].clone();
    let arguments = Arguments::new(&args).unwrap_or_else(
        |err| {
            if err.contains("help") {
                process::exit(0);
            } else {
                eprintln!("{} problem parsing arguments: {}", program, err);
                process::exit(0);
            }
        }
    );

    let num_threads = arguments.threads;
    let addr = arguments.ip_addr;
    let (tx, rx) = channel();
    for i in 0..num_threads {
        let tx = tx.clone();

        thread::spawn(move || {
            scan(tx, i, addr, num_threads);
        });
    }

    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    }

    println!("");
    out.sort();
    for v in out {
        println!("{} is open", v);
    }
}
