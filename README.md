# TransIP Dynamic DNS

This app can be used if TransIP is your DNS provider
and you want to update the IP address of an A record
based on the perceived external IP address. This is
for example handy when you have an ISP that does not
allocate a static IP address to you.

## Config

To config the app, it needs some environment variables.
These can be put into a `.env` file for convenience.

| Key              | example          | description                                                                            |
| ---------------- | ---------------- | -------------------------------------------------------------------------------------- |
| TRANSIP_KEY_PATH | /etc/transip.key | Location of your TransIP API Key file. You can retreive this from their admin console. |
| TRANSIP_DOMAIN   | mydomain.com     | The domain you registered with TransIP                                                 |
| TRANSIP_PREFIX   | example          | the DNS prefix you want to update. This would be for example.mydomain.com              |
| TRANSIP_LOGIN    | myusername       | Your TransIP username                                                                  |

## Running it

Assuming you have a `.env` file, and a `transip.key` in the same
folder, you can run

```
docker run \
    -v $PWD/.env:/app/.env \
    -v $PWD/transip.key:/app/transip.key \
    ghcr.io/alexnederlof/transip-dynamic-dns
```
