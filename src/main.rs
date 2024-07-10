use anyhow::{Context, Result};
use serde::Serialize;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::time;
use clap::Parser;

mod system_info;



#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "127.0.0.1")]
    server: String,

    #[clap(short, long, default_value_t = 35601)]
    port: u16,

    #[clap(short, long, default_value = "114514")]
    user: String,

    #[clap( long, default_value = "114514")]
    password: String,

    #[clap(short, long, default_value_t = 1)]
    interval: u64,
}


#[derive(Serialize)]
struct ServerStatus {
    uptime: u64,
    load: f64,
    memory_total: u64,
    memory_used: u64,
    swap_total: u64,
    swap_used: u64,
    hdd_total: u64,
    hdd_used: u64,
    cpu: f64,
    network_rx: u64,
    network_tx: u64,
    network_in: u64,
    network_out: u64,
    online4: bool,
    online6: bool,
}

// #[tokio::main]
// async fn main() -> Result<()> {
//     let args = Args::parse();
//     let mut network = system_info::Network::new()?;

//     loop {
//         match connect_and_update(&mut network, &args).await {
//             Ok(_) => println!("Update successful"),
//             Err(e) => eprintln!("Error: {}", e),
//         }
//         time::sleep(Duration::from_secs(args.interval)).await;
//     }
// }

// async fn connect_and_update(network: &mut system_info::Network, args: &Args) -> Result<()> {
//     let mut stream = TcpStream::connect(format!("{}:{}", args.server, args.port))
//         .await
//         .context("Failed to connect to server")?;

//     // Authentication
//     let mut buffer = [0; 1024];
//     let n = stream.read(&mut buffer).await?;
//     if std::str::from_utf8(&buffer[..n])?.contains("Authentication required") {
//         stream.write_all(format!("{}:{}\n", args.user, args.password).as_bytes()).await?;
//         let n = stream.read(&mut buffer).await?;
//         if !std::str::from_utf8(&buffer[..n])?.contains("Authentication successful") {
//             anyhow::bail!("Authentication failed");
//         }
//     }

//     let (network_rx, network_tx) = network.get_speed()?;
//     let (network_in, network_out) = network.get_traffic()?;

//     let status = ServerStatus {
//         uptime: system_info::get_uptime()?,
//         load: system_info::get_load()?,
//         memory_total: system_info::get_memory()?.0,
//         memory_used: system_info::get_memory()?.1,
//         swap_total: system_info::get_memory()?.2,
//         swap_used: system_info::get_memory()?.3,
//         hdd_total: system_info::get_hdd()?.0,
//         hdd_used: system_info::get_hdd()?.1,
//         cpu: system_info::get_cpu().await?,
//         network_rx,
//         network_tx,
//         network_in,
//         network_out,
//         online4: system_info::get_network(4).await?,
//         online6: system_info::get_network(6).await?,
//     };

//     let json = serde_json::to_string(&status)?;
//     let message = format!("update {}\n", json);
//     stream.write_all(message.as_bytes()).await?;

//     Ok(())
// }

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let mut network = system_info::Network::new()?;

    // 连接并认证
    let mut stream = connect_and_authenticate(&args).await?;

    loop {
        match update_status(&mut stream, &mut network).await {
            Ok(_) => {},
            Err(e) => {
                eprintln!("Error: {}", e);
                // 如果出错，尝试重新连接
                stream = match connect_and_authenticate(&args).await {
                    Ok(new_stream) => new_stream,
                    Err(e) => {
                        eprintln!("Failed to reconnect: {}", e);
                        continue;
                    }
                };
            }
        }
        time::sleep(Duration::from_secs(args.interval)).await;
    }
}

async fn connect_and_authenticate(args: &Args) -> Result<TcpStream> {
    let mut stream = TcpStream::connect(format!("{}:{}", args.server, args.port))
        .await
        .context("Failed to connect to server")?;

    // Authentication
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    if std::str::from_utf8(&buffer[..n])?.contains("Authentication required") {
        stream.write_all(format!("{}:{}\n", args.user, args.password).as_bytes()).await?;
        let n = stream.read(&mut buffer).await?;
        if !std::str::from_utf8(&buffer[..n])?.contains("Authentication successful") {
            let s = String::from_utf8_lossy(&buffer[..n]).to_string();
            print!("{}",s);
            anyhow::bail!("Authentication failed");
        }
    }

    Ok(stream)
}

async fn update_status(stream: &mut TcpStream, network: &mut system_info::Network) -> Result<()> {
    let (network_rx, network_tx) = network.get_speed()?;
    let (network_in, network_out) = network.get_traffic()?;

    let status = ServerStatus {
        uptime: system_info::get_uptime()?,
        load: system_info::get_load()?,
        memory_total: system_info::get_memory()?.0,
        memory_used: system_info::get_memory()?.1,
        swap_total: system_info::get_memory()?.2,
        swap_used: system_info::get_memory()?.3,
        hdd_total: system_info::get_hdd()?.0,
        hdd_used: system_info::get_hdd()?.1,
        cpu: system_info::get_cpu().await?,
        network_rx,
        network_tx,
        network_in,
        network_out,
        online4: system_info::get_network(4).await?,
        online6: system_info::get_network(6).await?,
    };

    let json = serde_json::to_string(&status)?;
    let message = format!("update {}\n", json);
    stream.write_all(message.as_bytes()).await?;

    Ok(())
}