# [Build] Section

Build files are named with a `.build` extension and contain a section `[Build]` describing the image
build command. The generated service is a one-time command that ensures that the image is built on
the host from a supplied Containerfile and context directory. Subsequent (re-)starts of the
generated built service will usually finish quickly, as image layer caching will skip unchanged
build steps.

A minimal `.build` unit needs at least the `ImageTag=` key, and either of `File=` or
`SetWorkingDirectory=` keys.

Using build units allows containers and volumes to depend on images being built locally. This can be
interesting for creating container images not available on container registries, or for local
testing and development.

Valid options for `[Build]` are listed below:

*Based on [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) official documentation.*

### Annotation=

Add an image _annotation_ (e.g. annotation= _value_) to the image metadata. Can be used multiple
times.

This is equivalent to the `--annotation` option of `podman build`.

### Arch=

Override the architecture, defaults to hosts’, of the image to be built.

This is equivalent to the `--arch` option of `podman build`.

### AuthFile=

Path of the authentication file.

This is equivalent to the `--authfile` option of `podman build`.

### BuildArg=

Specifies a build argument and its value in the same way environment variables are
(e.g., env= _value_), but it is not added to the environment variable list in the
resulting image’s configuration. Can be listed multiple times.

This is equivalent to the `--build-arg` option of `podman build`.

### ContainersConfModule=

Load the specified containers.conf(5) module. Equivalent to the Podman `--module` option.

This key can be listed multiple times.

### DNS=

Set network-scoped DNS resolver/nameserver for the build container.

This key can be listed multiple times.

This is equivalent to the `--dns` option of `podman build`.

### DNSOption=

Set custom DNS options.

This key can be listed multiple times.

This is equivalent to the `--dns-option` option of `podman build`.

### DNSSearch=

Set custom DNS search domains. Use **DNSSearch=.** to remove the search domain.

This key can be listed multiple times.

This is equivalent to the `--dns-search` option of `podman build`.

### Environment=

