# [Swap] Section

A unit configuration file whose name ends in
" `.swap`" encodes information about a swap device
or file for memory paging controlled and supervised by
systemd.

This man page lists the configuration options specific to
this unit type. See
[systemd.unit(5)](systemd.unit.html#)
for the common options of all unit configuration files. The common
configuration items are configured in the generic \[Unit\] and
\[Install\] sections. The swap specific configuration options are
configured in the \[Swap\] section.

Additional options are listed in
[systemd.exec(5)](systemd.exec.html#),
which define the execution environment the [swapon(8)](https://man7.org/linux/man-pages/man8/swapon.8.html)
program is executed in, in
[systemd.kill(5)](systemd.kill.html#),
which define the way these processes are
terminated, and in
[systemd.resource-control(5)](systemd.resource-control.html#),
which configure resource control settings for these processes of the
unit.

Swap units must be named after the devices or files they control. Example: the swap device `/dev/sda5` must be configured in a unit file `dev-sda5.swap`. For
details about the escaping logic used to convert a file system path to a unit name, see
[systemd.unit(5)](systemd.unit.html#). Note that swap
units cannot be templated, nor is possible to add multiple names to a swap unit by creating additional symlinks to
it.

Note that swap support on Linux is privileged, swap units are hence only available in the system
service manager (and root's user service manager), but not in unprivileged user's service manager.

*Based on [systemd.swap(5)](https://www.freedesktop.org/software/systemd/man/systemd.swap.html) official documentation.*

### What=

Takes an absolute path or a fstab-style identifier of a device node or file to use
for paging. See [swapon(8)](https://man7.org/linux/man-pages/man8/swapon.8.html) for
details. If this refers to a device node, a dependency on the respective device unit is automatically
created. (See
[systemd.device(5)](systemd.device.html#)
for more information.) If this refers to a file, a dependency on the respective mount unit is
automatically created. (See
[systemd.mount(5)](systemd.mount.html#) for
more information.) This option is mandatory. Note that the usual specifier expansion is applied to
this setting, literal percent characters should hence be written as
" `%%`".

### Priority=

Swap priority to use when activating the swap
device or file. This takes an integer. This setting is
optional and ignored when the priority is set by `pri=` in the
`Options=` key.

### Options=

May contain an option string for the swap device. This may be used for controlling discard
options among other functionality, if the swap backing device supports the discard or trim operation. (See
[swapon(8)](https://man7.org/linux/man-pages/man8/swapon.8.html)
for more information.) Note that the usual specifier expansion is applied to this setting, literal percent
characters should hence be written as " `%%`".

Added in version 217.

### TimeoutSec=

Configures the time to wait for the swapon
command to finish. If a command does not exit within the
configured time, the swap will be considered failed and be
shut down again. All commands still running will be terminated
forcibly via `SIGTERM`, and after another
delay of this time with `SIGKILL`. (See
`KillMode=` in
[systemd.kill(5)](systemd.kill.html#).)
Takes a unit-less value in seconds, or a time span value such
as "5min 20s". Pass " `0`" to disable the
timeout logic. Defaults to
`DefaultTimeoutStartSec=` from the manager
configuration file (see
[systemd-system.conf(5)](systemd-system.conf.html#)).

