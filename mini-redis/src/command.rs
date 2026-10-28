use std::fmt;

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

#[derive(Debug, PartialEq)]
pub enum CommandError {
    Empty,
    UnknownCommand(String),
    Usage(&'static str),
    InvalidSeconds,
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandError::Empty => write!(f, "empty command"),
            CommandError::UnknownCommand(name) => write!(f, "unknown command '{name}'"),
            CommandError::Usage(usage) => write!(f, "usage: {usage}"),
            CommandError::InvalidSeconds => write!(f, "seconds must be a positive number"),
        }
    }
}

impl std::error::Error for CommandError {}

impl Command {
    pub fn parse(line: &str) -> Result<Command, CommandError> {
        let parts: Vec<&str> = line.split_whitespace().collect();

        let Some((name, args)) = parts.split_first() else {
            return Err(CommandError::Empty);
        };

        match name.to_uppercase().as_str() {
            "PING" => Ok(Command::Ping),
            "GET" => Ok(Command::Get { key: one_key("GET key", args)? }),
            "DEL" => Ok(Command::Del { key: one_key("DEL key", args)? }),
            "EXISTS" => Ok(Command::Exists { key: one_key("EXISTS key", args)? }),
            "TTL" => Ok(Command::Ttl { key: one_key("TTL key", args)? }),
            "EXPIRE" => {
                if args.len() != 2 {
                    return Err(CommandError::Usage("EXPIRE key seconds"));
                }
                let seconds = args[1]
                    .parse::<u64>()
                    .map_err(|_| CommandError::InvalidSeconds)?;
                Ok(Command::Expire { key: args[0].to_string(), seconds })
            }
            "SET" => parse_set(args),
            other => Err(CommandError::UnknownCommand(other.to_string())),
        }
    }
}

fn one_key(usage: &'static str, args: &[&str]) -> Result<String, CommandError> {
    if args.len() != 1 {
        return Err(CommandError::Usage(usage));
    }
    Ok(args[0].to_string())
}

fn parse_set(args: &[&str]) -> Result<Command, CommandError> {
    if args.len() < 2 {
        return Err(CommandError::Usage("SET key value [EX seconds]"));
    }
    let key = args[0].to_string();

    // If the line ends with "EX <number>", it's the TTL.
    let n = args.len();
    if n >= 4 && args[n - 2].eq_ignore_ascii_case("EX") {
        let ttl = args[n - 1]
            .parse::<u64>()
            .map_err(|_| CommandError::InvalidSeconds)?;
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

    #[test]
    fn errors_have_the_right_kind() {
        assert_eq!(Command::parse(""), Err(CommandError::Empty));
        assert_eq!(
            Command::parse("FLY away"),
            Err(CommandError::UnknownCommand("FLY".into()))
        );
        assert_eq!(Command::parse("EXPIRE a x"), Err(CommandError::InvalidSeconds));
        assert_eq!(Command::parse("GET"), Err(CommandError::Usage("GET key")));
    }
}