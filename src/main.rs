use clap::Parser;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpListener, UdpSocket},
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, disable_help_flag = true)]
struct ListenArgs {
    #[clap(long, action = clap::ArgAction::HelpLong)]
    help: Option<bool>,
    #[arg(short, long, default_value = "0.0.0.0")]
    host: String,
    #[clap(long = "host:tcp", short = None, value_name = "TCP_HOST")]
    tcp_host: Option<String>,
    #[clap(long = "host:udp", short = None, value_name = "UDP_HOST")]
    udp_host: Option<String>,
    #[arg(short, long, default_value_t = 1234)]
    port: u16,
    #[clap(long = "port:tcp", short = None, value_name = "TCP_PORT")]
    tcp_port: Option<u16>,
    #[clap(long = "port:udp", short = None, value_name = "UDP_PORT")]
    udp_port: Option<u16>,
}

fn gen_msg(ip: &str, port: u16, protocol: &str) -> String {
    let _ = port;
    format!("[{}] Hello, {}!\n", protocol.to_uppercase(), ip)
}

#[tokio::main]
async fn main() {
    let args = ListenArgs::parse();
    let host = args.host;
    let port = args.port;
    let tcp_host = args.tcp_host.unwrap_or_else(|| host.clone());
    let udp_host = args.udp_host.unwrap_or_else(|| host.clone());
    let tcp_port = args.tcp_port.unwrap_or(port);
    let udp_port = args.udp_port.unwrap_or(port);

    // start tcp server
    let tcp_task = tokio::spawn(async move {
        if let Err(e) = tcp_server(&tcp_host, tcp_port).await {
            eprintln!("TCP server error: {}", e);
        }
    });

    // start udp server
    let udp_task = tokio::spawn(async move {
        if let Err(e) = udp_server(&udp_host, udp_port).await {
            eprintln!("UDP server error: {}", e);
        }
    });

    // wait for both servers to finish
    tokio::select! {
        _ = tcp_task => (),
        _ = udp_task => (),
    }
}

async fn tcp_server(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("{}:{}", host, port)).await?;
    println!("\x1b[34mTCP listening on {}:{}\x1b[0m", host, port);

    loop {
        let (mut socket, addr) = listener.accept().await?;
        let ip = addr.ip().to_string();
        let port = addr.port();

        tokio::spawn(async move {
            let msg = gen_msg(&ip, port, "tcp");
            if let Err(e) = socket.write_all(msg.as_bytes()).await {
                eprintln!("TCP write error: {}", e);
            } else {
                println!("* Accepted from {}:{}/tcp", ip, port);
            }
            // Explicitly shutdown the write half to close the connection gracefully
            if let Err(e) = socket.shutdown().await {
                eprintln!("TCP shutdown error: {}", e);
            }
        });
    }
}

async fn udp_server(host: &str, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let socket = UdpSocket::bind(format!("{}:{}", host, port)).await?;
    let mut buf = [0; 1024];
    println!("\x1b[34mUDP listening on {}:{}\x1b[0m", host, port);

    loop {
        match socket.recv_from(&mut buf).await {
            Ok((_, src)) => {
                let ip = src.ip().to_string();
                let port = src.port();
                let msg = gen_msg(&ip, port, "udp");
                if let Err(e) = socket.send_to(msg.as_bytes(), &src).await {
                    eprintln!("UDP send error: {}", e);
                } else {
                    println!("* Accepted from {}:{}/udp", ip, port);
                }
            }
            Err(e) => eprintln!("UDP recv error: {}", e),
        }
    }
}
