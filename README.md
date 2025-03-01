# dyndnsd

This is a simple daemon that runs in the background and checks for your public
IP regularily. If it changes, the service sets the corresponding DNS record. The
service supports both IPv4 and IPv6 addresses.

## Installation

Use the package manager [cargo](https://doc.rust-lang.org/cargo/) to install
dyndnsd.

```bash
cargo install dyndnsd
```

## Usage
### Config

`dyndnsd` expects to find a config file at `/etc/dyndnsd/config.toml`.

As you can see from the path, the configuration should be a toml file. A sample
could look like this:

```toml
zone = "example.com"
domain = "example.example.com"
ipv4 = true    # defaults to true
ipv6 = true    # defaults to false
interval = 15  # seconds, defaults to 60

# Or you can use RFC 2136 with TSIG
[dns_provider_config]
url = "udp://1.2.3.4:53"
key_name = "test"
key = "test"
algorithm = "hmac-sha256"
```

### Running

To run the service, just call the binary. You can optionally set the `RUST_LOG`
env var to configure the log level:

```bash
RUST_LOG=info dyndnsd
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

dyndnsd is cooperative non-violent software: you can use,
redistribute, and/or modify it under the terms of the CNPLv7+ as found in the
`LICENSE.md` file in the source code root directory or at
<https://git.pixie.town/thufie/npl-builder>.

dyndnsd comes with ABSOLUTELY NO WARRANTY, to the extent
permitted by applicable law.  See `LICENSE.md` for details.

[CNPLv7+](https://thufie.lain.haus/NPL.html)

## Attribution

This work is derived from
[cloudflare-ddns](https://github.com/zbrox/cloudflare-ddns), a commandline
utility fullfilling the same purpose. It's written by Rostislav Raykov
<z@zbrox.org> and available under the MIT license at the link above.

This fork has made major changes to the project, to the extent where most of
the code has been rewritten and the tool is quite different:
 - Runs as a service instead of in a cron job
 - Supports IPv6
 - Homegrown Cloudflare API client has been replaced with hickory-dns based rfc2136 updates
