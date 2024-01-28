use std::{
    env,
    io::{self, Write},
    net::{IpAddr, TcpStream},
    process,
    str::FromStr,
    sync::mpsc::{channel, Sender},
    thread,
};

// Max IP Port.
const MAX: u16 = 65535;

struct Args {
    ipaddr: IpAddr,
    threads: u16,
}

impl Args {
    // we use 'static lifetime because we want to return a reference to a string in the main function
    fn new(args: &Vec<String>) -> Result<Args, &'static str> {
        if args.len() < 2 {
            return Err("not enough arguments");
        } else if args.len() > 4 {
            return Err("too many arguments");
        }

        // check if the first argument is a valid IP address
        let f = args[1].clone();
        if let Ok(ipaddr) = IpAddr::from_str(&f) {
            return Ok(Args { ipaddr, threads: 4 });
        }

        // check if -h or --help is the first argument
        let flag = args[1].clone();
        if flag.contains("-h") || flag.contains("--help") && args.len() == 2 {
            println!(
                "\r\nUsage:\r\n-j to select how many threads you want\r\n-h or --help to show this help message.\r\n\r\nExample:\r\n./port-scanner -j 100"
            );

            // we return an error because we want to handle this separately from this function
            return Err("help");
        }

        // -h or --help should be the only argument
        if flag.contains("-h") || flag.contains("--help") {
            return Err("too many arguments");
        }

        // check if the first argument is -j
        if flag.contains("-j") {
            // check if the second argument is a valid integer
            let threads = match args[2].parse::<u16>() {
                Ok(s) => s,
                Err(_) => return Err("failed to parse thread number"),
            };

            // check if the third argument is a valid IP address
            let ipaddr = match IpAddr::from_str(&args[3]) {
                Ok(s) => s,
                Err(_) => return Err("not a valid IPADDR; must be IPv4 or IPv6"),
            };

            return Ok(Args { threads, ipaddr });
        }

        // if we reach this point, we have an invalid syntax
        return Err("invalid syntax");
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port: u16 = start_port + 1;

    // we use loop because we want to keep scanning until we reach the last port
    loop {
        match TcpStream::connect((addr, port)) {
            Ok(_) => {
                // user feedback
                print!(".");

                // we flush the output because we want to print the output immediately
                io::stdout().flush().unwrap();

                // we send the port number to the main thread
                tx.send(port).unwrap();
            }
            Err(_) => {} // we don't want to print closed ports
        }
        if (MAX - port) <= num_threads {
            break;
        }
        port += num_threads;
    }
}

fn main() {
    // first argument is the path to the executable
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let arguments = Args::new(&args).unwrap_or_else(|err| {
        if err.contains("help") {
            // we return 0 because we want to exit the program
            println!("{} help", program);
            process::exit(0);
        }

        // we return 1 because we want to exit the program
        eprintln!("{} problem parsing arguments: {}", program, err);
        process::exit(1);
    });

    let num_threads = arguments.threads;
    let ipaddr = arguments.ipaddr;
    let (tx, rx) = channel();
    for i in 0..num_threads {
        // we clone the transmitter because we want to send it to a new thread
        let tx = tx.clone();

        // we spawn a new thread
        thread::spawn(move || {
            // we use unwrap because we know that the IP address is valid
            scan(tx, i, ipaddr, num_threads);
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
