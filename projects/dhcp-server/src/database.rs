use std::net::Ipv4Addr;

use rusqlite::{Connection, NO_PARAMS, Rows, params};

pub fn select_addresses(con: &Connection, deleted: Option<u8>) -> Result<Vec<Ipv4Addr>, failure::Error> {
    if let Some(deleted) = deleted {
        let mut statement = con.prepare("SELECT ip_addr FROM lease_entries WHERE deleted = ?")?;
        let ip_addrs = statement.query(params![deleted.to_string()])?;
        get_addresses_from_row(ip_addrs)
    } else {
        let mut statement = con.prepare("SELECT ip_addr FROM lease_entries")?;
        let ip_addrs = statement.query(NO_PARAMS)?;
        get_addresses_from_row(ip_addrs)
    }
}

fn get_addresses_from_row(mut ip_addrs: Rows) -> Result<Vec<Ipv4Addr>, failure::Error> {
    let mut leased_addrs: Vec<Ipv4Addr> = Vec::new();
    while let Some(entry) = ip_addrs.next()? {
        let ip_addr = match entry.get(0) {
            Ok(ip) => {
                let ip: String = ip;
                ip.parse()?
            },
            Err(_) => continue,
        };
        leased_addrs.push(ip_addr);
    }
    Ok(leased_addrs)
}