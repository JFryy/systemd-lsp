# [Container] Section

Container units are named with a `.container` extension and contain a `[Container]` section describing
the container that is run as a service. The resulting service file contains a line like
`ExecStart=podman run … image-name`, and most of the keys in this section control the command-line
options passed to Podman. However, some options also affect the details of how systemd is set up to run and
interact with the container.

By default, the Podman container has the same name as the unit, but with a `systemd-` prefix, i.e.
a `$name.container` file creates a `$name.service` unit and a `systemd-$name` Podman container. The
`ContainerName` option allows for overriding this default name with a user-provided one.

There is only one required key, `Image`, which defines the container image the service runs.

Valid options for `[Container]` are listed below:

*Based on [podman-systemd.unit(5)](https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html) official documentation.*

### AddCapability=

Add these capabilities, in addition to the default Podman capability set, to the container.

This is a space separated list of capabilities. This key can be listed multiple times.

For example:

```
AddCapability=CAP_DAC_OVERRIDE CAP_IPC_OWNER

```

### AddDevice=

Adds a device node from the host into the container. The format of this is
`HOST-DEVICE[:CONTAINER-DEVICE][:PERMISSIONS]`, where `HOST-DEVICE` is the path of
the device node on the host, `CONTAINER-DEVICE` is the path of the device node in
the container, and `PERMISSIONS` is a list of permissions combining ‘r’ for read,
‘w’ for write, and ‘m’ for mknod(2). The `-` prefix tells Quadlet to add the device
only if it exists on the host.

This key can be listed multiple times.

### AddHost=

Add host-to-IP mapping to /etc/hosts.
The format is `hostname:ip`.

Equivalent to the Podman `--add-host` option.
This key can be listed multiple times.

### Annotation=

Set one or more OCI annotations on the container. The format is a list of `key=value` items,
similar to `Environment`.

This key can be listed multiple times.

### AppArmor=

Sets the apparmor confinement profile for the container. A value of `unconfined` turns off apparmor confinement.

### AutoUpdate=

Indicates whether the container will be auto-updated ( [podman-auto-update(1)](podman-auto-update.1.html)). The following values are supported:

- `registry`: Requires a fully-qualified image reference (e.g., quay.io/podman/stable:latest) to be used to create the container. This enforcement is necessary to know which image to actually check and pull. If an image ID was used, Podman does not know which image to check/pull anymore.

- `local`: Tells Podman to compare the image a container is using to the image with its raw name in local storage. If an image is updated locally, Podman simply restarts the systemd unit executing the container.

### CgroupsMode=

The cgroups mode of the Podman container. Equivalent to the Podman `--cgroups` option.

By default, the cgroups mode of the container created by Quadlet is `split`,
which differs from the default ( `enabled`) used by the Podman CLI.

If the container joins a pod (i.e. `Pod=` is specified), you may want to change this to
`no-conmon` or `enabled` so that pod level cgroup resource limits can take effect.

### ContainerName=

The (optional) name of the Podman container. If this is not specified, the default value
of `systemd-%N` is used, which is the same as the service name but with a `systemd-`
prefix to avoid conflicts with user-managed containers.

### ContainersConfModule=

Load the specified containers.conf(5) module. Equivalent to the Podman `--module` option.

This key can be listed multiple times.

### DNS=

Set network-scoped DNS resolver/nameserver for containers in this network.

This key can be listed multiple times.

### DNSOption=

Set custom DNS options.

This key can be listed multiple times.

### DNSSearch=

Set custom DNS search domains. Use **DNSSearch=.** to remove the search domain.

This key can be listed multiple times.

### DropCapability=

Drop these capabilities from the default podman capability set, or `all` to drop all capabilities.

This is a space separated list of capabilities. This key can be listed multiple times.

For example:

```
DropCapability=CAP_DAC_OVERRIDE CAP_IPC_OWNER

```

### Entrypoint=

Override the default ENTRYPOINT from the image.
Equivalent to the Podman `--entrypoint` option.
Specify multi option commands in the form of a JSON string.

### Environment=

