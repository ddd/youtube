use hyper_tls::native_tls;
use thiserror::Error;
use std::net::{IpAddr, Ipv6Addr};
use rand::Rng;


#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Network error: {0}")]
    NetworkError(#[from] std::io::Error),
    #[error("TLS error: {0}")]
    TlsError(#[from] native_tls::Error),
}

pub fn get_rand_ipv6(subnet: &str, range_id: u16) -> Result<IpAddr, Box<dyn std::error::Error>> {
    // Split the subnet string into address and prefix length
    let parts: Vec<&str> = subnet.split('/').collect();
    if parts.len() != 2 {
        return Err("Invalid subnet format".into());
    }

    // Parse the IPv6 address
    let ipv6: u128 = parts[0].parse::<Ipv6Addr>()?.into();

    // Parse the prefix length
    let prefix_len: u8 = parts[1].parse()?;
    if prefix_len != 48 {
        return Err("Only /48 subnets are supported".into());
    }

    // Clear the lower 80 bits (128 - 48) of the network address
    let net_part = (ipv6 >> 80) << 80;
    
    // Shift the range_id into position (place it in bits 64-79)
    let range_part = (range_id as u128) << 64;
    
    // Generate random number for host portion (lower 64 bits)
    let rand: u128 = rand::thread_rng().gen();
    let host_part = rand & ((1u128 << 64) - 1);  // Only keep lower 64 bits
    
    // Combine all parts
    let result = net_part | range_part | host_part;

    Ok(IpAddr::V6(result.into()))
}