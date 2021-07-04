This repository centers around the rust binary project `fand`, whose main
purpose is to run a control loop for a cooling fan on a Raspberry Pi 4. As such,
the binary is oriented to this problem in particular, however future development
should be oriented on making this a general lightweight control loop daemon.

On the other hand, this repository also contains library code which could in the
future be spun up as its own crate to implement control loops in general. This
will require some design work to better understand what a good API for potential
users of the library might be. As it stands now, the API is very inflexible,
unfriendly and quirky.

## How to use

### Compile it with cargo

```sh
cargo build --release
```

### RaspberryPi 4 specific
To be able to use the PWM output (which is the default), one needs to follow the
instructions [here](https://docs.golemparts.com/rppal/0.12.0/rppal/pwm/).

### Running the binary

```sh
./fand --help
```
The binary can take a config file ([see example](fand.conf)) or use the default
pipeline. To see all available operations and their parameters refer to either
the documentation (which you can compile with `cargo doc`) or to
[parameter.rs](src/operations/parameters.rs).

### Running with systemd

The intention of this software is to be run as a daemon; this is easy with
systemd, simply create a `fand.service` file in `/etc/systemd/system/` (or the
corresponding directory in your distro) with the following contents:

```sh
[Unit]
Description=Fan speed controller daemon
ConditionPathExists=/path/to/fand
After=network.target
StartLimitIntervalSec=0

[Service]
Type=simple
Restart=always
RestartSec=1
User=fand
ExecStart=/path/to/fand -s /tmp/fand.socket

[Install]
WantedBy=multi-user.target
```

Be sure to replace the path to `fand`, and if you don't need to use the unix
socket (see below) you can remove the `-s /path/to/socket` part of the command.

### Retrieving current state

This repo also contains two helper/debugging binaries, `fan-cli` and
`fan-get-out`, which can be used to get the internal state of the control loop
by connecting to a socket created by `fand`. The first of these two will print
out all the internal updates of the different operations, while the second one
will, *when using the default configuration*, print the current control loop
output. You are welcome to check the code of these two binaries to possibly
design your own to retrieve any piece of information you would want.
