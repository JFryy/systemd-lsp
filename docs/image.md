# [Image] Section

The `[Image]` section describes a container image pull operation that will be executed using `podman image pull` and managed by systemd through Podman Quadlet. This allows you to declaratively ensure container images are pulled and available before dependent containers start. Image units use the `.image` file extension and are automatically converted into systemd service units.

*Based on [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) official documentation.*

## Required Configuration

### Image=
The container image identifier to retrieve.

**Format:** Registry/repository/image with optional tag or digest

**Recommendation:** Use fully-qualified names (include registry) for performance and robustness

**Required:** Yes

**Example:**
- `Image=docker.io/nginx:latest`
- `Image=quay.io/centos/centos:stream9`
- `Image=registry.example.com/myapp@sha256:abcd1234...`

### ImageTag=
Specifies an alternate reference name for the pulled image when other Quadlet units reference it via `.image` notation.

**Format:** Image name/tag

**Default:** Uses the `Image=` value

**Purpose:** Allows renaming/tagging the pulled image for use by other units

**Example:** `ImageTag=myapp:latest`

## Pull Policy

### Policy=
Controls when image pulls occur.

**Values:**
- `always` - Always pull from registry (ignore local cache)
- `missing` - Pull only if not present locally
- `never` - Never pull, use local only (fail if missing)
- `newer` - Pull if registry has newer version

**Example:** `Policy=always`

### AllTags=
Retrieves all tagged variants of a container image from the repository.

**Format:** `true` or `false`

**Default:** `false`

**Example:** `AllTags=true`

### Retry=
Sets how many times the pull operation automatically retries upon encountering HTTP connection errors.

**Format:** Numeric count

**Example:** `Retry=5`

### RetryDelay=
Establishes the waiting period between consecutive retry attempts for failed pulls.

**Format:** Time duration with units

**Example:** `RetryDelay=10s`

## Architecture and Platform

### Arch=
Specifies a non-default processor architecture for the pulled image.

**Format:** Architecture identifier

**Default:** System's native architecture

**Example:**
- `Arch=aarch64`
- `Arch=amd64`
- `Arch=arm`

### OS=
Specifies the target operating system for the image.

**Format:** OS identifier

**Default:** Host's operating system

**Example:**
- `OS=linux`
- `OS=windows`

### Variant=
Overrides the default processor architecture variant.

**Format:** Architecture variant specification

**Example:** `Variant=arm/v7`

## Authentication

### AuthFile=
Indicates the location of authentication credentials for accessing private container registries.

**Format:** Absolute file path

**Example:** `AuthFile=/etc/registry/auth.json`

### Creds=
Supplies username and password credentials for authenticating with container registries.

**Format:** `username:password`

**Warning:** Credentials stored in plain text; prefer `AuthFile=` for security

**Example:** `Creds=myuser:mypassword`

## TLS and Certificates

### TLSVerify=
Determines whether SSL/TLS certificate validation is enforced when connecting to registries.

**Format:** `true` or `false`

**Default:** `true` (verification enabled)

**Example:** `TLSVerify=false`

### CertDir=
Specifies where certificate files for registry verification are stored on the system.

**Format:** Directory path

**Example:** `CertDir=/etc/registry/certs`

## Encryption

### DecryptionKey=
Provides the path to a key file necessary for decrypting encrypted container images.

**Format:** File path

**Example:** `DecryptionKey=/etc/registry/decryption.key`

## Advanced Options

### ContainersConfModule=
Loads a specific containers.conf configuration module to modify pull behavior.

**Format:** File path

**Multiple:** Yes

**Example:** `ContainersConfModule=/etc/containers/nvidia.conf`

### GlobalArgs=
Passes low-level arguments directly to the Podman binary between "podman" and "image."

**Format:** Space-separated arguments with optional escaping

**Multiple:** Yes

**Warning:** Not recommended for general use

**Example:** `GlobalArgs=--log-level=debug`

### PodmanArgs=
Transmits arguments directly to the end of the pull command before the image name.

**Format:** Space-separated arguments with optional escaping

**Multiple:** Yes

**Warning:** Not recommended for general use

**Example:** `PodmanArgs=--platform=linux/amd64`

## Example: Basic Image Pull

```ini
[Unit]
Description=Pull nginx image

[Image]
Image=docker.io/nginx:latest
Policy=missing

[Install]
WantedBy=multi-user.target
```

## Example: Private Registry with Authentication

```ini
[Unit]
Description=Pull private application image

[Image]
Image=registry.example.com/myapp:v1.0
AuthFile=/etc/registry/auth.json
TLSVerify=true
Policy=always
Retry=3
RetryDelay=5s

[Install]
WantedBy=multi-user.target
```

## Example: Cross-Architecture Pull

```ini
[Unit]
Description=Pull ARM image on x86 host

[Image]
Image=docker.io/myapp:latest
Arch=aarch64
OS=linux
Variant=arm/v8
ImageTag=myapp:arm64

[Install]
WantedBy=multi-user.target
```

## Example: Pull All Tags

```ini
[Unit]
Description=Pull all nginx tags

[Image]
Image=docker.io/nginx
AllTags=true
Policy=always

[Install]
WantedBy=multi-user.target
```

## Example: Encrypted Image

```ini
[Unit]
Description=Pull encrypted image

[Image]
Image=registry.example.com/secure/myapp:latest
AuthFile=/etc/registry/auth.json
DecryptionKey=/etc/registry/decryption.key
TLSVerify=true

[Install]
WantedBy=multi-user.target
```

## Using Pulled Images

Reference image units from container or build units using `.image` suffix:

```ini
# myapp.container
[Container]
Image=nginx.image
PublishPort=8080:80
```

Or reference in volume units:

```ini
# data.volume
[Volume]
Driver=image
Image=nginx.image
```

The LSP will automatically create service dependencies to ensure the image is pulled before dependent units start.

## Comparison: Image vs Build Units

| Feature | `.image` Unit | `.build` Unit |
|---------|--------------|---------------|
| Purpose | Pull existing image | Build image from Containerfile |
| Source | Container registry | Local Containerfile |
| Use Case | Production deployments | Custom/local images |
| Speed | Faster (just download) | Slower (compilation) |
| Dependencies | Network access | Build tools, context |

## See Also

- [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html)
- [podman-image(1)](https://docs.podman.io/en/latest/markdown/podman-image.1.html)
- [podman-pull(1)](https://docs.podman.io/en/latest/markdown/podman-pull.1.html)
- systemd.unit(5)
- systemd.service(5)
