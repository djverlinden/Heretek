/// Versies van Proxmox VE
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProxmoxVersion {
    /// Proxmox VE 6.x
    V6,
    /// Proxmox VE 7.x
    V7,
    /// Proxmox VE 8.x
    V8,
}

impl ProxmoxVersion {
    /// Controleert of een specifiek commando beschikbaar is in deze versie
    pub fn supports_command(&self, command: &str) -> bool {
        match self {
            // Proxmox VE 6.x ondersteunt niet alle commando's
            ProxmoxVersion::V6 => {
                !command.contains("qm clone") && 
                !command.contains("pct rollback") &&
                !command.starts_with("pvecm nodes")
            },
            // Proxmox VE 7.x en hoger ondersteunt alle commando's
            _ => true,
        }
    }
}

pub struct MockServer {
    /// De versie van Proxmox die wordt gesimuleerd
    version: ProxmoxVersion,
}

impl MockServer {
    pub fn new() -> Self {
        // Standaard de nieuwste versie (Proxmox VE 8)
        MockServer {
            version: ProxmoxVersion::V8,
        }
    }
    
    /// Maak een nieuwe MockServer met een specifieke Proxmox-versie
    pub fn with_version(version: ProxmoxVersion) -> Self {
        MockServer {
            version,
        }
    }
    
    /// Geeft de huidige gesimuleerde Proxmox-versie
    pub fn get_version(&self) -> ProxmoxVersion {
        self.version
    }
    
    /// Wijzigt de gesimuleerde Proxmox-versie
    pub fn set_version(&mut self, version: ProxmoxVersion) {
        self.version = version;
    }

