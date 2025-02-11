use std::{
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

use anyhow::Result;
use chrono::{DateTime, Utc};
use winnow::{
    ascii::{digit1, space0, space1},
    combinator::{alt, delimited, separated},
    token::take_until,
    Parser,
};

#[allow(unused)]
#[derive(Debug, PartialEq)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Trace,
    Patch,
}

#[allow(unused)]
#[derive(Debug, PartialEq)]
enum HttpProtocol {
    HTTP1_0,
    HTTP1_1,
    HTTP2_0,
    HTTP3_0,
}

#[allow(unused)]
#[derive(Debug)]
struct NginxLog {
    addr: IpAddr,
    datetime: DateTime<Utc>,
    method: HttpMethod,
    url: String,
    protocol: HttpProtocol,
    status: u16,
    body_bytes: u64,
    referer: String,
    user_agent: String,
}

fn main() -> Result<()> {
    let s = r#"93.180.71.3 - - [17/May/2015:08:05:32 +0000] "GET /downloads/product_1 HTTP/1.1" 304 0 "-" "Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)""#;

    let nginx_log = parse_nginx_log(s).unwrap();

    println!("NginxLog: {nginx_log:#?}");

    Ok(())
}

fn parse_nginx_log(s: &str) -> winnow::Result<NginxLog> {
    let input = &mut (&*s);
    let ip = parse_ip(input)?;
    println!("ip: {:?}", ip);
    parse_ignored(input)?;
    parse_ignored(input)?;
    let datetime = parse_datetime(input)?;
    println!("datetime: {:?}", datetime);
    let (method, url, protocol) = parse_http(input)?;
    println!(
        "method: {:?}, url: {:?}, protocol: {:?}",
        method, url, protocol
    );
    let status = parse_status(input)?;
    println!("status: {:?}", status);
    let body_bytes = parse_body_bytes(input)?;
    println!("body_bytes: {:?}", body_bytes);
    let referer = parse_referer(input)?;
    println!("referer: {:?}", referer);
    let user_agent = parse_user_agent(input)?;
    println!("user_agent: {:?}", user_agent);
    Ok(NginxLog {
        addr: ip,
        datetime,
        method,
        url,
        protocol,
        status,
        body_bytes,
        referer,
        user_agent,
    })
}

fn parse_ip(s: &mut &str) -> winnow::Result<IpAddr> {
    let ret: Vec<u8> = separated(4, digit1.parse_to::<u8>(), '.').parse_next(s)?;
    space1(s)?;

    Ok(IpAddr::V4(Ipv4Addr::new(ret[0], ret[1], ret[2], ret[3])))
}

fn parse_ignored(s: &mut &str) -> winnow::Result<()> {
    let _ = "-".parse_next(s)?;
    space1(s)?;
    Ok(())
}

fn parse_datetime(s: &mut &str) -> winnow::Result<DateTime<Utc>> {
    let ret = delimited('[', take_until(1.., ']'), ']').parse_next(s)?;
    space1(s)?;
    Ok(DateTime::parse_from_str(ret, "%d/%b/%Y:%H:%M:%S %z")
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap())
}

fn parse_http(s: &mut &str) -> winnow::Result<(HttpMethod, String, HttpProtocol)> {
    let parser = (parse_method, parse_url, parse_protocol);
    let ret = delimited('"', parser, '"').parse_next(s)?;
    space1(s)?;
    Ok(ret)
}

fn parse_method(s: &mut &str) -> winnow::Result<HttpMethod> {
    let ret = alt((
        "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "CONNECT", "TRACE", "PATCH",
    ))
    .parse_to()
    .parse_next(s)?;
    space1(s)?;
    Ok(ret)
}

fn parse_protocol(s: &mut &str) -> winnow::Result<HttpProtocol> {
    let ret = alt(("HTTP/1.0", "HTTP/1.1", "HTTP/2.0", "HTTP/3.0"))
        .parse_to()
        .parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_url(s: &mut &str) -> winnow::Result<String> {
    let ret = take_until(1.., ' ').parse_next(s)?;
    space1(s)?;
    Ok(ret.to_string())
}

fn parse_status(s: &mut &str) -> winnow::Result<u16> {
    let ret = digit1.parse_to::<u16>().parse_next(s)?;
    space1(s)?;
    Ok(ret)
}

fn parse_body_bytes(s: &mut &str) -> winnow::Result<u64> {
    let ret = digit1.parse_to::<u64>().parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_referer(s: &mut &str) -> winnow::Result<String> {
    let ret = delimited('"', take_until(1.., '"'), '"').parse_next(s)?;
    space1(s)?;
    Ok(ret.to_string())
}

fn parse_user_agent(s: &mut &str) -> winnow::Result<String> {
    let ret = delimited('"', take_until(1.., '"'), '"').parse_next(s)?;
    space0(s)?;
    Ok(ret.to_string())
}

impl FromStr for HttpMethod {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "HEAD" => Ok(HttpMethod::Head),
            "OPTIONS" => Ok(HttpMethod::Options),
            "CONNECT" => Ok(HttpMethod::Connect),
            "TRACE" => Ok(HttpMethod::Trace),
            "PATCH" => Ok(HttpMethod::Patch),
            _ => Err(anyhow::anyhow!("Invalid HTTP method")),
        }
    }
}

impl FromStr for HttpProtocol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "HTTP/1.0" => Ok(HttpProtocol::HTTP1_0),
            "HTTP/1.1" => Ok(HttpProtocol::HTTP1_1),
            "HTTP/2.0" => Ok(HttpProtocol::HTTP2_0),
            "HTTP/3.0" => Ok(HttpProtocol::HTTP3_0),
            _ => Err(anyhow::anyhow!("Invalid HTTP protocol")),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    #[test]
    fn parse_ip_should_work() -> Result<()> {
        let mut s = "1.1.1.1 ";
        let ip = parse_ip(&mut s).unwrap();
        assert_eq!(s, "");
        assert_eq!(ip, IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)));
        Ok(())
    }

    #[test]
    fn parse_datetime_should_work() -> Result<()> {
        let mut s = "[17/May/2015:08:05:32 +0000] ";
        let dt = parse_datetime(&mut s).unwrap();
        assert_eq!(s, "");
        assert_eq!(dt, Utc.with_ymd_and_hms(2015, 5, 17, 8, 5, 32).unwrap());
        Ok(())
    }

    #[test]
    fn parse_http_should_work() -> Result<()> {
        let mut s = r#""GET /downloads/product_1 HTTP/1.1" "#;
        let (method, url, protocol) = parse_http(&mut s).unwrap();
        assert_eq!(s, "");
        assert_eq!(method, HttpMethod::Get);
        assert_eq!(url, "/downloads/product_1");
        assert_eq!(protocol, HttpProtocol::HTTP1_1);
        Ok(())
    }

    #[test]
    fn parse_method_should_work() -> Result<()> {
        let mut s = "GET ";
        let method = parse_method(&mut s).unwrap();
        assert_eq!(s, "");
        assert_eq!(method, HttpMethod::Get);
        Ok(())
    }

    #[test]
    fn parse_url_should_work() -> Result<()> {
        let mut s = "/downloads/product_1 ";
        let url = parse_url(&mut s).unwrap();
        assert_eq!(s, "");
        assert_eq!(url, "/downloads/product_1");
        Ok(())
    }

    #[test]
    fn parse_protocol_should_work() -> Result<()> {
        let mut s = "HTTP/1.1";
        let protocol = parse_protocol(&mut s).unwrap();
        assert_eq!(s, "");
        assert_eq!(protocol, HttpProtocol::HTTP1_1);
        Ok(())
    }
}
