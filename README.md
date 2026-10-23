<p align="center">
  <a href="https://check-host.one"><img src="https://check-host.one/logo-96.png" width="72" alt="check-host.one"></a>
</p>

<h1 align="center">check-host.one Rust SDK</h1>

<p align="center">
  Official Rust client for <a href="https://check-host.one"><b>check-host.one</b></a>: run <b>ping</b>, <b>TCP ping</b>, <b>HTTP</b>, <b>traceroute</b> and <b>DNS</b> checks against any host from measurement servers in <b>100+ countries</b>, and look up ASN and geolocation.<br>
  No account, no API key.
</p>

<p align="center">
  <a href="https://check-host.one"><img src="https://img.shields.io/badge/website-check--host.one-2563eb" alt="Website"></a>
  <a href="https://check-host.one/docs#api"><img src="https://img.shields.io/badge/docs-API-0ea5e9" alt="API docs"></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-green" alt="MIT license"></a>
</p>

Use it to find out whether a site or server is **down for everyone or just you**, compare latency between regions, check that a port is reachable worldwide, or verify DNS propagation, from your own code, CI pipeline or monitoring script.

## Install

```toml
[dependencies]
check-host-one = { git = "https://github.com/Check-Host-one/check-host-rust" }
```

## Quick start

```rust
let ch = checkhost::Client::new();
let m = ch.ping("example.com", 5, &["DE", "US", "Tokyo"])?;
for r in m["results"].as_array().unwrap() {
    println!("{}, {}: avg {} ms", r["probe"]["city"], r["probe"]["country"], r["result"]["stats"]["avg"]);
}
println!("https://check-host.one/r/{}", m["resultSlug"].as_str().unwrap()); // shareable result page

ch.tcp_ping("example.com", 443, 5, &[])?;
ch.http("example.com", 5, &["Europe"])?;
ch.traceroute("example.com", 3, &[])?;
ch.dns("example.com", "MX", 5, &[])?;
ch.host_info(Some("1.1.1.1"))?; // ASN, operator, geolocation
```

Blocking client on `ureq` + `serde_json`; results are `serde_json::Value`. Errors are `checkhost::Error::Api { status, kind, message, retry_after }`.

## What you can check

| Method | What it does |
| --- | --- |
| ping | ICMP ping: min/avg/max latency and packet loss per server |
| TCP ping | Times TCP handshakes to a port, for hosts that block ICMP |
| HTTP | Status code and timings for an HTTP/HTTPS/HTTP2 request |
| traceroute | Hop-by-hop network path (ICMP, UDP or TCP) |
| DNS | A, AAAA, MX, TXT, NS, CNAME, SOA, PTR answers from each region |
| host info | ASN, network operator and geolocation of a host or IP |
| run | Any check with full `measurementOptions`, see the [OpenAPI spec](https://check-host.one/openapi.json) |

**Locations** are free-form strings: a country code (`DE`), a city (`Tokyo`), a continent (`Europe`), an ASN (`AS13335`) or a network name. Up to 25 servers per check.

Every finished check has a `resultSlug`, a permanent link at `https://check-host.one/r/<slug>` you can share or post in an incident channel. Saved results stay up for 90 days.

## Limits

About 20 checks per minute per client and 30 per minute against any one target. Over the limit the API answers 429 with a `Retry-After` header, which the SDK exposes on its error type.

## Test

```sh
cargo run --example smoke   # runs real checks against the live API
```

## Related

- 🌐 [check-host.one](https://check-host.one): run checks in the browser, with a live world map
- 🤖 [MCP server](https://github.com/Check-Host-one/check-host-mcp): let Claude, Cursor and other AI agents run network checks
- 📖 [API docs](https://check-host.one/docs#api) · [OpenAPI](https://check-host.one/openapi.json) · [llms.txt](https://check-host.one/llms.txt)
- Other SDKs: [Python](https://github.com/Check-Host-one/check-host-python) · [JavaScript / TypeScript](https://github.com/Check-Host-one/check-host-js) · [Go](https://github.com/Check-Host-one/check-host-go) · [Java](https://github.com/Check-Host-one/check-host-java) · [PHP](https://github.com/Check-Host-one/check-host-php) · [Ruby](https://github.com/Check-Host-one/check-host-ruby) · [C# / .NET](https://github.com/Check-Host-one/check-host-dotnet)

## License

MIT
