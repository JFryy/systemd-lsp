# [Kube] Section

The `[Kube]` section describes a Kubernetes YAML file that will be run using `podman kube play` and managed by systemd through Podman Quadlet. This allows you to deploy Kubernetes manifests on a single host using Podman. Kube units use the `.kube` file extension and are automatically converted into systemd service units.

*Based on [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) official documentation.*

## Required Configuration

### Yaml=
Specifies path to Kubernetes YAML file for `podman kube play`.

**Format:** Absolute or relative path to valid Kubernetes manifest

**Relative Path Resolution:** Resolved relative to unit file location

**Required:** Yes—mandatory key for functional Kube units

**Multiple:** Yes

**Example:**
- `Yaml=/etc/containers/manifests/myapp.yaml`
- `Yaml=./manifests/app.yaml`

## Configuration Files

### ConfigMap=
Supplies Kubernetes ConfigMap YAML path to `podman kube play` command.

**Format:** Single path per entry; supports absolute or relative paths

**Relative Path Handling:** Resolved relative to the unit file location

**Multiple:** Yes

**Example:** `ConfigMap=/etc/containers/configmaps/app-config.yaml`

## Working Directory

### SetWorkingDirectory=
Sets the `WorkingDirectory` field of the `Service` group of the systemd service unit file.

**Purpose:** Allows `podman kube play` to correctly resolve relative paths

**Supported Values:**
- `yaml` - Sets working directory to YAML file location
- `unit` - Sets working directory to Quadlet unit file location

**Limitation:** Only `unit` supported with multiple `Yaml` paths

**Note:** User-set `WorkingDirectory` in `[Service]` group takes precedence

**Example:** `SetWorkingDirectory=yaml`

## Networking

### Network=
Designates custom network configuration for containers.

**Format:**
- Network name
- `host` - Use host networking
- `none` - No networking
- `.network` suffix - Reference to Quadlet network unit (creates dependency on corresponding `.network` unit with `systemd-%N` prefix)

**Multiple:** Yes

**Example:**
- `Network=host`
- `Network=mynetwork.network`

### PublishPort=
Exposes ports from container to host system.

**Format:**
- `containerPort` (e.g., `80`)
- `hostPort:containerPort` (e.g., `8080:80`)
- `ip:hostPort:containerPort` (e.g., `127.0.0.1:8080:80`)
- `ip::containerPort` (dynamic host port)
- Port ranges: `50-59`

**IPv6:** Use `[::]` for IPv6 binding; `0.0.0.0` or omitted binds all IPv4 addresses

**Merging:** Ports from unit file merge with YAML file ports; unit file entries take precedence

**Multiple:** Yes

**Example:** `PublishPort=8080:80`

## User Namespaces

### UserNS=
Configures user namespace mode for container execution.

**Format:** `MODE[:OPTIONS,...]`

**Examples:** Modes like `keep-id:uid=200,gid=210`

**Example:** `UserNS=keep-id`

## Lifecycle Management

### ExitCodePropagation=
Determines how the systemd service main process exit status reflects container failures.

**Supported Values:**
- `none` - Service always exits zero, ignoring container failures (default)
- `any` - Service exits non-zero if any container failed
- `all` - Service exits non-zero only if all containers failed

**Default:** `none`

**Example:** `ExitCodePropagation=any`

### KubeDownForce=
Removes all resources, including volumes, when executing `podman kube down`.

**Format:** `true` or `false`

**Default:** `false`

**Example:** `KubeDownForce=true`

## Updates

### AutoUpdate=
Controls whether containers will be automatically updated via podman-auto-update(1).

**Supported Values:**
- `registry` - Requires fully-qualified image references; checks and pulls from registry
- `local` - Compares container image to locally stored image with same name
- `name/(local|registry)` - Performs autoupdate on specified container name only

**Default:** Not enabled

**Multiple:** Yes

**Example:**
- `AutoUpdate=registry`
- `AutoUpdate=mycontainer/registry`

## Logging

### LogDriver=
Specifies the log driver for Podman container execution.

**Format:** Driver name

**Example:**
- `LogDriver=journald`
- `LogDriver=json-file`

## Advanced Options

### ContainersConfModule=
Loads specified containers.conf(5) module into the Kube unit.

**Format:** Path to module file

**Multiple:** Yes

**Example:** `ContainersConfModule=/etc/containers/modules/mymodule.conf`

### GlobalArgs=
Passes arguments directly between `podman` and `kube` in the generated file.

**Usage:** Enables access to otherwise unsupported Podman features

**Format:** Space-separated list; supports escaped whitespace and control characters

**Multiple:** Yes

**Warning:** Not recommended; generator unaware of potential unexpected interactions

### PodmanArgs=
Passes arguments directly to the end of the `podman kube play` command.

**Position:** Right before YAML file path in command line

**Usage:** Accesses Podman features unsupported by the generator

**Format:** Space-separated list with optional escape sequences

**Multiple:** Yes

**Warning:** Not recommended due to potential interactions

## Example: Basic Kube Unit

```ini
[Unit]
Description=WordPress application from Kubernetes YAML
After=network-online.target
Wants=network-online.target

[Kube]
Yaml=/etc/containers/wordpress/deployment.yaml
ConfigMap=/etc/containers/wordpress/config.yaml
Network=wordpress-net.network
PublishPort=8080:80
ExitCodePropagation=any
AutoUpdate=registry

[Service]
Restart=always
TimeoutStartSec=900

[Install]
WantedBy=multi-user.target
```

## Example: Multi-YAML Kube Unit

```ini
[Unit]
Description=Multi-component application

[Kube]
Yaml=./manifests/database.yaml
Yaml=./manifests/backend.yaml
Yaml=./manifests/frontend.yaml
SetWorkingDirectory=unit
Network=myapp-net.network
ExitCodePropagation=all

[Install]
WantedBy=multi-user.target
```

## Example: Internal Application with ConfigMaps

```ini
[Unit]
Description=Internal microservice

[Kube]
Yaml=/srv/apps/myservice/deployment.yaml
ConfigMap=/srv/apps/myservice/app-config.yaml
ConfigMap=/srv/apps/myservice/db-config.yaml
Network=internal.network
LogDriver=journald
UserNS=keep-id

[Install]
WantedBy=multi-user.target
```

## Kubernetes YAML Example

The YAML file referenced by `Yaml=` should be a valid Kubernetes manifest:

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: myapp
spec:
  containers:
  - name: web
    image: docker.io/nginx:latest
    ports:
    - containerPort: 80
    volumeMounts:
    - name: data
      mountPath: /usr/share/nginx/html
  volumes:
  - name: data
    emptyDir: {}
```

## See Also

- [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html)
- [podman-kube(1)](https://docs.podman.io/en/latest/markdown/podman-kube.1.html)
- [podman-kube-play(1)](https://docs.podman.io/en/latest/markdown/podman-kube-play.1.html)
- [podman-kube-down(1)](https://docs.podman.io/en/latest/markdown/podman-kube-down.1.html)
- systemd.unit(5)
- systemd.service(5)
