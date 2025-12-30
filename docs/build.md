# [Build] Section

The `[Build]` section describes a container image build process that will be executed using `podman build` and managed by systemd through Podman Quadlet. This allows you to build container images declaratively and ensure they're built before dependent containers start. Build units use the `.build` file extension and are automatically converted into systemd service units.

*Based on [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) official documentation.*

## Required Configuration

At least one of these must be specified:

### File=
Specifies Containerfile path for image building.

**Format:** Absolute or relative path, or remote URL (`http(s)://`)

**Mandatory:** Unless `SetWorkingDirectory` is configured

**Example:**
- `File=/etc/containers/builds/Containerfile`
- `File=./Containerfile`
- `File=https://example.com/Containerfile`

### SetWorkingDirectory=
Provides build context directory to `podman build`.

**Supported Values:**
- `file` - Parent directory of `File=` path
- `unit` - Parent directory of `.build` unit file
- Absolute path to build context
- URL (Git repository or archive)

**Behavior:** When using `file` or `unit`, systemd `WorkingDirectory` is automatically set

**Note:** User-defined `WorkingDirectory` in `[Service]` section takes precedence

**Mandatory:** Unless `File=` is provided

**Example:**
- `SetWorkingDirectory=file`
- `SetWorkingDirectory=/srv/builds/myapp`
- `SetWorkingDirectory=https://github.com/user/repo.git`

## Image Naming

### ImageTag=
Assigns name to resulting image upon successful build.

**Format:** Image name with optional tag

**Multiple:** Yes (first instance used as artifact name for `.build` references)

**Example:**
- `ImageTag=myapp:latest`
- `ImageTag=localhost/myapp:v1.0`

## Build Arguments and Environment

### BuildArg=
Defines build arguments in `key=value` format.

**Note:** These arguments don't persist in the resulting image configuration

**Multiple:** Yes

**Example:** `BuildArg=VERSION=1.0 ENVIRONMENT=production`

### Environment=
Adds environment variables to built image using systemd format.

**Format:** `key=value` pairs

**Multiple:** Yes

**Example:** `Environment=PATH=/usr/local/bin:$PATH`

### Secret=
Passes secret information safely to build stages.

**Format:** `secret[,opt=opt ...]`

**Example:** `Secret=id=mysecret,src=/run/secrets/token`

## Build Context

### Target=
Specifies target build stage to build.

**Note:** Commands after target stage are skipped (multi-stage builds)

**Example:** `Target=production`

### IgnoreFile=
Specifies alternate `.containerignore` file path.

**Recommendation:** Set `SetWorkingDirectory=` with relative paths

**Example:** `IgnoreFile=.buildignore`

## Architecture

### Arch=
Overrides the system architecture for the built image.

**Default:** Host architecture

**Example:**
- `Arch=aarch64`
- `Arch=amd64`

### Variant=
Overrides default architecture variant.

**Format:** Variant identifier

**Example:** `Variant=arm/v7`

## Networking During Build

### Network=
Sets network namespace configuration during build.

**Format:**
- Network mode (e.g., `host`, `none`)
- `.network` suffix - Quadlet unit reference (automatic dependencies)

**Multiple:** Yes

**Example:**
- `Network=host`
- `Network=build-net.network`

### DNS=
Sets DNS resolver/nameserver for build container.

**Format:** IP address

**Multiple:** Yes

**Example:** `DNS=8.8.8.8`

### DNSOption=
Sets custom DNS options during build.

**Multiple:** Yes

**Example:** `DNSOption=ndots:1`

### DNSSearch=
Sets custom DNS search domains; use `.` to remove default search domain.

**Multiple:** Yes

**Example:** `DNSSearch=example.com`

## Storage and Volumes

### Volume=
Mounts volumes during RUN instruction execution.

**Format:** `[[SOURCE|HOST-DIR:]CONTAINER-DIR[:OPTIONS]]`

**Relative Paths:** Starting with `.` resolve to unit file directory

**Special Case:** Supports `.volume` Quadlet unit references with automatic dependencies

**Multiple:** Yes