    pub fn handle_command(&self, input: &str) -> String {
        let trimmed = input.trim_end().trim_start_matches(|c| c == '\r' || c == '\n');

        // Commando en eventuele argumenten splitsen
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let command = parts.get(0).unwrap_or(&"");
        
        // Check Proxmox versie info command
        if trimmed == "pveversion" || trimmed == "pveversion -v" {
            let detail = if trimmed.ends_with("-v") { "verbose" } else { "short" };
            return self.get_version_info(detail);
        }
        
        // Controleer of het commando ondersteund wordt in deze Proxmox-versie
        if !self.version.supports_command(trimmed) {
            return format!("ERROR: Commando '{}' is niet beschikbaar in Proxmox VE {:?}\n", 
                          trimmed, self.version);
        }
        
        match trimmed {
            // Container management (pct)
            "pct list" => {
                "VMID       Status     Name\n100        running    test-container\n101        stopped    debian-11\n102        running    ubuntu-lts\n".to_string()
            },
            "pct start 100" => {
                "starting container 100...\nstarted successfully\n".to_string()
            },
            "pct start 101" => {
                "starting container 101...\nstarted successfully\n".to_string()
            },
            "pct stop 100" => {
                "stopping container 100...\nstopped successfully\n".to_string()
            },
            "pct stop 101" => {
                "stopping container 101...\nstopped successfully\n".to_string()
            },
            "pct create 103" => {
                "creating container 103...\ncreated successfully\n".to_string()
            },
            "pct destroy 103" => {
                "destroying container 103...\ndestroyed successfully\n".to_string()
            },
            "pct snapshot 100 snap1" => {
                "creating snapshot snap1 of container 100...\nsnapshot created successfully\n".to_string()
            },
            "pct rollback 100 snap1" => {
                "rolling back container 100 to snapshot snap1...\nrollback successful\n".to_string()
            },
            "pct migrate 100 node2" => {
                "migrating container 100 to node2...\nmigration successful\n".to_string()
            },
            
            // VM management (qm)
            "qm list" => {
                "VMID       NAME                 STATUS     MEM(MB)    BOOTDISK(GB) PID\n200        debian-vm            running    2048        32            12345\n201        ubuntu-vm            stopped    4096        64            -\n".to_string()
            },
            "qm start 200" => {
                "starting virtual machine 200...\nstarted successfully\n".to_string()
            },
            "qm start 201" => {
                "starting virtual machine 201...\nstarted successfully\n".to_string()
            },
            "qm stop 200" => {
                "stopping virtual machine 200...\nstopped successfully\n".to_string()
            },
            "qm stop 201" => {
                "stopping virtual machine 201...\nstopped successfully\n".to_string()
            },
            "qm create 202" => {
                "creating virtual machine 202...\ncreated successfully\n".to_string()
            },
            "qm destroy 202" => {
                "destroying virtual machine 202...\ndestroyed successfully\n".to_string()
            },
            "qm snapshot 200 snap1" => {
                "creating snapshot snap1 of virtual machine 200...\nsnapshot created successfully\n".to_string()
            },
            "qm rollback 200 snap1" => {
                "rolling back virtual machine 200 to snapshot snap1...\nrollback successful\n".to_string()
            },
            "qm clone 200 203" => {
                "cloning virtual machine 200 to 203...\ncloning successful\n".to_string()
            },
            "qm migrate 200 node2" => {
                "migrating virtual machine 200 to node2...\nmigration successful\n".to_string()
            },
            
            // Storage management
            "zfs list" => {
                "NAME              USED  AVAIL  REFER  MOUNTPOINT\nrpool             3.50G   96.5G   384K  /rpool\nrpool/data        1.20G   95.3G   1.20G  /rpool/data\nrpool/vm-images   2.30G   94.2G   2.30G  /rpool/vm-images\n".to_string()
            },
            "zfs create rpool/test" => {
                "filesystem rpool/test created successfully\n".to_string()
            },
            "pvesm status" => {
                "Name             Type     Status           Total            Used        Available\nlocal             dir      active           458.4G          25.4G          433.0G\nlocal-lvm         lvm      active           458.4G          25.4G          433.0G\nlocal-zfs         zfs      active           458.4G          25.4G          433.0G\n".to_string()
            },
            "pvesm alloc local-zfs 204 10G" => {
                "allocated 10G storage on local-zfs for ID 204\n".to_string()
            },
            
            // Template management
            "pveam available" => {
                "system          ubuntu-20.04-standard_20.04-1_amd64.tar.gz\nsystem          debian-11-standard_11.0-1_amd64.tar.gz\nsystem          centos-8-default_8.0-1_amd64.tar.gz\n".to_string()
            },
            "pveam download local ubuntu-20.04-standard_20.04-1_amd64.tar.gz" => {
                "downloading template ubuntu-20.04-standard_20.04-1_amd64.tar.gz to local storage...\ndownload successful\n".to_string()
            },
            "pveam list" => {
                "local:vztmpl/ubuntu-20.04-standard_20.04-1_amd64.tar.gz  328MB\nlocal:vztmpl/debian-11-standard_11.0-1_amd64.tar.gz  275MB\n".to_string()
            },
            
            // Netwerk management
            "ifreload -a" => {
                "reloading network configuration...\nreload successful\n".to_string()
            },
            "ip addr" => {
                "1: lo: <LOOPBACK,UP,LOWER_UP> mtu 65536 qdisc noqueue state UNKNOWN\n    inet 127.0.0.1/8 scope host lo\n2: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500 qdisc pfifo_fast state UP\n    inet 192.168.1.100/24 brd 192.168.1.255 scope global eth0\n".to_string()
            },
            "ip route" => {
                "default via 192.168.1.1 dev eth0 proto static\n192.168.1.0/24 dev eth0 proto kernel scope link src 192.168.1.100\n".to_string()
            },
            
            // Cluster management
            "pvecm status" => {
                "Cluster information\n-------------------\nName:             pve-cluster\nConfig Version:   1\nTransport:        knet\nCryptokey:        aes256\nRedundant rings:  yes\n\nNodelist\n--------\nNodeid      Votes Name\n1               1 node1 (local)\n2               1 node2\n".to_string()
            },
            "pvecm nodes" => {
                "Membership information\n----------------------\nNodeid      Name\n1           node1 (local)\n2           node2\n".to_string()
            },
            
            // Backup en restore
            "vzdump 100" => {
                "starting backup of container 100...\nbackup successful: /var/lib/vz/dump/vzdump-lxc-100.tar.gz\n".to_string()
            },
            "qmrestore /var/lib/vz/dump/vzdump-qemu-200.vma.gz 205" => {
                "restoring virtual machine 200 to ID 205...\nrestore successful\n".to_string()
            },
            "pct restore 105 /var/lib/vz/dump/vzdump-lxc-100.tar.gz" => {
                "restoring container 100 to ID 105...\nrestore successful\n".to_string()
            },
            
            // Fallback voor onbekende commando's
            _ => {
                if parts.len() > 1 && (*command == "pct" || *command == "qm") {
                    let action = parts.get(1).unwrap_or(&"");
                    let id = parts.get(2).unwrap_or(&"");
                    
                    match (*command, *action) {
                        ("pct", "start") => {
                            format!("starting container {}...\nstarted successfully\n", id)
                        },
                        ("pct", "stop") => {
                            format!("stopping container {}...\nstopped successfully\n", id)
                        },
                        ("qm", "start") => {
                            format!("starting virtual machine {}...\nstarted successfully\n", id)
                        },
                        ("qm", "stop") => {
                            format!("stopping virtual machine {}...\nstopped successfully\n", id)
                        },
                        _ => format!("mock: onbekend commando '{}'\n", trimmed),
                    }
                } else {
                    format!("mock: onbekend commando '{}'\n", trimmed)
                }
            },
        }
    }
    
