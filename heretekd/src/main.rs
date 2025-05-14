use chrono::Local;
use heretek_config::load_config;
use heretekd::{MockServer, ProxmoxVersion};
use std::env;
use std::error::Error;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

fn is_proxmox() -> bool {
    Path::new("/etc/pve").exists()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = load_config();
    let bind_address = format!("{}:{}", cfg.heretekd.host, cfg.heretekd.port);
    let ctl_address = format!("{}:{}", cfg.heretekctl.host, cfg.heretekctl.port);

    let listener = TcpListener::bind(&bind_address).await?;
    println!("heretekd: SSH mock luistert op {bind_address} (om te stoppen: gebruik Ctrl+C of kill <PID>)");

    // Lees versie uit command line arguments, anders uit configuratie
    let args: Vec<String> = env::args().collect();
    let cmd_version = args
        .iter()
        .position(|arg| arg == "--version" || arg == "-v")
        .and_then(|i| args.get(i + 1))
        .map(|v| v.as_str());

    let version = if let Some(ver_str) = cmd_version {
        match ver_str {
            "6" => ProxmoxVersion::V6,
            "7" => ProxmoxVersion::V7,
            "8" => ProxmoxVersion::V8,
            _ => {
                eprintln!(
                    "⚠️ Ongeldige versie: {}. Geldige waarden zijn 6, 7 of 8.",
                    ver_str
                );
                return Ok(());
            }
        }
    } else if let Some(ver_str) = cfg.proxmox.version.as_deref() {
        match ver_str {
            "6" => ProxmoxVersion::V6,
            "7" => ProxmoxVersion::V7,
            _ => ProxmoxVersion::V8,
        }
    } else {
        ProxmoxVersion::V8
    };

    println!(
        "heretekd: Proxmox VE {:?} wordt gesimuleerd (om te stoppen: gebruik Ctrl+C of kill <PID>)",
        version
    );
    println!("heretekd: Gebruik '--version 6/7/8' om een specifieke versie te simuleren");
    println!(
        "heretekd: Gebruik 'heretek-setversion 6/7/8' om de versie tijdens runtime te wijzigen"
    );

    // Initialiseer MockServer één keer en hergebruik deze
    let mock: Arc<Mutex<MockServer>> = Arc::new(Mutex::new(MockServer::with_version(version)));
    println!("✅ Proxmox simulator gereed met versie {:?}", version);

    loop {
        let (mut socket, _addr) = listener.accept().await?;
        let ctl_address = ctl_address.clone();
        // Deel de MockServer instantie met de task
        let mock = Arc::clone(&mock);

        tokio::spawn(async move {
            let mut reader = BufReader::new(&mut socket);
            let mut line = String::new();

            match reader.read_line(&mut line).await {
                Ok(0) => {
                    println!("❌ Lege invoer");
                    return;
                }
                Ok(_) => {
                    let ts = Local::now().format("%Y-%m-%d %H:%M:%S");
                    println!("[{ts}] 📩 Ontvangen van SSH-client: {line}");
                }
                Err(e) => {
                    eprintln!("❌ Leesfout: {e}");
                    return;
                }
            }

            if is_proxmox() {
                match TcpStream::connect(&ctl_address).await {
                    Ok(mut ctl_stream) => {
                        if ctl_stream.write_all(line.as_bytes()).await.is_err() {
                            eprintln!("❌ Versturen naar heretekctl mislukt");
                            return;
                        }

                        let mut ctl_reader = BufReader::new(&mut ctl_stream);
                        let mut response = String::new();
                        if ctl_reader.read_line(&mut response).await.is_ok() {
                            println!("🔁 Antwoord van ctl: {response}");
                            if let Err(e) = socket.write_all(response.as_bytes()).await {
                                eprintln!("❌ Fout bij versturen antwoord naar client: {e}");
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Kan heretekctl niet bereiken: {e}");
                        if let Err(e) = socket
                            .write_all(b"[heretekd] heretekctl onbereikbaar\n")
                            .await
                        {
                            eprintln!("❌ Fout bij versturen foutmelding naar client: {e}");
                        }
                    }
                }
            } else {
                // Controleer en verwerk speciale heretek-commando's
                let response = if line.trim().starts_with("heretek-setversion ") {
                    // Extract versie nummer uit het commando
                    let ver_str = line.trim().split_whitespace().nth(1).unwrap_or("invalid");
                    let response = match ver_str {
                        "6" => {
                            if let Ok(mut server) = mock.lock() {
                                server.set_version(ProxmoxVersion::V6);
                                format!("✅ Proxmox versie succesvol gewijzigd naar V6\n")
                            } else {
                                "❌ Kon server niet vergrendelen om versie te wijzigen\n"
                                    .to_string()
                            }
                        }
                        "7" => {
                            if let Ok(mut server) = mock.lock() {
                                server.set_version(ProxmoxVersion::V7);
                                format!("✅ Proxmox versie succesvol gewijzigd naar V7\n")
                            } else {
                                "❌ Kon server niet vergrendelen om versie te wijzigen\n"
                                    .to_string()
                            }
                        }
                        "8" => {
                            if let Ok(mut server) = mock.lock() {
                                server.set_version(ProxmoxVersion::V8);
                                format!("✅ Proxmox versie succesvol gewijzigd naar V8\n")
                            } else {
                                "❌ Kon server niet vergrendelen om versie te wijzigen\n"
                                    .to_string()
                            }
                        }
                        _ => format!("❌ Ongeldige versie: {}. Gebruik 6, 7 of 8.\n", ver_str),
                    };
                    response
                } else {
                    // Normale commando's verwerken
                    if let Ok(server) = mock.lock() {
                        server.handle_command(&line)
                    } else {
                        "❌ Kon server niet vergrendelen om commando te verwerken\n".to_string()
                    }
                };

                if let Err(e) = socket.write_all(response.as_bytes()).await {
                    eprintln!("❌ Fout bij versturen mock-antwoord naar client: {e}");
                }
            }
        });
    }
}
