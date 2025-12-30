# [Pod] Section

The `[Pod]` section describes a Podman pod that will be run as a systemd service using Podman Quadlet. A pod is a group of one or more containers that share resources like network namespaces, making them operate as a cohesive unit. Pod units use the `.pod` file extension and are automatically converted into systemd service units.

*Based on [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) official documentation.*

## Basic Configuration

### PodName=
Sets a custom name for the Podman pod instead of using the default.

**Default:** `systemd-%N` (service name with `systemd-` prefix)

**Note:** Cannot conflict with container names

### ServiceName=
Overrides the default systemd service unit naming; prevents appending `-pod` suffix.

**Format:** Unit name without `.service` extension

**Default:** Quadlet appends `-pod` to unit name

## Networking

### Network=
Specifies custom network for the pod.

**Format:**
- Network name
- `host` - Use host networking
- `none` - No networking
- `.network` suffix - Reference to Quadlet network unit (auto-generates service dependency)

**Multiple:** Yes

**Example:**
- `Network=host`
- `Network=mynetwork.network`

### PublishPort=
Exposes port(s) from pod to host; supports ranges.

**Format:**
- `containerPort` (e.g., `80`)
- `hostPort:containerPort` (e.g., `8080:80`)
- `ip:hostPort:containerPort` (e.g., `127.0.0.1:8080:80`)
- `ip::containerPort` (dynamic host port)
- Port ranges (e.g., `50-59`)

**Multiple:** Yes

**Note:** Host port auto-selection differs per invocation; find with `podman port` command

**Constraint:** Cannot use with `Network=host`

**Example:** `PublishPort=8080:80`

### IP=
Specifies static IPv4 address for the pod.

**Format:** IPv4 address

**Example:** `IP=10.88.64.128`

### IP6=
Specifies static IPv6 address for the pod.

**Format:** IPv6 address

**Example:** `IP6=fd46:db93:aa76:ac37::10`

### NetworkAlias=
Adds network-scoped alias for the pod; useful for DNS resolution grouping.

**Multiple:** Yes

### HostName=
Sets the pod's hostname inside all containers; adds hostname to /etc/hosts.

**Multiple:** Yes

### DNS=
Sets network-scoped DNS resolver or nameserver for containers within the pod.

**Multiple:** Yes

**Example:** `DNS=8.8.8.8`

### DNSSearch=
Establishes custom DNS search domains.

**Format:** Domain name or `.` to remove search domain

**Multiple:** Yes

### DNSOption=
Configures custom DNS options for the pod.

**Multiple:** Yes

**Example:** `DNSOption=ndots:1`

### AddHost=
Adds host-to-IP mappings to the pod's /etc/hosts file.

**Format:** `hostname:ip`

**Multiple:** Yes

**Example:** `AddHost=db.local:192.168.1.10`

## Storage

### Volume=
Mounts volume in the pod.

**Format:** `[[SOURCE-VOLUME|HOST-DIR:]CONTAINER-DIR[:OPTIONS]]`

**Special Case:** If SOURCE-VOLUME ends with `.volume`, uses corresponding Quadlet unit with auto-generated service dependency

**Relative Path Handling:** Paths starting with `.` resolve relative to unit file location

**Multiple:** Yes

**Example:**
- `Volume=/srv/data:/data:Z`
- `Volume=myvolume.volume:/data`

### ShmSize=
Sets the size of /dev/shm for the pod.

**Format:** `number[unit]`

**Example:** `ShmSize=100m`

## User Namespaces

### UserNS=
Sets user namespace mode for the pod.

**Format:** `MODE[:OPTIONS,...]`

**Example:** `UserNS=keep-id:uid=200,gid=210`

### UIDMap=
Creates pod in new user namespace using supplied UID mapping.

**Format:** `container_uid:host_uid:range`

**Multiple:** Yes

**Example:** `UIDMap=0:10000:10`

### GIDMap=
Creates pod in new user namespace using supplied GID mapping.

**Format:** `container_gid:host_gid:range`

**Multiple:** Yes

**Example:** `GIDMap=0:10000:10`

### SubUIDMap=
Creates pod in new user namespace using named map from /etc/subuid file.

**Format:** Map name

### SubGIDMap=
Creates pod in new user namespace using named map from /etc/subgid file.

**Format:** Map name

## Metadata and Labels

### Label=
Sets one or more OCI labels on the pod.

**Format:** `key=value` items, similar to Environment variable format

**Multiple:** Yes

**Example:** `Label=version=1.0 app=myapp`

## Lifecycle Management

### ExitPolicy=
Determines pod behavior when the last container exits.

**Values:**
- `stop` - Stop the pod (default for Quadlets)
- `continue` - Keep pod active

**Default:** `stop`

### StopTimeout=
Specifies time in seconds for graceful pod shutdown; containers forcibly killed after period expires.

**Format:** Numeric seconds

**Note:** Should be lower than systemd unit timeout to prevent interruption

## Advanced Options

### ContainersConfModule=
Loads a specified containers.conf(5) module for the pod.

**Multiple:** Yes

### GlobalArgs=
Contains arguments passed directly between `podman` and `pod` in generated files.

**Format:** Space-separated list, optionally individually escaped

**Note:** Not recommended; use for unsupported features only

**Multiple:** Yes

### PodmanArgs=
Contains arguments passed directly to end of `podman pod create` command.

**Format:** Space-separated list, optionally individually escaped

**Note:** Not recommended; use for unsupported features only

**Multiple:** Yes

## Example

```ini
[Unit]
Description=Application pod
After=network-online.target
Wants=network-online.target

[Pod]
PodName=myapp-pod
Network=mynetwork.network
PublishPort=8080:80
PublishPort=8443:443
Volume=/srv/data:/data:Z
DNS=8.8.8.8
DNSSearch=example.com
Label=app=myapp version=1.0

[Service]
Restart=always

[Install]
WantedBy=multi-user.target
```

## Using Pods with Containers

After creating a pod unit, reference it from container units using the `Pod=` directive:

```ini
# myapp.container
[Container]
Pod=myapp.pod
Image=docker.io/myapp:latest
```

## See Also

- [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html)
- [podman-pod(1)](https://docs.podman.io/en/latest/markdown/podman-pod.1.html)
- [podman-pod-create(1)](https://docs.podman.io/en/latest/markdown/podman-pod-create.1.html)
- systemd.unit(5)
- systemd.service(5)