    /// Geeft versie-informatie terug
    fn get_version_info(&self, detail: &str) -> String {
        match self.version {
            ProxmoxVersion::V6 => {
                if detail == "verbose" {
                    "pve-manager/6.4-15/765d9ceb (running kernel: 5.4.143-1-pve)\n".to_string()
                } else {
                    "pve-manager/6.4-15 (running kernel: 5.4.143-1-pve)\n".to_string()
                }
            },
            ProxmoxVersion::V7 => {
                if detail == "verbose" {
                    "pve-manager/7.4-3/ac086726 (running kernel: 5.15.102-1-pve)\n".to_string()
                } else {
                    "pve-manager/7.4-3 (running kernel: 5.15.102-1-pve)\n".to_string()
                }
            },
            ProxmoxVersion::V8 => {
                if detail == "verbose" {
                    "pve-manager/8.0.4/b1a16e71 (running kernel: 6.2.16-3-pve)\n".to_string()
                } else {
                    "pve-manager/8.0.4 (running kernel: 6.2.16-3-pve)\n".to_string()
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_creation() {
        let default_mock = MockServer::new();
        assert_eq!(default_mock.get_version(), ProxmoxVersion::V8);
        
        let v7_mock = MockServer::with_version(ProxmoxVersion::V7);
        assert_eq!(v7_mock.get_version(), ProxmoxVersion::V7);
        
        let mut mock = MockServer::new();
        mock.set_version(ProxmoxVersion::V6);
        assert_eq!(mock.get_version(), ProxmoxVersion::V6);
    }
    
    #[test]
    fn test_version_info_command() {
        let v6_mock = MockServer::with_version(ProxmoxVersion::V6);
        let v7_mock = MockServer::with_version(ProxmoxVersion::V7);
        let v8_mock = MockServer::with_version(ProxmoxVersion::V8);
        
        let v6_info = v6_mock.handle_command("pveversion");
        let v7_info = v7_mock.handle_command("pveversion");
        let v8_info = v8_mock.handle_command("pveversion");
        
        assert!(v6_info.contains("pve-manager/6.4"));
        assert!(v7_info.contains("pve-manager/7.4"));
        assert!(v8_info.contains("pve-manager/8.0"));
    }
    
    #[test]
    fn test_version_compatibility() {
        let v6_mock = MockServer::with_version(ProxmoxVersion::V6);
        let v8_mock = MockServer::with_version(ProxmoxVersion::V8);
        
        // V6 ondersteunt bepaalde commando's niet
        let v6_result = v6_mock.handle_command("qm clone 200 203");
        assert!(v6_result.contains("niet beschikbaar in Proxmox VE V6"));
        
        // V8 ondersteunt alle commando's
        let v8_result = v8_mock.handle_command("qm clone 200 203");
        assert!(v8_result.contains("cloning successful"));
    }

    #[test]
    fn test_zfs_list_command() {
        let mock = MockServer::new();
        let response = mock.handle_command("zfs list");
        assert!(response.contains("rpool"));
        assert!(response.contains("USED"));
        assert!(response.ends_with("\n"));
    }

    #[test]
    fn test_pct_list_command() {
        let mock = MockServer::new();
        let response = mock.handle_command("pct list");
        assert!(response.contains("test-container"));
        assert!(response.contains("VMID"));
        assert!(response.ends_with("\n"));
    }

    #[test]
    fn test_pveam_available_command() {
        let mock = MockServer::new();
        let response = mock.handle_command("pveam available");
        assert!(response.contains("ubuntu-20.04-standard"));
        assert!(response.ends_with("\n"));
    }

    #[test]
    fn test_unknown_command() {
        let mock = MockServer::new();
        let response = mock.handle_command("onbekend commando");
        assert!(response.contains("mock: onbekend commando"));
        assert!(response.contains("onbekend commando"));
        assert!(response.ends_with("\n"));
    }

    #[test]
    fn test_command_with_newlines() {
        let mock = MockServer::new();
        let response = mock.handle_command("zfs list\n");
        assert!(response.contains("rpool"));
        
        let response_with_cr = mock.handle_command("\r\npct list");
        assert!(response_with_cr.contains("test-container"));
    }
    
    #[test]
    fn test_pct_start_command() {
        let mock = MockServer::new();
        let response = mock.handle_command("pct start 101");
        assert!(response.contains("starting container 101"));
        assert!(response.contains("started successfully"));
    }
    
    #[test]
    fn test_pct_stop_command() {
        let mock = MockServer::new();
        let response = mock.handle_command("pct stop 100");
        assert!(response.contains("stopping container 100"));
        assert!(response.contains("stopped successfully"));
    }
    
    #[test]
    fn test_qm_list_command() {
        let mock = MockServer::new();
        let response = mock.handle_command("qm list");
        assert!(response.contains("debian-vm"));
        assert!(response.contains("VMID"));
    }
    
    #[test]
    fn test_qm_start_command() {
        let mock = MockServer::new();
        let response = mock.handle_command("qm start 201");
        assert!(response.contains("starting virtual machine 201"));
        assert!(response.contains("started successfully"));
    }
    
    #[test]
    fn test_cluster_command() {
        let mock = MockServer::new();
        let response = mock.handle_command("pvecm status");
        assert!(response.contains("pve-cluster"));
        assert!(response.contains("node1"));
    }
    
    #[test]
    fn test_backup_command() {
        let mock = MockServer::new();
        let response = mock.handle_command("vzdump 100");
        assert!(response.contains("backup of container 100"));
        assert!(response.contains("backup successful"));
    }
    
    #[test]
    fn test_dynamic_command() {
        let mock = MockServer::new();
        // Test een niet-gedefinieerd maar wel ondersteund commando
        let response = mock.handle_command("pct start 999");
        assert!(response.contains("starting container 999"));
        assert!(response.contains("started successfully"));
    }
}
