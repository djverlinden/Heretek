use chrono::Local;
use heretek_config::load_config;
use std::error::Error;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_command() {
        let response = handle_command("pct list").await;
        assert!(response.status);
        assert_eq!(response.content, "100 debian10 running");

        let response = handle_command("zfs list").await;
        assert!(response.status);
        assert_eq!(response.content, "rpool 1G 99G");

        let response = handle_command("invalid").await;
        assert!(!response.status);
        assert!(response.content.contains("Onbekend commando"));
    }

    #[tokio::test]
    async fn test_command_response() {
        let ok_response = CommandResponse::ok("test");
        assert!(ok_response.status);
        assert_eq!(ok_response.content, "test");

        let err_response = CommandResponse::error("error");
        assert!(!err_response.status);
        assert_eq!(err_response.content, "error");
    }

    #[test]
    fn test_log_format() {
        let msg = "test message";
        let emoji = "🔧";
        // We kunnen de exacte output niet testen vanwege de timestamp,
        // maar we kunnen wel controleren of de functie uitvoert zonder paniek
        log(emoji, msg);
    }

    // Uncomment deze test wanneer we een betere manier hebben om een client verbinding te simuleren
    // 
    // #[tokio::test]
    // async fn test_handle_client_error() {
    //     // Test client error handling
    // }
}

// Emoji constants voor consistente logging
const EMOJI_BRAIN: &str = "🧠";
const EMOJI_ANTENNA: &str = "📡";
const EMOJI_INBOX: &str = "📥";
const EMOJI_OUTBOX: &str = "📤";
const EMOJI_ERROR: &str = "❌";

/// Logt een bericht met timestamp en emoji
///
/// # Arguments
///
/// * `emoji` - Het emoji om voor het bericht te tonen
/// * `msg` - Het te loggen bericht
fn log(emoji: &str, msg: &str) {
    let now = Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("[{now}] {emoji} {msg}");
}

/// Representeert een antwoord op een commando met inhoud en status
#[derive(Debug)]
struct CommandResponse {
    /// De inhoud van het antwoord
    content: String,
    /// Of het commando succesvol was (true) of niet (false)
    status: bool,
}

impl CommandResponse {
    fn ok(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            status: true,
        }
    }

    fn error(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            status: false,
        }
    }
}

/// Verwerkt een commando en retourneert een gepast antwoord
///
/// # Arguments
///
/// * `command` - Het te verwerken commando
///
/// # Returns
///
/// Een CommandResponse met de inhoud en status van het antwoord
async fn handle_command(command: &str) -> CommandResponse {
    match command.trim() {
        "pct list" => CommandResponse::ok("100 debian10 running"),
        "zfs list" => CommandResponse::ok("rpool 1G 99G"),
        cmd => CommandResponse::error(format!("Onbekend commando: {}", cmd)),
    }
}

/// Verwerkt een client verbinding door commando's te lezen en antwoorden te versturen
///
/// # Arguments
///
/// * `socket` - De TCP stream van de client verbinding
/// * `addr` - Het IP-adres en poort van de client
async fn handle_client(
    socket: TcpStream,
    addr: std::net::SocketAddr,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    log(EMOJI_ANTENNA, &format!("Verbinding van {addr}"));
    
    let (reader, mut writer) = socket.into_split();
    let mut lines = BufReader::new(reader).lines();

    while let Some(command) = lines.next_line().await? {
        log(EMOJI_INBOX, &format!("Ontvangen: {command}"));
        
        let response = handle_command(&command).await;
        let response_text = format!("{}\n", response.content);
        
        writer.write_all(response_text.as_bytes()).await?;
        
        let status = if response.status { "OK" } else { "ERROR" };
        log(EMOJI_OUTBOX, &format!("Verzonden ({status}): {}", response.content));
    }

    Ok(())
}

/// De hoofdfunctie van de applicatie
///
/// Laadt de configuratie, start de server, en handelt client verbindingen af
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cfg = load_config();
    let bind_address = format!("{}:{}", cfg.heretekctl.host, cfg.heretekctl.port);

    let listener = TcpListener::bind(&bind_address).await?;
    log(EMOJI_BRAIN, &format!("heretekctl luistert op {bind_address} (om te stoppen: gebruik Ctrl+C of kill <PID>)"));

    loop {
        match listener.accept().await {
            Ok((socket, addr)) => {
                tokio::spawn(async move {
                    if let Err(e) = handle_client(socket, addr).await {
                        log(EMOJI_ERROR, &format!("Clientfout: {e}"));
                    }
                });
            }
            Err(e) => log(EMOJI_ERROR, &format!("Acceptatiefout: {e}")),
        }
    }
}