**Example:**
- `Volume=/srv/cache:/cache`
- `Volume=build-cache.volume:/cache`

## Metadata

### Label=
Adds image labels to metadata.

**Format:** `key=value` pairs

**Multiple:** Yes

**Example:** `Label=maintainer=admin@example.com version=1.0`

### Annotation=
Adds image annotations to metadata.

**Format:** `key=value` pairs

**Multiple:** Yes

**Example:** `Annotation=org.opencontainers.image.source=https://github.com/user/repo`

## Authentication and Security

### AuthFile=
Specifies authentication file path for registry access.

**Format:** File path (absolute or relative)

**Example:** `AuthFile=/etc/registry/auth.json`

### TLSVerify=
Controls HTTPS requirement and certificate verification.

**Format:** `true` or `false`

**Default:** `true` (requires valid certificates)

**Example:** `TLSVerify=false`

## Image Pull Policy

### Pull=
Sets image pull policy during build.

**Values:** `always`, `missing`, `never`, `newer`

**Example:** `Pull=always`

### Retry=
Number of retries for HTTP errors during image operations.

**Format:** Numeric count

**Example:** `Retry=3`

### RetryDelay=
Delay duration between pull retry attempts.

**Format:** Time duration

**Example:** `RetryDelay=10s`

## User and Group

### GroupAdd=
Assigns additional groups to build process user.

**Format:** Group name/GID or `keep-groups` special flag

**Example:** `GroupAdd=docker`

## Build Behavior

### ForceRM=
Controls intermediate container removal after build.

**Format:** `true` or `false`

**Default:** `true` (removes containers even on failure)

**Example:** `ForceRM=true`

## Advanced Options

### ContainersConfModule=
Loads specified containers.conf modules.

**Format:** File path

**Multiple:** Yes

**Example:** `ContainersConfModule=/etc/containers/nvidia.conf`

### GlobalArgs=
Passes arguments directly between `podman` and `build` command.

**Format:** Space-separated arguments with optional escaping

**Multiple:** Yes

**Warning:** Not recommended for general use

### PodmanArgs=
Passes arguments directly to end of `podman build` command.

**Format:** Space-separated arguments with optional escaping

**Multiple:** Yes

**Warning:** Not recommended for general use

## Example: Basic Build

```ini
[Unit]
Description=Build myapp container image

[Build]
File=./Containerfile
SetWorkingDirectory=unit
ImageTag=localhost/myapp:latest
BuildArg=VERSION=1.0

[Install]
WantedBy=multi-user.target
```

## Example: Multi-Stage Build

```ini
[Unit]
Description=Build production image

[Build]
File=/srv/myapp/Containerfile
SetWorkingDirectory=/srv/myapp
ImageTag=myapp:production
Target=production
BuildArg=ENVIRONMENT=production
Pull=always
Label=version=1.0 environment=production

[Install]
WantedBy=multi-user.target
```

## Example: Cross-Architecture Build

```ini
[Unit]
Description=Build ARM image on x86

[Build]
File=./Containerfile
SetWorkingDirectory=unit
ImageTag=myapp:arm64
Arch=aarch64
BuildArg=TARGETARCH=arm64

[Install]
WantedBy=multi-user.target
```

## Example: Build with Custom Network and Cache

```ini
[Unit]
Description=Build with build network and cache volume

[Build]
File=./Containerfile
SetWorkingDirectory=unit
ImageTag=localhost/myapp:latest
Network=build-net.network
Volume=build-cache.volume:/cache
DNS=8.8.8.8
AuthFile=/etc/registry/auth.json

[Install]
WantedBy=multi-user.target
```

## Using Built Images

Reference build units from container units using `.build` suffix:

```ini
# myapp.container
[Container]
Image=myapp.build
PublishPort=8080:80
```

The LSP will automatically create service dependencies to ensure the image is built before the container starts.

## See Also

- [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html)
- [podman-build(1)](https://docs.podman.io/en/latest/markdown/podman-build.1.html)
- [Containerfile(5)](https://docs.podman.io/en/latest/markdown/Containerfile.5.html)
- systemd.unit(5)
- systemd.service(5)
