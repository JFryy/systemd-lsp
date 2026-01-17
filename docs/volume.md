# [Volume] Section

Volume files are named with a `.volume` extension and contain a section `[Volume]` describing the
named Podman volume. The generated service is a one-time command that ensures that the volume
exists on the host, creating it if needed.

By default, the Podman volume has the same name as the unit, but with a `systemd-` prefix, i.e. for
a volume file named `$NAME.volume`, the generated Podman volume is called `systemd-$NAME`, and the
generated service file is `$NAME-volume.service`. The `VolumeName` option allows for overriding this
default name with a user-provided one.

Using volume units allows containers to depend on volumes being automatically pre-created. This is
particularly interesting when using special options to control volume creation,
as Podman otherwise creates volumes with the default options.

Valid options for `[Volume]` are listed below:

*Based on [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) official documentation.*

### ContainersConfModule=

Load the specified containers.conf(5) module. Equivalent to the Podman `--module` option.

This key can be listed multiple times.

### Copy=

If enabled, the content of the image located at the mountpoint of the volume is copied into the
volume on the first run.

### Device=

The path of a device which is mounted for the volume.

### Driver=

Specify the volume driver name. When set to `image`, the `Image` key must also be set.

This is equivalent to the Podman `--driver` option.

### GlobalArgs=

This key contains a list of arguments passed directly between `podman` and `volume`
in the generated file. It can be used to access Podman features otherwise unsupported by the generator. Since the generator is unaware
of what unexpected interactions can be caused by these arguments, it is not recommended to use
this option.

The format of this is a space separated list of arguments, which can optionally be individually
escaped to allow inclusion of whitespace and other control characters.

This key can be listed multiple times.

### Group=

The host (numeric) GID, or group name to use as the group for the volume

### Image=

Specifies the image the volume is based on when `Driver` is set to the `image`.
It is recommended to use a fully qualified image name rather than a short name, both for
performance and robustness reasons.

The format of the name is the same as when passed to `podman pull`. So, it supports using
`:tag` or digests to guarantee the specific image version.

Special case:

- If the `name` of the image ends with `.image`, Quadlet will use the image
  pulled by the corresponding `.image` file, and the generated systemd service contains a dependency on the `$name-image.service` (or the service name set in the .image file). Note: the corresponding `.image` file must exist.

### Label=

Set one or more OCI labels on the volume. The format is a list of
`key=value` items, similar to `Environment`.

This key can be listed multiple times.

### Options=

The mount options to use for a filesystem as used by the **mount(8)** command `-o` option.

### PodmanArgs=

This key contains a list of arguments passed directly to the end of the `podman volume create` command
in the generated file (right before the name of the volume in the command line). It can be used to
access Podman features otherwise unsupported by the generator. Since the generator is unaware
of what unexpected interactions can be caused by these arguments, is not recommended to use
this option.

The format of this is a space separated list of arguments, which can optionally be individually
escaped to allow inclusion of whitespace and other control characters.

This key can be listed multiple times.

### Type=

The filesystem type of `Device` as used by the **mount(8)** commands `-t` option.

### User=

The host (numeric) UID, or user name to use as the owner for the volume

### VolumeName=

The (optional) name of the Podman volume.
If this is not specified, the default value is the same name as the unit, but with a `systemd-` prefix,
i.e. a `$name.volume` file creates a `systemd-$name` Podman volume to avoid
conflicts with user-managed volumes.

