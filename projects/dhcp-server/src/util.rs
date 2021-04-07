use std::{
    collections::HashMap,
    fs, io,
    net::{AddrParseError, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use byteorder::{BigEndian, WriteBytesExt};

pub const NETWORK_ADDR_KEY: &str = "NETWORK_ADDR";
pub const SUBNET_MASK_KEY: &str = "SUBNET_MASK";
pub const SERVER_IDENTIFIER_KEY: &str = "SERVER_IDENTIFIER";
pub const DEFAULT_GATEWAY_KEY: &str = "DEFAULT_GATEWAY";
pub const DNS_SERVER_KEY: &str = "DNS_SERVER";
pub const LEASE_TIME_KEY: &str = "LEASE_TIME";

pub fn load_env() -> HashMap<String, String> {
    return fs::read_to_string(".env")
        .expect("Failed to read .env file")
        .lines()
        .filter_map(|line| {
            let tokens = line.split("=").map(str::trim).collect::<Vec<_>>();
            match tokens.len() {
                2 => Some((tokens[0].to_string(), tokens[1].to_string())),
                _ => None,
            }
        })
        .collect();
}

pub fn obtain_static_addresses(
    env: &HashMap<String, String>,
) -> Result<HashMap<String, Ipv4Addr>, AddrParseError> {
    let mut map = HashMap::new();

    parse_and_insert(NETWORK_ADDR_KEY, env, &mut map)?;
    parse_and_insert(SUBNET_MASK_KEY, env, &mut map)?;
    parse_and_insert(SERVER_IDENTIFIER_KEY, env, &mut map)?;
    parse_and_insert(DEFAULT_GATEWAY_KEY, env, &mut map)?;
    parse_and_insert(DNS_SERVER_KEY, env, &mut map)?;

    Ok(map)
}

fn parse_and_insert(
    key: &str,
    env: &HashMap<String, String>,
    store: &mut HashMap<String, Ipv4Addr>,
) -> Result<(), AddrParseError> {
    let addr = get_and_parse_addr(key, env)?;
    store.insert(key.to_string(), addr);

    Ok(())
}

pub fn get_and_parse_addr<F>(
    key: &str,
    env: &HashMap<String, String>,
) -> Result<F, <F as FromStr>::Err>
where
    F: FromStr,
{
    env.get(key)
        .expect(format!("Missing {:?} entry", key).as_str())
        .parse::<F>()
}

pub fn make_big_endian_vec_from_u32(i: u32) -> Result<Vec<u8>, io::Error> {
    let mut v = Vec::new();
    v.write_u32::<BigEndian>(i)?;
    Ok(v)
}
