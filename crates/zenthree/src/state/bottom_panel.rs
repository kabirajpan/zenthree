use crate::state::{ListeningPort, ZenthreeState};
use std::collections::HashSet;
use std::process::Command;

impl ZenthreeState {
    pub fn refresh_listening_ports(&mut self) {
        let output = if cfg!(windows) {
            Command::new("netstat").args(["-ano", "-p", "tcp"]).output()
        } else {
            Command::new("ss")
                .args(["-ltnp"])
                .output()
                .or_else(|_| Command::new("netstat").args(["-lnt"]).output())
        };

        match output {
            Ok(output) if output.status.success() => {
                self.listening_ports =
                    parse_listening_ports(&String::from_utf8_lossy(&output.stdout));
                self.ports_error = None;
            }
            Ok(output) => {
                self.ports_error = Some(format!(
                    "Could not inspect listening ports: {}",
                    String::from_utf8_lossy(&output.stderr).trim()
                ));
            }
            Err(error) => {
                self.ports_error = Some(format!("Could not inspect listening ports: {}", error));
            }
        }
    }
}

fn parse_listening_ports(output: &str) -> Vec<ListeningPort> {
    let mut ports = Vec::new();
    let mut seen = HashSet::new();

    for line in output.lines() {
        let fields: Vec<_> = line.split_whitespace().collect();
        let Some(state_index) = fields.iter().position(|field| {
            field.eq_ignore_ascii_case("LISTENING") || field.eq_ignore_ascii_case("LISTEN")
        }) else {
            continue;
        };
        let local_index = if fields
            .first()
            .is_some_and(|protocol| protocol.eq_ignore_ascii_case("TCP"))
            && fields.get(state_index) == Some(&"LISTENING")
        {
            1
        } else {
            3
        };
        let Some(local_address) = fields.get(local_index) else {
            continue;
        };
        let Some((address, port)) = local_address.rsplit_once(':') else {
            continue;
        };
        let Ok(port) = port.parse::<u16>() else {
            continue;
        };

        let process_id = if fields.get(state_index) == Some(&"LISTENING") {
            fields.last().and_then(|field| field.parse().ok())
        } else {
            fields
                .iter()
                .find_map(|field| field.split("pid=").nth(1))
                .and_then(|tail| {
                    tail.chars()
                        .take_while(char::is_ascii_digit)
                        .collect::<String>()
                        .parse()
                        .ok()
                })
        };

        if seen.insert((address.to_string(), port)) {
            ports.push(ListeningPort {
                address: address.to_string(),
                port,
                process_id,
            });
        }
    }

    ports.sort_by_key(|port| (port.port, port.address.clone()));
    ports
}

#[cfg(test)]
mod tests {
    use super::parse_listening_ports;

    #[test]
    fn parses_windows_listening_ports_and_process_ids() {
        let ports =
            parse_listening_ports("  TCP    0.0.0.0:8080    0.0.0.0:0    LISTENING    4216\n");

        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].address, "0.0.0.0");
        assert_eq!(ports[0].port, 8080);
        assert_eq!(ports[0].process_id, Some(4216));
    }

    #[test]
    fn parses_ss_listening_ports() {
        let ports = parse_listening_ports(
            "LISTEN 0 4096 127.0.0.1:5173 0.0.0.0:* users:((\"node\",pid=812,fd=20))\n",
        );

        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].address, "127.0.0.1");
        assert_eq!(ports[0].port, 5173);
        assert_eq!(ports[0].process_id, Some(812));
    }
}
