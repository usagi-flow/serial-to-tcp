# OpenRC script

## Usage

Copy this script:

```bash
cp serial-to-tcp /etc/init.d/
```

Enable autostart on boot by adding the service:

```bash
rc-update add serial-to-tcp
```

Start the script manually and/or check its status:

```bash
rc-service serial-to-tcp start
rc-service serial-to-tcp status
```

By default, logs will be stored in `/var/log/serial-to-tcp.log`:

```bash
tail -f /var/log/serial-to-tcp.log
```
