# Cloudflare DDNS service

This is a simple daemon that runs in the background and checks for your public
IP regularily. If it changes, the service sets the corresponding DNS record. The
service supports both IPv4 and IPv6 addresses.

## Installation

Use the package manager [cargo](https://doc.rust-lang.org/cargo/) to install
Cloudflare DDNS service.

```bash
cargo install cloudflare-ddns-service
```

## Usage
### Config

`cloudflare-ddns-service` expects to find a config file at
`/etc/cloudflare-ddns-service/config.toml`.

As you can see from the path, the configuration should be a toml file. A sample
could look like this:

```toml
api_token = "secretkey"
zone = "example.com"
domain = "example.example.com"
ipv4 = true    # defaults to true
ipv6 = true    # defaults to false
interval = 15  # seconds, defaults to 60
```

As you can see, we have a token here. This token needs to have access to at
least:
 - reading you account zones (for getting the zone ID from the zone name)
 - reading and writing to the DNS zone (for first fetching the records and then
   modifying them.

Aside of the token, you also have to prepare some DNS records before running
this: If you enabled IPv4 support, there needs to be a DNS `A` record for the
configured domain already, and if you enabled IPv6 support, you need a DNS
`AAAA` record set on the configured domain. The service will not create new
records, it just modifies existing records.

### Running

To run the service, just call the binary. You can optionally set the `RUST_LOG`
env var to configure the log level:

```bash
RUST_LOG=info cloudflare-ddns-service
```

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

## License

cloudflare-ddns-service is cooperative non-violent software: you can use,
redistribute, and/or modify it under the terms of the CNPLv7+ as found in the
`LICENSE.md` file in the source code root directory or at
<https://git.pixie.town/thufie/npl-builder>.

cloudflare-ddns-service comes with ABSOLUTELY NO WARRANTY, to the extent
permitted by applicable law.  See `LICENSE.md` for details.

[CNPLv7+](https://thufie.lain.haus/NPL.html)

## Attribution

This work is derived from
[cloudflare-ddns](https://github.com/zbrox/cloudflare-ddns), a commandline
utility fullfilling the same purpose. It's written by Rostislav Raykov
<z@zbrox.org> and available under the MIT license at the link above.

This fork has been made to severly refactor the utility (have it running
constantly instead of running it in cron, for supporting IPv6, and for not using
a homegrown Cloudflare API client but the library provided by Cloudflare
themselves). Due to the nature of these changes, I have not sent a PR, as it
makes this basically a separate tool and nearly all code has been rewritten.
