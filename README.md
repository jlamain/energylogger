# EnergyLogger

This program connects to a Homewizard P1 meter and logs the electricity and gas usage every 5 minutes.

To use this program you need a P1 meter (https://www.homewizard.com/nl/p1-meter/) connected to your network.

Build the program using:

```
cargo build
```

And run using:

```
target/debug/energylogger
```

The program will find your p1 meter using mdns. If it cant find your meter you can set your using:

```
target/debug/energylogger --ip <ip1.ip2.ip3.ip4>
```

For example

```
target/debug/energylogger --ip 192.168.45.77
```


The program, by default, will only log errors and warnings.
To enable more logs set the RUST_LOG environment variable, for example:
```
export RUST_LOG=info
```
