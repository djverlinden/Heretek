// Server module with Proxmox simulation functionality
//
// This module provides the ProxmoxServer for Proxmox command simulation
// and handles loading of command configurations from YAML files.

mod proxmox_server;

pub use proxmox_server::{
    ProxmoxServer,
    ProxmoxVersion,
    CommandConfig,
    CommandLoader,
    CommandMap,
    get_command_response,
};

// Alias MockServer naar ProxmoxServer voor backward compatibiliteit
pub use proxmox_server::ProxmoxServer as MockServer;