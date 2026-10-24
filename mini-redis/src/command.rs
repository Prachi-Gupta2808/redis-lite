#[derive(Debug, PartialEq)]
pub enum Command {
    Set { key: String, value: String, ttl_secs: Option<u64> },
    Get { key: String },
    Del { key: String },
    Exists { key: String },
    Ttl { key: String },
    Expire { key: String, seconds: u64 },
    Ping,
}

impl Command {
    pub fn parse(line: &str) -> Result<Command, String> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        let Some((name, args)) = parts.split_first() else {
            return Err("empty command".to_string());
        };

        match name.to_uppercase().as_str() {
            "PING" => Ok(Command::Ping),
            "GET" => Ok(Command::Get { key: one_key("GET", args)? }),
            "DEL" => Ok(Command::Del { key: one_key("DEL", args)? }),
            "EXISTS" => Ok(Command::Exists { key: one_key("EXISTS", args)? }),
            "TTL" => Ok(Command::Ttl { key: one_key("TTL", args)? }),
            "EXPIRE" => {
                if args.len() != 2 {
                    return Err("usage: EXPIRE key seconds".to_string());
                }
                let seconds = args[1]
                    .parse::<u64>()
                    .map_err(|_| "seconds must be a positive number".to_string())?;
                Ok(Command::Expire { key: args[0].to_string(), seconds })
            }
            "SET" => parse_set(args),
            other => Err(format!("unknown command '{other}'")),
        }
    }
}

fn one_key(cmd: &str, args: &[&str]) -> Result<String, String> {
    if args.len() != 1 {
        return Err(format!("usage: {cmd} key"));
    }
    Ok(args[0].to_string())
}

fn parse_set(args: &[&str]) -> Result<Command, String> {
    if args.len() < 2 {
        return Err("usage: SET key value [EX seconds]".to_string());
    }
    let key = args[0].to_string();

    // If the line ends with "EX <number>", it's the TTL.
    let n = args.len();
    if n >= 4 && args[n - 2].eq_ignore_ascii_case("EX") {
        let ttl = args[n - 1]
            .parse::<u64>()
            .map_err(|_| "EX needs a positive number of seconds".to_string())?;
        let value = args[1..n - 2].join(" ");
        return Ok(Command::Set { key, value, ttl_secs: Some(ttl) });
    }

    // Otherwise everything after the key is the value.
    let value = args[1..].join(" ");
    Ok(Command::Set { key, value, ttl_secs: None })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_simple_set() {
        assert_eq!(
            Command::parse("SET foo bar"),
            Ok(Command::Set { key: "foo".into(), value: "bar".into(), ttl_secs: None })
        );
    }

    #[test]
    fn parses_multiword_value_with_ttl() {
        assert_eq!(
            Command::parse("set name Prachi Gupta EX 10"),
            Ok(Command::Set { key: "name".into(), value: "Prachi Gupta".into(), ttl_secs: Some(10) })
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert!(Command::parse("").is_err());
        assert!(Command::parse("GET").is_err());
        assert!(Command::parse("FLY away").is_err());
        assert!(Command::parse("SET a b EX abc").is_err());
    }
}