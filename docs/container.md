# [Container] Section

The `[Container]` section describes the container that will be run as a systemd service using Podman Quadlet. Quadlet is a feature of Podman that allows you to manage containers declaratively through systemd unit files. Container units use the `.container` file extension and are automatically converted into systemd service units.

*Based on [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) official documentation.*

## Required Directives

### Image=
The container image to run. It is recommended to use a fully qualified image name rather than a short name, both for performance and robustness reasons.

**Format:** `registry.com/repository/image:tag` or image digest

**Example:** `Image=docker.io/nginx:latest`

**Note:** Supports references to `.image` or `.build` Quadlet files

## Basic Configuration

### ContainerName=
Specifies a custom name for the Podman container instead of using the default.

**Default:** `systemd-%N` (service name with `systemd-` prefix)

### Exec=
Additional arguments for the container; this has exactly the same effect as passing more arguments after a `podman run <image> <arguments>` invocation.

**Format:** Command line arguments matching systemd command syntax

### WorkingDir=
Overrides the default working directory for command execution inside the container.

**Default:** `/` (root directory, or the image's WORKDIR if set)

### Entrypoint=
Overrides the default ENTRYPOINT instruction from the container image.

**Format:** Command string or JSON string for multi-option commands

### User=
Specifies the numeric user ID (UID) or username for processes executing in the container.

**Note:** Can be combined with `Group=` to create `--user USER:GROUP` argument

### Group=
Sets the numeric GID for processes running within the container.

**Note:** Must be paired with `User=` to create `--user USER:GROUP` argument

### HostName=
Configures the hostname accessible within the container environment.

**Example:** `HostName=example.com`

## Networking

### Network=
Defines networking configuration including network mode or custom network attachment.

**Format:** `host`, `none`, network name, or `.network` Quadlet reference

**Default:** Bridge network

**Multiple:** Yes (can specify multiple networks)

**Example:**
- `Network=host` - Use host networking
- `Network=mynetwork.network` - Use Quadlet network

### PublishPort=
Exposes container ports to the host with optional host binding and IP specification.

**Format:**
- `containerPort` (e.g., `80`)
- `hostPort:containerPort` (e.g., `8080:80`)
- `ip:hostPort:containerPort` (e.g., `127.0.0.1:8080:80`)
- `ip::containerPort` (dynamic host port)

**Multiple:** Yes

**Example:** `PublishPort=8080:80`

### ExposeHostPort=
Exposes host ports or port ranges to the container.

**Format:** Port number or range (e.g., `50-59`)

**Multiple:** Yes

### IP=
Assigns a static IPv4 address to the container.

**Example:** `IP=10.88.64.128`

### IP6=
Assigns a static IPv6 address to the container.

**Example:** `IP6=fd46:db93:aa76:ac37::10`

### NetworkAlias=
Registers network-scoped DNS aliases grouping containers for service discovery.

**Multiple:** Yes

### DNS=
Assigns network-scoped DNS resolver or nameserver for the container.

**Format:** IP address

**Multiple:** Yes

**Example:** `DNS=192.168.55.1`

### DNSSearch=
Configures DNS search domains for hostname resolution within the container.

**Format:** Domain name or `.` to remove search domain

**Multiple:** Yes

### DNSOption=
Sets custom DNS resolver options and behaviors.

**Example:** `DNSOption=ndots:1`

**Multiple:** Yes

### AddHost=
Establishes hostname-to-IP address mappings within the container's /etc/hosts file.

**Format:** `hostname:ip`

**Multiple:** Yes

**Example:** `AddHost=db.local:192.168.1.10`

## Storage and Volumes

### Volume=
Mounts host directories or named volumes into the container filesystem.

**Format:** `[[SOURCE-VOLUME|HOST-DIR:]CONTAINER-DIR[:OPTIONS]]`

**Options:**
- `z` - Shared SELinux label
- `Z` - Private SELinux label
- `ro` - Read-only
- `rw` - Read-write

**Multiple:** Yes

**Example:**
- `Volume=/srv/data:/usr/share/nginx/html:Z`
- `Volume=myvolume.volume:/data`

### Mount=
Attaches filesystem mounts with advanced configuration options.

**Format:** `type=TYPE,TYPE-SPECIFIC-OPTION[,...]`

**Multiple:** Yes

**Note:** Supports `.volume` and `.image` file references

### Tmpfs=
Mounts temporary filesystems within the container at specified paths.

**Format:** `CONTAINER-DIR[:OPTIONS]`

**Multiple:** Yes

**Example:** `Tmpfs=/tmp:size=64M`

### Rootfs=
Specifies a directory containing container filesystem content instead of using an image.

**Note:** Conflicts with `Image` directive; supports overlay mount syntax

### ReadOnly=
Mounts the container's root filesystem in read-only mode.

**Format:** `true` or `false`

**Default:** `false`

### ReadOnlyTmpfs=
If ReadOnly is set to `true`, mount a read-write tmpfs on /dev, /dev/shm, /run, /tmp, and /var/tmp.

**Format:** `true` or `false`

**Default:** `true` (when ReadOnly is enabled)

## Environment Variables

### Environment=
Sets environment variables inside the container using systemd service variable format.

**Format:** `name=value` pairs matching systemd environment syntax

**Multiple:** Yes

**Example:** `Environment=FOO=bar`

### EnvironmentFile=
Loads environment variables from a line-delimited file into the container.

**Format:** Absolute or relative path to environment file

**Multiple:** Yes; order persists when passed to podman run

### EnvironmentHost=
Inherits the host system's environment variables into the container.

**Format:** `true` or `false`

**Default:** `false`

## Security

### AddCapability=
Extends the default Podman capability set by adding specified capabilities to the container.

**Format:** Space-separated list of capability names

**Multiple:** Yes

**Example:** `AddCapability=CAP_DAC_OVERRIDE CAP_IPC_OWNER`

### DropCapability=
Removes capabilities from the default Podman set or removes all with `all`.

**Format:** Space-separated capability names or `all`

**Multiple:** Yes

### NoNewPrivileges=
Prevents container processes from acquiring additional privileges through setuid or capabilities.

**Format:** `true` or `false`

**Default:** `false`

### SeccompProfile=
Applies a seccomp (secure computing) filter profile for syscall restriction.

**Format:** JSON file path or `unconfined` to disable filters

### AppArmor=
Configures the AppArmor confinement profile for the container.

**Format:** Profile name or `unconfined` to disable AppArmor

### SecurityLabelDisable=
Disables SELinux label separation and isolation for the container.

**Format:** `true` or `false`

**Default:** `false` (labels enabled)

### SecurityLabelType=
Sets the SELinux process type context for container operations.

**Example:** `SecurityLabelType=spc_t`

### SecurityLabelLevel=
Assigns SELinux level context for container processes.

**Example:** `SecurityLabelLevel=s0:c1,c2`

### SecurityLabelFileType=
Sets the SELinux file type context for container files.

**Example:** `SecurityLabelFileType=usr_t`

### SecurityLabelNested=
Enables SELinux label functionality within the container for nested isolation.

**Format:** `true` or `false`

**Default:** `false`

### Mask=
Prevents access to specified filesystem paths within the container.

**Format:** Colon-separated paths

**Example:** `Mask=/proc/sys/foo:/proc/sys/bar`

### Unmask=
Removes read-only or masked restrictions from filesystem paths in the container.

**Format:** `ALL` or colon-separated paths

## User Namespaces

### UserNS=
Configures the user namespace mode with optional parameters.

**Format:** `MODE[:OPTIONS,...]`

**Example:** `UserNS=keep-id:uid=200,gid=210`

### UIDMap=
Establishes user ID mapping for the container's user namespace.

**Format:** `container_uid:host_uid:range`

**Multiple:** Yes

**Example:** `UIDMap=0:10000:10`

### GIDMap=
Establishes GID mapping for the container's new user namespace.

**Format:** `container_gid:host_gid:range`

**Multiple:** Yes

**Example:** `GIDMap=0:10000:10`

### SubUIDMap=
Applies user ID mapping using a named entry from /etc/subuid.

### SubGIDMap=
Applies group ID mapping using a named entry from /etc/subgid.

### GroupAdd=
Assigns additional groups to the primary user process or applies special flags.

**Format:** Group name, numeric GID, or `keep-groups` flag

**Multiple:** Yes

## Resource Limits

### Memory=
Restricts the maximum memory allocation for the container process.

**Example:** `Memory=20g`

**Default:** None (unlimited)

### PidsLimit=
Restricts the maximum number of processes within the container.

**Format:** Numeric limit

### ShmSize=
Specifies the allocation size for the /dev/shm shared memory filesystem.

**Example:** `ShmSize=100m`

### Ulimit=
Sets resource limits for processes within the container.

**Format:** `name=softLimit:hardLimit`

**Multiple:** Yes

**Example:** `Ulimit=nofile=1000:10000`

### Sysctl=
Configures namespace-scoped kernel parameters within the container.

**Format:** Space-separated `name=value` pairs

**Multiple:** Yes

**Example:** `Sysctl=net.ipv4.ip_forward=1`

## Health Checks

### HealthCmd=
Defines or modifies the healthcheck command executed within the container.

**Format:** Command string or `none` to disable existing healthchecks

### HealthInterval=
Establishes the time interval between successive healthcheck executions.

**Format:** Duration (e.g., `2m`) or `disable`

### HealthTimeout=
Establishes the maximum duration per healthcheck command before timeout.

**Example:** `HealthTimeout=20s`

### HealthRetries=
Sets the number of failed healthcheck attempts before marking unhealthy.

### HealthStartPeriod=
Allows initialization time before healthchecks begin.

**Example:** `HealthStartPeriod=1m`

### HealthOnFailure=
Specifies corrective action when the container becomes unhealthy.

**Example:** `HealthOnFailure=kill` (works best with systemd restart)

### HealthStartupCmd=
Specifies a startup-phase healthcheck command distinct from regular healthchecks.

### HealthStartupInterval=
Sets the interval for startup-phase healthcheck execution.

**Format:** Duration or `disable`

### HealthStartupRetries=
Limits startup-phase healthcheck attempts before restart.

### HealthStartupSuccess=
Specifies successful startup-phase runs before regular healthchecks activate.

### HealthStartupTimeout=
Sets the maximum duration for startup-phase healthcheck command execution.

### HealthLogDestination=
Specifies where healthcheck logs are stored or logged.

**Format:** `local` (default; overlay storage), `directory` (specified path), or `events_logger`

**Default:** `local`

### HealthMaxLogCount=
Limits the number of healthcheck log entries retained.

**Format:** Numeric value (0 = unlimited)

**Default:** `5` attempts

### HealthMaxLogSize=
Restricts healthcheck log size to a character limit.

**Format:** Numeric character count (0 = unlimited)

**Default:** `500` characters

## Devices and System Access

### AddDevice=
Mounts a device node from the host system into the container environment.

**Format:** `HOST-DEVICE[:CONTAINER-DEVICE][:PERMISSIONS]`

**Permissions:** Combine 'r' (read), 'w' (write), 'm' (mknod)

**Multiple:** Yes

**Note:** Prefix with `-` to add only if device exists

**Example:** `AddDevice=/dev/sda:/dev/xvdc:rwm`

## Logging

### LogDriver=
Selects the logging driver for container output handling.

**Example:** `LogDriver=journald`

### LogOpt=
Provides driver-specific logging configuration options.

**Multiple:** Yes

**Example:** `LogOpt=path=/var/log/mykube.json`

## Lifecycle and Signals

### StopSignal=
Designates the signal sent to halt the container process.

**Default:** `SIGTERM`

**Example:** `StopSignal=SIGINT`

### StopTimeout=
Sets seconds to wait before forcibly terminating the container.

**Note:** Should be lower than systemd timeout to prevent systemd killing podman rm

### ReloadSignal=
Adds an ExecReload directive sending a signal to the container's main process.

**Note:** Mutually exclusive with `ReloadCmd`

**Example:** `ReloadSignal=SIGHUP`

### ReloadCmd=
Adds an ExecReload directive executing a podman exec command for service reloads.

**Note:** Mutually exclusive with `ReloadSignal`

### Notify=
Configures systemd notification handling between container application and systemd.

**Format:**
- `false` (systemd handles) - default
- `true` (container handles)
- `healthy` (waits for healthcheck)

**Default:** `false`

### RunInit=
Provides a minimal init process within the container for signal forwarding.

**Format:** `true` or `false`

**Default:** `false`

## Container Groups (Pods)

### Pod=
Associates the container with a Quadlet `.pod` unit for group management.

**Format:** `<name>.pod` reference to existing pod unit

**Example:** `Pod=myapp.pod`

### StartWithPod=
Controls whether the container starts automatically when its associated pod starts.

**Format:** `true` or `false`

**Default:** `true`

## Image Management

### Pull=
Determines when and how container images are pulled from registries.

**Format:** Policy keyword (e.g., `never`, `always`, `missing`)

### Retry=
Sets the number of image pull retry attempts on HTTP errors.

### RetryDelay=
Establishes the delay interval between image pull retry attempts.

**Example:** `RetryDelay=5s`

### AutoUpdate=
Controls whether the container undergoes automatic updates via podman-auto-update(1).

**Format:**
- `registry` (requires fully-qualified image reference)
- `local` (compares against locally-stored image)

### HttpProxy=
Governs whether proxy environment variables propagate to the container during image operations.

**Format:** `true` or `false`

**Default:** `true`

## Advanced Options

### CgroupsMode=
The cgroups mode of the container created by Quadlet.

**Format:** `split`, `no-conmon`, `enabled`, or other cgroup modes

**Default:** `split` (differs from Podman CLI default of `enabled`)

### ContainersConfModule=
Loads specified containers.conf(5) module configurations.

**Format:** Path to module file

**Multiple:** Yes

### Timezone=
Sets the timezone for processes executing within the container.

**Example:** `Timezone=local`

### Label=
Attaches OCI metadata labels to the container as key-value pairs.

**Format:** List of `key=value` items

**Multiple:** Yes

**Example:** `Label=version=1.0 app=web`

### Annotation=
Assigns OCI annotations to the container as metadata key-value pairs.

**Format:** List of `key=value` items

**Multiple:** Yes

### Secret=
Injects Podman secrets into the container as files or environment variables.

**Format:** `secret[,opt=opt ...]` syntax

### GlobalArgs=
Passes arguments directly between `podman` and `run` commands, enabling access to unsupported Podman features.

**Format:** Space-separated arguments, individually escapable for whitespace

**Multiple:** Yes

### PodmanArgs=
Injects arguments directly into the `podman run` command for unsupported features.

**Format:** Space-separated arguments, individually escapable

**Multiple:** Yes

## Example

```ini
[Unit]
Description=Nginx web server
After=network-online.target
Wants=network-online.target

[Container]
Image=docker.io/nginx:latest
ContainerName=nginx-server
PublishPort=8080:80
Volume=/srv/www:/usr/share/nginx/html:Z
Environment=NGINX_PORT=80
AutoUpdate=registry
HealthCmd=curl -f http://localhost/ || exit 1
HealthInterval=30s

[Service]
Restart=always
TimeoutStartSec=900

[Install]
WantedBy=multi-user.target
```

## See Also

- [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html)
- [podman-run(1)](https://docs.podman.io/en/latest/markdown/podman-run.1.html)
- systemd.unit(5)
- systemd.service(5)
