# [Volume] Section

The `[Volume]` section describes a Podman volume that will be created and managed by systemd using Podman Quadlet. Volumes provide persistent storage for containers that survives container restarts and deletions. Volume units use the `.volume` file extension and are automatically converted into systemd service units.

*Based on [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) official documentation.*

## Basic Configuration

### VolumeName=
Allows custom naming of the Podman volume.

**Default:** `systemd-%N` (where %N is the unit filename without extension)

**Note:** Avoids naming conflicts with user-managed volumes

**Example:** `VolumeName=myapp-data`

## Storage Configuration

### Driver=
Designates which volume driver manages the storage backend.

**Format:** Driver name

**Supported Drivers:**
- Default driver
- `image` - Uses container image as volume source (requires `Image=` directive)
- Other specialized drivers

**Example:** `Driver=image`

### Device=
Specifies a device path that will be mounted to provide the volume storage.

**Format:** Device path

**Example:** `Device=/dev/sda1`

### Type=
Determines the filesystem type for the specified device during mounting operations.

**Format:** Filesystem type identifier (same as mount(8) `-t` option)

**Example:** `Type=ext4`

### Options=
Passes mount command options controlling filesystem behavior and mounting characteristics.

**Format:** Mount options as used by mount(8) `-o` option

**Example:** `Options=noatime,nodiratime`

## Image-Based Volumes

### Image=
Identifies the container image serving as the volume's foundation when driver is `image`.

**Format:** Image reference with optional tag/digest

**Recommendation:** Use fully-qualified names for performance and robustness

**Special Case:** If name ends with `.image`, the referenced `.image` file's built image is used with automatic service dependency

**Example:**
- `Image=docker.io/nginx:latest`
- `Image=myimage.image`

### Copy=
Controls whether content from the image at the volume's mountpoint is duplicated into the volume upon initial creation.

**Format:** `true` or `false`

**Default:** `true`

**Note:** The content of the image located at the mountpoint of the volume is copied into the volume on the first run

## Permissions

### User=
Establishes ownership of the volume through either numeric UID or username.

**Format:** Numeric UID or user name

**Example:**
- `User=1000`
- `User=myuser`

### Group=
Assigns group ownership of the volume using either numeric GID or group name.

**Format:** Numeric GID or group name

**Example:**
- `Group=1000`
- `Group=mygroup`

## Metadata

### Label=
Attaches OCI metadata labels to the volume using key-value pairs.

**Format:** `key=value` format similar to environment variables

**Multiple:** Yes

**Example:** `Label=purpose=database backup=daily`

## Advanced Options

### ContainersConfModule=
Loads a specified containers.conf module for volume operations.

**Format:** Path to module file

**Multiple:** Yes

### GlobalArgs=
Provides access to podman options not otherwise exposed through Quadlet directives by passing arguments directly between podman and volume command.

**Format:** Space-separated argument list with optional escape sequences

**Multiple:** Yes

**Note:** Not recommended to use this option

### PodmanArgs=
Injects arguments directly preceding the volume name in the podman volume create command for unsupported features.

**Format:** Space-separated argument list with optional escape sequences

**Multiple:** Yes

**Note:** Not recommended to use this option

## Example: Basic Volume

```ini
[Unit]
Description=Application data volume

[Volume]
VolumeName=myapp-data
User=1000
Group=1000
Label=app=myapp type=data

[Install]
WantedBy=multi-user.target
```

## Example: Image-Based Volume

```ini
[Unit]
Description=Web content volume from image

[Volume]
VolumeName=web-content
Driver=image
Image=docker.io/nginx:latest
Copy=true
User=nginx
Group=nginx

[Install]
WantedBy=multi-user.target
```

## Example: Device-Based Volume

```ini
[Unit]
Description=External storage volume

[Volume]
VolumeName=external-data
Device=/dev/sdb1
Type=ext4
Options=noatime,nodiratime
User=1000
Group=1000

[Install]
WantedBy=multi-user.target
```

## Using Volumes in Containers

Reference volume units from container or pod units:

```ini
# myapp.container
[Container]
Image=docker.io/myapp:latest
Volume=myapp-data.volume:/data
```

The LSP will automatically create service dependencies when using `.volume` references.

## See Also

- [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html)
- [podman-volume(1)](https://docs.podman.io/en/latest/markdown/podman-volume.1.html)
- [podman-volume-create(1)](https://docs.podman.io/en/latest/markdown/podman-volume-create.1.html)
- systemd.unit(5)
- mount(8)
