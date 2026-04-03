use embassy_net::dns::Error;
use embassy_net::Stack;
use embassy_net::IpAddress;
use embassy_net::dns::DnsQueryType;
use log::{info, warn};

pub async fn nslookup(stack: &Stack<'static>, addr: &str) -> Result<IpAddress, Error> {
    match (*stack).dns_query(addr, DnsQueryType::A).await {
        Ok(response) => {
            info!("[WIFI_task]: Got response: {:?}", response);
            let addr: IpAddress = *response.get(0)
                .expect("DNS response is empty");
            Ok(addr)
        }
        Err(e) => {
            warn!("DNS lookup failed");
            Err(e)
        }
    }
}