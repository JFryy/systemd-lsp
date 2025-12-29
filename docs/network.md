# [Network] Section

The `[Network]` section describes a Podman network that will be created and managed by systemd using Podman Quadlet. Networks allow containers and pods to communicate with each other and control external network access. Network units use the `.network` file extension and are automatically converted into systemd service units.

*Based on [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) official documentation.*

## Basic Configuration

### NetworkName=
Specifies the optional Podman network name, overriding the default naming convention.

**Default:** `systemd-%N` (where %N is the unit filename without extension)

**Example:** `NetworkName=myapp-network`

### Driver=
Specifies the driver managing the network.

**Supported Drivers:**
- `bridge` (default)
- `macvlan`
- `ipvlan`

**Default:** `bridge`

**Example:** `Driver=bridge`

## Network Addressing

### Subnet=
Specifies the subnet in CIDR notation.

**Format:** CIDR notation (e.g., `10.88.0.0/16`)

**Multiple:** Yes

**Example:**
- `Subnet=10.88.0.0/16`
- `Subnet=fd00:dead:beef::/48` (IPv6)

### Gateway=
Defines a gateway for the subnet.

**Format:** IP address

**Requirement:** Requires a subnet to also be provided via the `Subnet=` directive

**Multiple:** Yes

**Example:** `Gateway=10.88.0.1`

### IPRange=
Allocates container IPs from a specified range.

**Format:**
- CIDR notation (e.g., `10.88.0.0/24`)
- Range syntax: `<startIP>-<endIP>`

**Requirement:** Must be used with `Subnet=` directive

**Multiple:** Yes

**Example:**
- `IPRange=10.88.0.0/24`
- `IPRange=10.88.0.10-10.88.0.100`

## IPv6 Support

### IPv6=
Enables IPv6 (Dual Stack) networking for the network.

**Format:** `true` or `false`

**Example:** `IPv6=true`

## DNS Configuration

### DNS=
Sets network-scoped DNS resolver or nameserver for containers operating within this network.

**Format:** IP address

**Multiple:** Yes

**Example:** `DNS=8.8.8.8`

### DisableDNS=
Disables the DNS plugin for the network when enabled.

**Format:** `true` or `false`

**Default:** `false`

**Example:** `DisableDNS=false`

## Network Isolation

### Internal=
Restricts external access to this network when enabled.

**Format:** `true` or `false`

**Default:** `false`

**Example:** `Internal=true`

### NetworkDeleteOnStop=
When enabled, the network is removed when the service stops.

**Format:** `true` or `false`

**Default:** `false`

**Example:** `NetworkDeleteOnStop=true`

## Advanced Configuration

### InterfaceName=
Maps the network interface option in network configuration.

**Driver-Specific Behavior:**
- **bridge:** Uses the bridge interface name
- **macvlan/ipvlan:** Designates the parent device

**Example:**
- `InterfaceName=br0` (bridge)
- `InterfaceName=eth0` (macvlan/ipvlan parent)

### IPAMDriver=
Sets the IP Address Management Driver for the network.

**Supported Options:**
- `host-local`
- `dhcp`
- `none`

**Example:** `IPAMDriver=host-local`

### Options=
Sets driver-specific options.

**Format:** Option string

**Example:** `Options=mtu=1500`

## Metadata

### Label=
Sets one or more OCI labels on the network.

**Format:** `key=value` format, similar to environment variables

**Multiple:** Yes

**Example:** `Label=environment=production tier=backend`

## Advanced Options

### ContainersConfModule=
Loads a specified containers.conf(5) module.

**Format:** Path to module file

**Multiple:** Yes

### GlobalArgs=
Contains arguments passed directly between `podman` and `network` in the generated file.

**Format:** Space-separated format with optional escaping

**Multiple:** Yes

**Note:** Not recommended for general use due to unpredictable interactions

### PodmanArgs=
Contains arguments passed directly to the end of the `podman network create` command, before the network name.

**Format:** Space-separated format with optional escaping

**Multiple:** Yes

**Note:** Not recommended for general use

## Example: Basic Bridge Network

```ini
[Unit]
Description=Application bridge network

[Network]
NetworkName=myapp-net
Driver=bridge
Subnet=10.88.0.0/16
Gateway=10.88.0.1
DNS=8.8.8.8
DNS=8.8.4.4
Label=app=myapp

[Install]
WantedBy=multi-user.target
```

## Example: Internal Network

```ini
[Unit]
Description=Internal database network

[Network]
NetworkName=db-internal
Driver=bridge
Subnet=172.20.0.0/16
Gateway=172.20.0.1
Internal=true
Label=tier=database security=internal

[Install]
WantedBy=multi-user.target
```

## Example: IPv6 Dual Stack Network

```ini
[Unit]
Description=Dual stack network

[Network]
NetworkName=dualstack-net
Driver=bridge
Subnet=10.89.0.0/16
Subnet=fd00:dead:beef::/48
Gateway=10.89.0.1
Gateway=fd00:dead:beef::1
IPv6=true

[Install]
WantedBy=multi-user.target
```

## Example: macvlan Network

```ini
[Unit]
Description=macvlan network on eth0

[Network]
NetworkName=macvlan-net
Driver=macvlan
InterfaceName=eth0
Subnet=192.168.1.0/24
Gateway=192.168.1.1
IPRange=192.168.1.200-192.168.1.250

[Install]
WantedBy=multi-user.target
```

## Using Networks in Containers and Pods

Reference network units from container or pod units:

```ini
# myapp.container
[Container]
Image=docker.io/myapp:latest
Network=myapp-net.network

# Or in a pod
# myapp.pod
[Pod]
Network=myapp-net.network
```

The LSP will automatically create service dependencies when using `.network` references.

## See Also

- [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html)
- [podman-network(1)](https://docs.podman.io/en/latest/markdown/podman-network.1.html)
- [podman-network-create(1)](https://docs.podman.io/en/latest/markdown/podman-network-create.1.html)
- systemd.unit(5)
