# [Kube] Section

Kube units are named with a `.kube` extension and contain a `[Kube]` section describing
how `podman kube play` runs as a service. The resulting service file contains a line like
`ExecStart=podman kube play … file.yml`, and most of the keys in this section control the command-line
options passed to Podman. However, some options also affect the details of how systemd is set up to run and
interact with the container.

There is only one required key, `Yaml`, which defines the path to the Kubernetes YAML file.

Valid options for `[Kube]` are listed below:

*Based on [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) official documentation.*

### AutoUpdate=

Indicates whether containers will be auto-updated ( [podman-auto-update(1)](podman-auto-update.1.html)). AutoUpdate can be specified multiple times. The following values are supported:

- `registry`: Requires a fully-qualified image reference (e.g., quay.io/podman/stable:latest) to be used to create the container. This enforcement is necessary to know which images to actually check and pull. If an image ID was used, Podman does not know which image to check/pull anymore.

- `local`: Tells Podman to compare the image a container is using to the image with its raw name in local storage. If an image is updated locally, Podman simply restarts the systemd unit executing the Kubernetes Quadlet.

- `name/(local|registry)`: Tells Podman to perform the `local` or `registry` autoupdate on the specified container name.

### ConfigMap=

Pass the Kubernetes ConfigMap YAML path to `podman kube play` via the `--configmap` argument.
Unlike the `configmap` argument, the value may contain only one path but
it may be absolute or relative to the location of the unit file.

This key may be used multiple times

### ContainersConfModule=

Load the specified containers.conf(5) module. Equivalent to the Podman `--module` option.

This key can be listed multiple times.

### ExitCodePropagation=

Control how the main PID of the systemd service should exit. The following values are supported:

- `all`: exit non-zero if all containers have failed (i.e., exited non-zero)

- `any`: exit non-zero if any container has failed

- `none`: exit zero and ignore failed containers

The current default value is `none`.

### GlobalArgs=

This key contains a list of arguments passed directly between `podman` and `kube`
in the generated file. It can be used to access Podman features otherwise unsupported by the generator. Since the generator is unaware
of what unexpected interactions can be caused by these arguments, it is not recommended to use
this option.

The format of this is a space separated list of arguments, which can optionally be individually
escaped to allow inclusion of whitespace and other control characters.

This key can be listed multiple times.

### KubeDownForce=

Remove all resources, including volumes, when calling `podman kube down`.
Equivalent to the Podman `--force` option.

### LogDriver=

Set the log-driver Podman uses when running the container.
Equivalent to the Podman `--log-driver` option.

### Network=

Specify a custom network for the container. This has the same format as the `--network` option
to `podman kube play`. For example, use `host` to use the host network in the container, or `none` to
not set up networking in the container.

Special case:

- If the `name` of the network ends with `.network`, a Podman network called `systemd-$name` is used, and the generated systemd service contains a dependency on the `$name-network.service`. Such a network can be automatically created by using a `$name.network` Quadlet file. Note: the corresponding `.network` file must exist.

This key can be listed multiple times.

### PodmanArgs=

This key contains a list of arguments passed directly to the end of the `podman kube play` command
in the generated file (right before the path to the yaml file in the command line). It can be used to
access Podman features otherwise unsupported by the generator. Since the generator is unaware
of what unexpected interactions can be caused by these arguments, is not recommended to use
this option.

The format of this is a space separated list of arguments, which can optionally be individually
escaped to allow inclusion of whitespace and other control characters.

This key can be listed multiple times.

### PublishPort=

Exposes a port, or a range of ports (e.g. `50-59`), from the container to the host. Equivalent
to the `podman kube play`’s `--publish` option. The format is similar to the Podman options, which is of
the form `ip:hostPort:containerPort`, `ip::containerPort`, `hostPort:containerPort` or
`containerPort`, where the number of host and container ports must be the same (in the case
of a range).

If the IP is set to 0.0.0.0 or not set at all, the port is bound on all IPv4 addresses on
the host; use \[::\] for IPv6.

The list of published ports specified in the unit file is merged with the list of ports specified
in the Kubernetes YAML file. If the same container port and protocol is specified in both, the
entry from the unit file takes precedence

This key can be listed multiple times.

### SetWorkingDirectory=

Set the `WorkingDirectory` field of the `Service` group of the Systemd service unit file.
Used to allow `podman kube play` to correctly resolve relative paths.
Supported values are `yaml` and `unit` to set the working directory to that of the YAML or Quadlet Unit file respectively.

Alternatively, users can explicitly set the `WorkingDirectory` field of the `Service` group in the `.kube` file.
Please note that if the `WorkingDirectory` field of the `Service` group is set,
Quadlet will not set it even if `SetWorkingDirectory` is set

Special case:

- If multiple `Yaml` path are provided only `unit` is supported.

### UserNS=

Set the user namespace mode for the container. This is equivalent to the Podman `--userns` option and
generally has the form `MODE[:OPTIONS,...]`.

### Yaml=

The path, absolute or relative to the location of the unit file, to the Kubernetes YAML file to use.

This key can be listed multiple times.

