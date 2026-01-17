# [Network] Section

Network files are named with a `.network` extension and contain a section `[Network]` describing the
named Podman network. The generated service is a one-time command that ensures that the network
exists on the host, creating it if needed.

By default, the Podman network has the same name as the unit, but with a `systemd-` prefix, i.e. for
a network file named `$NAME.network`, the generated Podman network is called `systemd-$NAME`, and
the generated service file is `$NAME-network.service`. The `NetworkName` option allows for
overriding this default name with a user-provided one.

Please note that stopping the corresponding service will not remove the podman network.
In addition, updating an existing network is not supported.
In order to update the network parameters you will first need to manually remove the podman network and then restart the service.

Using network units allows containers to depend on networks being automatically pre-created. This is
particularly interesting when using special options to control network creation, as Podman otherwise creates networks with the default options.

Valid options for `[Network]` are listed below:

*Based on [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) official documentation.*

### ContainersConfModule=

Load the specified containers.conf(5) module. Equivalent to the Podman `--module` option.

This key can be listed multiple times.

### DisableDNS=

If enabled, disables the DNS plugin for this network.

This is equivalent to the Podman `--disable-dns` option

### DNS=

Set network-scoped DNS resolver/nameserver for containers in this network.

This key can be listed multiple times.

### Driver=

Driver to manage the network. Currently `bridge`, `macvlan` and `ipvlan` are supported.

This is equivalent to the Podman `--driver` option

### Gateway=

Define a gateway for the subnet. If you want to provide a gateway address, you must also provide a subnet option.

This is equivalent to the Podman `--gateway` option

This key can be listed multiple times.

### GlobalArgs=

This key contains a list of arguments passed directly between `podman` and `network`
in the generated file. It can be used to access Podman features otherwise unsupported by the generator. Since the generator is unaware
of what unexpected interactions can be caused by these arguments, it is not recommended to use
this option.

The format of this is a space separated list of arguments, which can optionally be individually
escaped to allow inclusion of whitespace and other control characters.

This key can be listed multiple times.

### InterfaceName=

This option maps the _network\_interface_ option in the network config, see **podman network inspect**.
Depending on the driver, this can have different effects; for `bridge`, it uses the bridge interface name.
For `macvlan` and `ipvlan`, it is the parent device on the host. It is the same as `--opt parent=...`.

This is equivalent to the Podman `--interface-name` option.

### Internal=

Restrict external access of this network.

This is equivalent to the Podman `--internal` option

### IPAMDriver=

Set the ipam driver (IP Address Management Driver) for the network. Currently `host-local`, `dhcp` and `none` are supported.

This is equivalent to the Podman `--ipam-driver` option

### IPRange=

Allocate container IP from a range. The range must be a either a complete subnet in CIDR notation or be
in the `<startIP>-<endIP>` syntax which allows for a more flexible range compared to the CIDR subnet.
The ip-range option must be used with a subnet option.

This is equivalent to the Podman `--ip-range` option

This key can be listed multiple times.

### IPv6=

Enable IPv6 (Dual Stack) networking.

This is equivalent to the Podman `--ipv6` option

### Label=

Set one or more OCI labels on the network. The format is a list of
`key=value` items, similar to `Environment`.

This key can be listed multiple times.

### NetworkDeleteOnStop=

When set to `true` the network is deleted when the service is stopped

### NetworkName=

The (optional) name of the Podman network.
If this is not specified, the default value is the same name as the unit, but with a `systemd-` prefix,
i.e. a `$name.network` file creates a `systemd-$name` Podman network to avoid
conflicts with user-managed network.

### Options=

Set driver specific options.

This is equivalent to the Podman `--opt` option

### PodmanArgs=

This key contains a list of arguments passed directly to the end of the `podman network create` command
in the generated file (right before the name of the network in the command line). It can be used to
access Podman features otherwise unsupported by the generator. Since the generator is unaware
of what unexpected interactions can be caused by these arguments, is not recommended to use
this option.

The format of this is a space separated list of arguments, which can optionally be individually
escaped to allow inclusion of whitespace and other control characters.

This key can be listed multiple times.

### Subnet=

The subnet in CIDR notation.

This is equivalent to the Podman `--subnet` option

This key can be listed multiple times.