Add a value (e.g. env= _value_) to the built image. This uses the same format as [services in\
systemd](https://www.freedesktop.org/software/systemd/man/systemd.exec.html#Environment=) and can be
listed multiple times.

### File=

Specifies a Containerfile which contains instructions for building the image. A URL starting with
`http(s)://` allows you to specify a remote Containerfile to be downloaded. Note that for a given
relative path to a Containerfile, or when using a `http(s)://` URL, you also must set
`SetWorkingDirectory=` in order for `podman build` to find a valid context directory for the
resources specified in the Containerfile.

Note that setting a `File=` field is mandatory for a `.build` file, unless `SetWorkingDirectory` (or
a `WorkingDirectory` in the `Service` group) has also been set.

This is equivalent to the `--file` option of `podman build`.

### ForceRM=

Always remove intermediate containers after a build, even if the build fails (default true).

This is equivalent to the `--force-rm` option of `podman build`.

### GlobalArgs=

This key contains a list of arguments passed directly between `podman` and `build` in the generated
file. It can be used to access Podman features otherwise unsupported by the generator. Since the
generator is unaware of what unexpected interactions can be caused by these arguments, it is not
recommended to use this option.

The format of this is a space separated list of arguments, which can optionally be individually
escaped to allow inclusion of whitespace and other control characters.

This key can be listed multiple times.

### GroupAdd=

Assign additional groups to the primary user running within the container process. Also supports the
`keep-groups` special flag.

This is equivalent to the `--group-add` option of `podman build`.

### IgnoreFile=

Path to an alternate .containerignore file to use when building the image.
Note that when using a relative path you should also set `SetWorkingDirectory=`

This is equivalent to the `--ignorefile` option of `podman build`.

### ImageTag=

Specifies the name which is assigned to the resulting image if the build process completes
successfully.

This is equivalent to the `--tag` option of `podman build`.

This key can be listed multiple times. The first instance will be used as the name of the created artifact when the `.build` file is referenced by another Quadlet unit.

### Label=

Add an image _label_ (e.g. label= _value_) to the image metadata. Can be used multiple times.

This is equivalent to the `--label` option of `podman build`.

### Network=

Sets the configuration for network namespaces when handling RUN instructions. This has the same
format as the `--network` option to `podman build`. For example, use `host` to use the host network,
or `none` to not set up networking.

Special case:

- If the `name` of the network ends with `.network`, Quadlet will look for the corresponding `.network` Quadlet unit. If found, Quadlet will use the name of the Network set in the Unit, otherwise, `systemd-$name` is used. The generated systemd service contains a dependency on the service unit generated for that `.network` unit, or on `$name-network.service` if the `.network` unit is not found. Note: the corresponding `.network` file must exist.

This key can be listed multiple times.

### PodmanArgs=

This key contains a list of arguments passed directly to the end of the `podman build` command
in the generated file (right before the image name in the command line). It can be used to
access Podman features otherwise unsupported by the generator. Since the generator is unaware
of what unexpected interactions can be caused by these arguments, it is not recommended to use
this option.

The format of this is a space separated list of arguments, which can optionally be individually
escaped to allow inclusion of whitespace and other control characters.

This key can be listed multiple times.

### Pull=

Set the image pull policy.

This is equivalent to the `--pull` option of `podman build`.

### Retry=

Number of times to retry the image pull when a HTTP error occurs. Equivalent to the Podman `--retry` option.

### RetryDelay=

Delay between retries. Equivalent to the Podman `--retry-delay` option.

### Secret=

Pass secret information used in Containerfile build stages in a safe way.

This is equivalent to the `--secret` option of `podman build` and generally has the form
`secret[,opt=opt ...]`.

### SetWorkingDirectory=

Provide context (a working directory) to `podman build`. Supported values are a path, a URL, or the
special keys `file` or `unit` to set the context directory to the parent directory of the file from
the `File=` key or to that of the Quadlet `.build` unit file, respectively. This allows Quadlet to
resolve relative paths.

When using one of the special keys ( `file` or `unit`), the `WorkingDirectory` field of the `Service`
group of the Systemd service unit will also be set to accordingly. Alternatively, users can
explicitly set the `WorkingDirectory` field of the `Service` group in the `.build` file. Please note
that if the `WorkingDirectory` field of the `Service` group is set by the user, Quadlet will not
overwrite it even if `SetWorkingDirectory` is set to `file` or `unit`.

By providing a URL to `SetWorkingDirectory=` you can instruct `podman build` to clone a Git
repository or download an archive file extracted to a temporary location by `podman build` as build
context. Note that in this case, the `WorkingDirectory` of the Systemd service unit is left
untouched by Quadlet.

Note that providing context directory is mandatory for a `.build` file, unless a `File=` key has
also been provided.

### Target=

Set the target build stage to build. Commands in the Containerfile after the target stage are
skipped.

This is equivalent to the `--target` option of `podman build`.

### TLSVerify=

Require HTTPS and verification of certificates when contacting registries.

This is equivalent to the `--tls-verify` option of `podman build`.

### Variant=

Override the default architecture variant of the container image to be built.

This is equivalent to the `--variant` option of `podman build`.

### Volume=

Mount a volume to containers when executing RUN instructions during the build. This is equivalent to
the `--volume` option of `podman build`, and generally has the form
`[[SOURCE-VOLUME|HOST-DIR:]CONTAINER-DIR[:OPTIONS]]`.

If `SOURCE-VOLUME` starts with `.`, Quadlet resolves the path relative to the location of the unit file.

Special case:

- If `SOURCE-VOLUME` ends with `.volume`, Quadlet will look for the corresponding `.volume` Quadlet unit. If found, Quadlet will use the name of the Volume set in the Unit, otherwise, `systemd-$name` is used. The generated systemd service contains a dependency on the service unit generated for that `.volume` unit, or on `$name-volume.service` if the `.volume` unit is not found. Note: the corresponding `.volume` file must exist.

This key can be listed multiple times.

