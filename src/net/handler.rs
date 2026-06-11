// Network Packet Handler
// Routes packets through the network stack layers

use super::arp;
use super::config;
use super::ethernet::{EtherType, EthernetFrame, MacAddr};
use super::icmp;
use super::ipv4::{IpProtocol, Ipv4Addr, Ipv4Packet};
use super::tcp;
use super::udp;
use crate::drivers::e1000;

/// Process a received Ethernet frame
pub fn handle_rx_frame(frame: EthernetFrame) {
    // Check if frame is for us or broadcast
    let our_mac = config::get_mac();
    if frame.header.dest != our_mac && !frame.header.dest.is_broadcast() {
        return; // Not for us
    }

    match frame.header.ethertype {
        EtherType::Arp => {
            // Handle ARP
            let our_ip = config::get_ip();
            if let Some(reply_frame) = arp::handle_arp(&frame, our_mac, our_ip) {
                // Send ARP reply
                if let Err(e) = e1000::send_frame(&reply_frame) {
                    crate::println!("Failed to send ARP reply: {}", e);
                }
            }
        }
        EtherType::Ipv4 => {
            // Handle IPv4
            if let Some(ip_packet) = Ipv4Packet::parse(frame.payload.as_slice()) {
                handle_ip_packet(ip_packet);
            }
        }
        _ => {
            crate::println!("Unknown EtherType: {:?}", frame.header.ethertype);
        }
    }
}

/// Process an IPv4 packet
fn handle_ip_packet(packet: Ipv4Packet) {
    // Check if packet is for us
    let our_ip = config::get_ip();
    if packet.header.dest != our_ip && !packet.header.dest.is_broadcast() {
        return; // Not for us
    }

    match packet.header.protocol {
        IpProtocol::Icmp => {
            // Handle ICMP (ping, etc.)
            if let Some(reply_packet) = icmp::handle_icmp(&packet) {
                send_ip_packet(reply_packet);
            }
        }
        IpProtocol::Tcp => {
            // Handle TCP
            tcp::handle_tcp(&packet);
        }
        IpProtocol::Udp => {
            // Handle UDP
            udp::handle_udp(&packet);
        }
        _ => {
            crate::println!("Unknown IP protocol: {:?}", packet.header.protocol);
        }
    }
}

/// Send an IPv4 packet (handles ARP resolution and Ethernet framing)
pub fn send_ip_packet(packet: Ipv4Packet) {
    let config = config::get_config();
    let dest_ip = config.next_hop(packet.header.dest);

    // Look up destination MAC address
    let dest_mac = match arp::lookup(dest_ip) {
        Some(mac) => mac,
        None => {
            // Send ARP request and queue packet for later
            crate::println!("ARP: Need to resolve {} - sending request", dest_ip);
            let arp_request = arp::send_request(config.mac_addr, config.ip_addr, dest_ip);

            if let Err(e) = e1000::send_frame(&arp_request) {
                crate::println!("Failed to send ARP request: {}", e);
            }

            // For now, just drop the packet
            // TODO: Queue packet and retry after ARP response
            crate::println!("Packet dropped - waiting for ARP response");
            return;
        }
    };

    // Create Ethernet frame
    let frame = EthernetFrame::new(
        dest_mac,
        config.mac_addr,
        EtherType::Ipv4,
        packet.to_bytes(),
    );

    // Send via driver
    if let Err(e) = e1000::send_frame(&frame) {
        crate::println!("Failed to send IP packet: {}", e);
    }
}

/// Poll for received packets
pub fn poll_rx() {
    while let Some(frame) = e1000::poll_receive() {
        handle_rx_frame(frame);
    }
}

/// Send a ping (ICMP echo request)
pub fn send_ping(dest_ip: Ipv4Addr, identifier: u16, sequence: u16) {
    let src_ip = config::get_ip();

    if src_ip == Ipv4Addr::ZERO {
        crate::println!("Error: Network not configured. Use 'ifconfig' first.");
        return;
    }

    let ping_packet = icmp::send_ping(src_ip, dest_ip, identifier, sequence);
    send_ip_packet(ping_packet);

    crate::println!("PING {} ({}) 56 data bytes", dest_ip, dest_ip);
}
