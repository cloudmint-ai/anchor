use crate::*;

#[derive(Debug, Default, Copy, Clone)]
pub enum HostProtocol {
    #[default]
    Http,
    Https,
}

impl std::fmt::Display for HostProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HostProtocol::Http => write!(f, "http"),
            HostProtocol::Https => write!(f, "https"),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub enum Host {
    #[default]
    NotSet,
    WithPort {
        protocol: HostProtocol,
        address: Option<String>,
        port: u16,
    },
    WithoutPort {
        protocol: HostProtocol,
        address: Option<String>,
    },
}

impl Host {
    pub fn from_string(host: String) -> Result<Self> {
        if host == "--" {
            return Ok(Host::NotSet);
        }

        let (protocol, rest) = if let Some(rest) = host.strip_prefix("http://") {
            (HostProtocol::Http, rest)
        } else if let Some(rest) = host.strip_prefix("https://") {
            (HostProtocol::Https, rest)
        } else {
            (HostProtocol::Http, host.as_str())
        };

        let parts: Vec<&str> = rest.split(':').collect();
        match parts.as_slice() {
            [address] => {
                if address.parse::<i64>().is_ok() {
                    Unexpected!("int address {} not supported", address)
                } else {
                    Ok(Host::WithoutPort {
                        protocol,
                        address: Some(address.parse()?),
                    })
                }
            }
            [address, port] => {
                if address.is_empty() {
                    Ok(Host::WithPort {
                        protocol,
                        address: None,
                        port: port.parse()?,
                    })
                } else {
                    Ok(Host::WithPort {
                        protocol,
                        address: Some(address.parse()?),
                        port: port.parse()?,
                    })
                }
            }
            _ => return Unexpected!("unexpected host string {host}"),
        }
    }

    pub fn for_service(&self) -> Result<String> {
        if let config::Host::WithPort {
            protocol,
            address,
            port,
        } = self
        {
            if let config::HostProtocol::Https = protocol {
                return Unexpected!("https not supported");
            } else {
                if let Some(address) = address {
                    Ok(format!("{}:{}", address, port))
                } else {
                    Ok(format!("0.0.0.0:{}", port))
                }
            }
        } else {
            Unexpected!("unexpected host {:?}", self)
        }
    }

    pub fn for_client(&self, path: &'static str) -> Result<String> {
        match self {
            Host::NotSet => Unexpected!("Host not set"),
            Host::WithPort {
                protocol,
                address,
                port,
            } => {
                if let Some(address) = address {
                    Ok(format!("{}://{}:{}{}", protocol, address, port, path))
                } else {
                    Ok(format!("{}://localhost:{}{}", protocol, port, path))
                }
            }
            Host::WithoutPort { protocol, address } => {
                if let Some(address) = address {
                    Ok(format!("{}://{}{}", protocol, address, path))
                } else {
                    Ok(format!("{}://localhost{}", protocol, path))
                }
            }
        }
    }
}

impl<'de> Deserialize<'de> for Host {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        Host::from_string(s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test::case]
    fn test_host_config() {
        #[derive(Debug, Default, Clone, Deserialize)]
        struct HostConfig {
            host: Host,
        }
        let toml_str = "host = \"0.0.0.0:7777\"";
        let config: HostConfig = toml::from_str(&toml_str)?;
        let _address: String = "0.0.0.0".to_owned();
        assert!(matches!(
            config.host,
            Host::WithPort {
                protocol: HostProtocol::Http,
                address: _address,
                port: 7777,
            }
        ));

        let toml_str = "host = \":7777\"";
        let config: HostConfig = toml::from_str(&toml_str)?;
        let _address: String = "0.0.0.0".to_owned();
        assert!(matches!(
            config.host.clone(),
            Host::WithPort {
                protocol: HostProtocol::Http,
                address: _address,
                port: 7777,
            }
        ));

        assert_eq!(config.host.for_service()?, "0.0.0.0:7777");
        assert_eq!(
            config.host.for_client("/aaaa")?,
            "http://localhost:7777/aaaa"
        );

        let toml_str = "host = \"7777\"";
        if let Ok(_) = toml::from_str::<HostConfig>(&toml_str) {
            assert!(false, "pure int not supported")
        }

        let toml_str = "host = \"192.168.9.231\"";
        let config: HostConfig = toml::from_str(&toml_str)?;
        let _address: String = "192.168.9.231".to_owned();
        assert!(matches!(
            config.host,
            Host::WithoutPort {
                protocol: HostProtocol::Http,
                address: _address,
            }
        ));

        let toml_str = "host = \"192.168.9.111:50000\"";
        let config: HostConfig = toml::from_str(&toml_str)?;
        let _address: String = "192.168.9.111".to_owned();
        assert!(matches!(
            config.host,
            Host::WithPort {
                protocol: HostProtocol::Http,
                address: _address,
                port: 50000
            }
        ));

        let toml_str = "host = \"https://detector:7777\"";
        let config: HostConfig = toml::from_str(&toml_str)?;
        let _address = "detector".to_string();

        assert!(matches!(
            config.host,
            Host::WithPort {
                protocol: HostProtocol::Https,
                address: _address,
                port: 7777
            }
        ));
    }
}
