# [Image] Section

Image files are named with a `.image` extension and contain a section `[Image]` describing the
container image pull command. The generated service is a one-time command that ensures that the image
exists on the host, pulling it if needed.

Using image units allows containers and volumes to depend on images being automatically pulled. This is
particularly interesting when using special options to control image pulls.

Valid options for `[Image]` are listed below:

*Based on [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) official documentation.*

### AllTags=

All tagged images in the repository are pulled.

This is equivalent to the Podman `--all-tags` option.

### Arch=

Override the architecture, defaults to hosts, of the image to be pulled.

This is equivalent to the Podman `--arch` option.

### AuthFile=

Path of the authentication file.

This is equivalent to the Podman `--authfile` option.

### CertDir=

Use certificates at path (\*.crt, \*.cert, \*.key) to connect to the registry.

This is equivalent to the Podman `--cert-dir` option.

### ContainersConfModule=

Load the specified containers.conf(5) module. Equivalent to the Podman `--module` option.

This key can be listed multiple times.

### Creds=

The `[username[:password]]` to use to authenticate with the registry, if required.

This is equivalent to the Podman `--creds` option.

### DecryptionKey=

The `[key[:passphrase]]` to be used for decryption of images.

This is equivalent to the Podman `--decryption-key` option.

### GlobalArgs=

This key contains a list of arguments passed directly between `podman` and `image`
in the generated file. It can be used to access Podman features otherwise unsupported by the generator. Since the generator is unaware
of what unexpected interactions can be caused by these arguments, it is not recommended to use
this option.

The format of this is a space separated list of arguments, which can optionally be individually
escaped to allow inclusion of whitespace and other control characters.

This key can be listed multiple times.

### Image=

The image to pull.
It is recommended to use a fully qualified image name rather than a short name, both for
performance and robustness reasons.

The format of the name is the same as when passed to `podman pull`. So, it supports using
`:tag` or digests to guarantee the specific image version.

### ImageTag=

Actual FQIN of the referenced `Image`.
Only meaningful when source is a file or directory archive.

For example, an image saved into a `docker-archive` with the following Podman command:

`podman image save --format docker-archive --output /tmp/archive-file.tar quay.io/podman/stable:latest`

requires setting

- `Image=docker-archive:/tmp/archive-file.tar`

- `ImageTag=quay.io/podman/stable:latest`

### OS=

Override the OS, defaults to hosts, of the image to be pulled.

This is equivalent to the Podman `--os` option.

### PodmanArgs=

This key contains a list of arguments passed directly to the end of the `podman image pull` command
in the generated file (right before the image name in the command line). It can be used to
access Podman features otherwise unsupported by the generator. Since the generator is unaware
of what unexpected interactions can be caused by these arguments, it is not recommended to use
this option.

The format of this is a space separated list of arguments, which can optionally be individually
escaped to allow inclusion of whitespace and other control characters.

This key can be listed multiple times.

### Policy=

The pull policy to use when pulling the image.

This is equivalent to the Podman `--policy` option.

### Retry=

Number of times to retry the image pull when a HTTP error occurs. Equivalent to the Podman `--retry` option.

### RetryDelay=

Delay between retries. Equivalent to the Podman `--retry-delay` option.

### TLSVerify=

Require HTTPS and verification of certificates when contacting registries.

This is equivalent to the Podman `--tls-verify` option.

### Variant=

Override the default architecture variant of the container image.

This is equivalent to the Podman `--variant` option.