Set an environment variable in the container. This uses the same format as
[services in systemd](https://www.freedesktop.org/software/systemd/man/systemd.exec.html#Environment=)
and can be listed multiple times.

### EnvironmentFile=

Use a line-delimited file to set environment variables in the container.
The path may be absolute or relative to the location of the unit file.
This key may be used multiple times, and the order persists when passed to `podman run`.

### EnvironmentHost=

Use the host environment inside of the container.

### Exec=

Additional arguments for the container; this has exactly the same effect as passing
more arguments after a `podman run <image> <arguments>` invocation.

The format is the same as for [systemd command lines](https://www.freedesktop.org/software/systemd/man/systemd.service.html#Command%20lines),
However, unlike the usage scenario for similarly-named systemd `ExecStart=` verb
which operates on the ambient root filesystem, it is very common for container
images to have their own `ENTRYPOINT` or `CMD` metadata which this interacts with.

The default expectation for many images is that the image will include an `ENTRYPOINT`
with a default binary, and this field will add arguments to that entrypoint.

Another way to describe this is that it works the same way as the [args field in a Kubernetes pod](https://kubernetes.io/docs/tasks/inject-data-application/define-command-argument-container/#running-a-command-in-a-shell).

### ExposeHostPort=

Exposes a port, or a range of ports (e.g. `50-59`), from the host to the container. Equivalent
to the Podman `--expose` option.

This key can be listed multiple times.

### GIDMap=

Run the container in a new user namespace using the supplied GID mapping.
Equivalent to the Podman `--gidmap` option.

This key can be listed multiple times.

### GlobalArgs=

This key contains a list of arguments passed directly between `podman` and `run`
in the generated file. It can be used to access Podman features otherwise unsupported by the generator. Since the generator is unaware
of what unexpected interactions can be caused by these arguments, it is not recommended to use
this option.

The format of this is a space separated list of arguments, which can optionally be individually
escaped to allow inclusion of whitespace and other control characters.

This key can be listed multiple times.

### Group=

The (numeric) GID to run as inside the container. This does not need to match the GID on the host,
which can be modified with `UserNS`, but if that is not specified, this GID is also used on the host.

Note: when both `User=` and `Group=` are specified, they are combined into a single `--user USER:GROUP`
argument passed to Podman. Using `Group=` without `User=` will result in an error.

### GroupAdd=

Assign additional groups to the primary user running within the container process. Also supports the `keep-groups` special flag.
Equivalent to the Podman `--group-add` option.

### HealthCmd=

Set or alter a healthcheck command for a container. A value of none disables existing healthchecks.
Equivalent to the Podman `--health-cmd` option.

### HealthInterval=

Set an interval for the healthchecks. An interval of disable results in no automatic timer setup.
Equivalent to the Podman `--health-interval` option.

### HealthLogDestination=

Set the destination of the HealthCheck log. Directory path, local or events\_logger (local use container state file)
(Default: local)
Equivalent to the Podman `--health-log-destination` option.

- `local`: (default) HealthCheck logs are stored in overlay containers. (For example: `$runroot/healthcheck.log`)

- `directory`: creates a log file named `<container-ID>-healthcheck.log` with HealthCheck logs in the specified directory.

- `events_logger`: The log will be written with logging mechanism set by events\_logger. It also saves the log to a default directory, for performance on a system with a large number of logs.

### HealthMaxLogCount=

Set maximum number of attempts in the HealthCheck log file. (‘0’ value means an infinite number of attempts in the log file)
(Default: 5 attempts)
Equivalent to the Podman `--Health-max-log-count` option.

### HealthMaxLogSize=

Set maximum length in characters of stored HealthCheck log. (“0” value means an infinite log length)
(Default: 500 characters)
Equivalent to the Podman `--Health-max-log-size` option.

### HealthOnFailure=

Action to take once the container transitions to an unhealthy state.
The “kill” action in combination integrates best with systemd. Once
the container turns unhealthy, it gets killed, and systemd restarts the
service.
Equivalent to the Podman `--health-on-failure` option.

### HealthRetries=

The number of retries allowed before a healthcheck is considered to be unhealthy.
Equivalent to the Podman `--health-retries` option.

### HealthStartPeriod=

The initialization time needed for a container to bootstrap.
Equivalent to the Podman `--health-start-period` option.

### HealthStartupCmd=

Set a startup healthcheck command for a container.
Equivalent to the Podman `--health-startup-cmd` option.

### HealthStartupInterval=

Set an interval for the startup healthcheck. An interval of disable results in no automatic timer setup.
Equivalent to the Podman `--health-startup-interval` option.

### HealthStartupRetries=

The number of attempts allowed before the startup healthcheck restarts the container.
Equivalent to the Podman `--health-startup-retries` option.

### HealthStartupSuccess=

The number of successful runs required before the startup healthcheck succeeds and the regular healthcheck begins.
Equivalent to the Podman `--health-startup-success` option.

### HealthStartupTimeout=

The maximum time a startup healthcheck command has to complete before it is marked as failed.
Equivalent to the Podman `--health-startup-timeout` option.

### HealthTimeout=

The maximum time allowed to complete the healthcheck before an interval is considered failed.
Equivalent to the Podman `--health-timeout` option.

### HostName=

Sets the host name that is available inside the container.
Equivalent to the Podman `--hostname` option.

### HttpProxy=

Controls whether proxy environment variables (http\_proxy, https\_proxy, ftp\_proxy, no\_proxy) are passed from the Podman process into the container during image pulls and builds.

Set to `true` to enable proxy inheritance (default Podman behavior) or `false` to disable it.
This option is particularly useful on systems that require proxy configuration for internet access but don’t want proxy settings passed to the container runtime.

Equivalent to the Podman `--http-proxy` option.

### Image=

The image to run in the container.
It is recommended to use a fully qualified image name rather than a short name, both for
performance and robustness reasons.

The format of the name is the same as when passed to `podman pull`. So, it supports using
`:tag` or digests to guarantee the specific image version.

Special Cases:

- If the `name` of the image ends with `.image`, Quadlet will use the image pulled by the corresponding `.image` file, and the generated systemd service contains a dependency on the `$name-image.service` (or the service name set in the .image file). Note that the corresponding `.image` file must exist.

- If the `name` of the image ends with `.build`, Quadlet will use the image built by the corresponding `.build` file, and the generated systemd service contains a dependency on the `$name-build.service`. Note: the corresponding `.build` file must exist.

### IP=

Specify a static IPv4 address for the container, for example **10.88.64.128**.
Equivalent to the Podman `--ip` option.

### IP6=

Specify a static IPv6 address for the container, for example **fd46:db93:aa76:ac37::10**.
Equivalent to the Podman `--ip6` option.

### Label=

Set one or more OCI labels on the container. The format is a list of `key=value` items,
similar to `Environment`.

This key can be listed multiple times.

### LogDriver=

Set the log-driver used by Podman when running the container.
Equivalent to the Podman `--log-driver` option.

### LogOpt=

Set the log-opt (logging options) used by Podman when running the container.
Equivalent to the Podman `--log-opt` option.
This key can be listed multiple times.

### Mask=

Specify the paths to mask separated by a colon. `Mask=/path/1:/path/2`. A masked path cannot be accessed inside the container.

### Memory=

Specify the amount of memory for the container.

### Mount=

Attach a filesystem mount to the container.
This is equivalent to the Podman `--mount` option, and
generally has the form `type=TYPE,TYPE-SPECIFIC-OPTION[,...]`.

Special cases:

- For `type=volume`, if `source` ends with `.volume`, the Podman named volume generated by the corresponding `.volume` file is used.

- For `type=image`, if `source` ends with `.image`, the image generated by the corresponding `.image` file is used.

In both cases, the generated systemd service will contain a dependency on the service generated for the corresponding unit. Note: the corresponding `.volume` or `.image` file must exist.

This key can be listed multiple times.

### Network=

Specify a custom network for the container. This has the same format as the `--network` option
to `podman run`. For example, use `host` to use the host network in the container, or `none` to
not set up networking in the container.

Special cases:

- If the `name` of the network ends with `.network`, a Podman network called
  `systemd-$name` is used, and the generated systemd service contains
  a dependency on the `$name-network.service`. Such a network can be automatically
  created by using a `$name.network` Quadlet file. Note: the corresponding `.network` file must exist.

- If the `name` ends with `.container`,
  the container will reuse the network stack of another container created by `$name.container`.
  The generated systemd service contains a dependency on `$name.service`. Note: the corresponding `.container` file must exist.

This key can be listed multiple times.

### NetworkAlias=

Add a network-scoped alias for the container. This has the same format as the `--network-alias`
option to `podman run`. Aliases can be used to group containers together in DNS resolution: for
example, setting `NetworkAlias=web` on multiple containers will make a DNS query for `web` resolve
to all the containers with that alias.

This key can be listed multiple times.

### NoNewPrivileges=

If enabled, this disables the container processes from gaining additional privileges via things like
setuid and file capabilities.

### Notify=

By default, Podman is run in such a way that the systemd startup notify command is handled by
the container runtime. In other words, the service is deemed started when the container runtime
starts the child in the container. However, if the container application supports
[sd\_notify](https://www.freedesktop.org/software/systemd/man/sd_notify.html), then setting
`Notify` to true passes the notification details to the container allowing it to notify
of startup on its own.

In addition, setting `Notify` to `healthy` will postpone startup notifications until such time as
the container is marked healthy, as determined by Podman healthchecks. Note that this requires
setting up a container healthcheck, see the `HealthCmd` option for more.

### PidsLimit=

Tune the container’s pids limit.
This is equivalent to the Podman `--pids-limit` option.

### Pod=

Specify a Quadlet `.pod` unit to link the container to.
The value must take the form of `<name>.pod` and the `.pod` unit must exist.

Quadlet will add all the necessary parameters to link between the container and the pod and between their corresponding services.

### PodmanArgs=

This key contains a list of arguments passed directly to the end of the `podman run` command
in the generated file (right before the image name in the command line). It can be used to
access Podman features otherwise unsupported by the generator. Since the generator is unaware
of what unexpected interactions can be caused by these arguments, it is not recommended to use
this option.

The format of this is a space separated list of arguments, which can optionally be individually
escaped to allow inclusion of whitespace and other control characters.

This key can be listed multiple times.

### PublishPort=

Exposes a port, or a range of ports (e.g. `50-59`), from the container to the host. Equivalent
to the Podman `--publish` option. The format is similar to the Podman options, which is of
the form `ip:hostPort:containerPort`, `ip::containerPort`, `hostPort:containerPort` or
`containerPort`, where the number of host and container ports must be the same (in the case
of a range).

If the IP is set to 0.0.0.0 or not set at all, the port is bound on all IPv4 addresses on
the host; use \[::\] for IPv6.

Note that not listing a host port means that Podman automatically selects one, and it
may be different for each invocation of service. This makes that a less useful option. The
allocated port can be found with the `podman port` command.

This key can be listed multiple times.

### Pull=

Set the image pull policy.
This is equivalent to the Podman `--pull` option

### ReadOnly=

If enabled, makes the image read-only.

### ReadOnlyTmpfs=

If ReadOnly is set to `true`, mount a read-write tmpfs on /dev, /dev/shm, /run, /tmp, and /var/tmp.

### ReloadCmd=

Add `ExecReload` line to the `Service` that runs ` podman exec` with this command in this container.

In order to execute the reload run `systemctl reload <Service>`

Mutually exclusive with `ReloadSignal`

### ReloadSignal=

Add `ExecReload` line to the `Service` that runs `podman kill` with this signal which sends the signal to the main container process.

In order to execute the reload run `systemctl reload <Service>`

Mutually exclusive with `ReloadCmd`

### Retry=

Number of times to retry the image pull when a HTTP error occurs. Equivalent to the Podman `--retry` option.

### RetryDelay=

Delay between retries. Equivalent to the Podman `--retry-delay` option.

### Rootfs=

The rootfs to use for the container. Rootfs points to a directory on the system that contains the content to be run within the container. This option conflicts with the `Image` option.

The format of the rootfs is the same as when passed to `podman run --rootfs`, so it supports overlay mounts as well.

Note: On SELinux systems, the rootfs needs the correct label, which is by default unconfined\_u:object\_r:container\_file\_t:s0.

### RunInit=

If enabled, the container has a minimal init process inside the
container that forwards signals and reaps processes.

### SeccompProfile=

Set the seccomp profile to use in the container. If unset, the default podman profile is used.
Set to either the pathname of a JSON file, or `unconfined` to disable the seccomp filters.

### Secret=

Use a Podman secret in the container either as a file or an environment variable.
This is equivalent to the Podman `--secret` option and generally has the form `secret[,opt=opt ...]`

### SecurityLabelDisable=

Turn off label separation for the container.

### SecurityLabelFileType=

Set the label file type for the container files.

### SecurityLabelLevel=

Set the label process level for the container processes.

### SecurityLabelNested=

Allow SecurityLabels to function within the container. This allows separation of containers created within the container.

### SecurityLabelType=

Set the label process type for the container processes.

### ShmSize=

Size of /dev/shm.

This is equivalent to the Podman `--shm-size` option and generally has the form `number[unit]`

### StartWithPod=

Start the container after the associated pod is created. Default to **true**.

If `true`, container will be started/stopped/restarted alongside the pod.

If `false`, the container will not be started when the pod starts. The container will be stopped with the pod. Restarting the pod will also restart the container as long as the container was also running before.

Note, the container can still be started manually or through a target by configuring the `[Install]` section. The pod will be started as needed in any case.

### StopSignal=

Signal to stop a container. Default is **SIGTERM**.

This is equivalent to the Podman `--stop-signal` option

### StopTimeout=

Seconds to wait before forcibly stopping the container.

Note, this value should be lower than the actual systemd unit timeout to make sure the podman rm command is not killed by systemd.

This is equivalent to the Podman `--stop-timeout` option

### SubGIDMap=

Run the container in a new user namespace using the map with name in the /etc/subgid file.
Equivalent to the Podman `--subgidname` option.

### SubUIDMap=

Run the container in a new user namespace using the map with name in the /etc/subuid file.
Equivalent to the Podman `--subuidname` option.

### Sysctl=

Configures namespaced kernel parameters for the container. The format is `Sysctl=name=value`.

This is a space separated list of kernel parameters. This key can be listed multiple times.

For example:

```
Sysctl=net.ipv6.conf.all.disable_ipv6=1 net.ipv6.conf.all.use_tempaddr=1

```

### Timezone=

The timezone to run the container in.

### Tmpfs=

Mount a tmpfs in the container. This is equivalent to the Podman `--tmpfs` option, and
generally has the form `CONTAINER-DIR[:OPTIONS]`.

This key can be listed multiple times.

### UIDMap=

Run the container in a new user namespace using the supplied UID mapping.
Equivalent to the Podman `--uidmap` option.

This key can be listed multiple times.

### Ulimit=

Ulimit options. Sets the ulimits values inside of the container.

This key can be listed multiple times.

### Unmask=

Specify the paths to unmask separated by a colon. unmask=ALL or /path/1:/path/2, or shell expanded paths (/proc/\*):

If set to `ALL`, Podman will unmask all the paths that are masked or made read-only by default.

The default masked paths are /proc/acpi, /proc/kcore, /proc/keys, /proc/latency\_stats, /proc/sched\_debug, /proc/scsi, /proc/timer\_list, /proc/timer\_stats, /sys/firmware, and /sys/fs/selinux.

The default paths that are read-only are /proc/asound, /proc/bus, /proc/fs, /proc/irq, /proc/sys, /proc/sysrq-trigger, /sys/fs/cgroup.

### User=

The (numeric) UID to run as inside the container. This does not need to match the UID on the host,
which can be modified with `UserNS`, but if that is not specified, this UID is also used on the host.

Note: when both `User=` and `Group=` are specified, they are combined into a single `--user USER:GROUP`
argument passed to Podman.

### UserNS=

Set the user namespace mode for the container. This is equivalent to the Podman `--userns` option and
generally has the form `MODE[:OPTIONS,...]`.

### Volume=

Mount a volume in the container. This is equivalent to the Podman `--volume` option, and
generally has the form `[[SOURCE-VOLUME|HOST-DIR:]CONTAINER-DIR[:OPTIONS]]`.

If `SOURCE-VOLUME` starts with `.`, Quadlet resolves the path relative to the location of the unit file.

Special case:

- If `SOURCE-VOLUME` ends with `.volume`, a Podman named volume called `systemd-$name` is used as the source, and the generated systemd service contains a dependency on the `$name-volume.service`. Note that the corresponding `.volume` file must exist.

This key can be listed multiple times.

### WorkingDir=

Working directory inside the container.

The default working directory for running binaries within a container is the root directory (/). The image developer can set a different default with the WORKDIR instruction. This option overrides the working directory by using the -w option.

