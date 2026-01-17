# [Mount] Section

A unit configuration file whose name ends in
" `.mount`" encodes information about a file system
mount point controlled and supervised by systemd.

This man page lists the configuration options specific to
this unit type. See
[systemd.unit(5)](systemd.unit.html#)
for the common options of all unit configuration files. The common
configuration items are configured in the generic \[Unit\] and
\[Install\] sections. The mount specific configuration options are
configured in the \[Mount\] section.

Additional options are listed in
[systemd.exec(5)](systemd.exec.html#),
which define the execution environment the
[mount(8)](https://man7.org/linux/man-pages/man8/mount.8.html)
program is executed in, and in
[systemd.kill(5)](systemd.kill.html#),
which define the way the processes are terminated, and in
[systemd.resource-control(5)](systemd.resource-control.html#),
which configure resource control settings for the processes of the
service.

Note that the options `User=` and
`Group=` are not useful for mount units.
systemd passes two parameters to
[mount(8)](https://man7.org/linux/man-pages/man8/mount.8.html);
the values of `What=` and `Where=`.
When invoked in this way,
[mount(8)](https://man7.org/linux/man-pages/man8/mount.8.html)
does not read any options from `/etc/fstab`, and
must be run as UID 0.

Mount units must be named after the mount point directories they control. Example: the mount point
`/home/lennart` must be configured in a unit file
`home-lennart.mount`. For details about the escaping logic used to convert a file
system path to a unit name, see
[systemd.unit(5)](systemd.unit.html#). Note
that mount units cannot be templated, nor is possible to add multiple names to a mount unit by creating
symlinks to its unit file.

Optionally, a mount unit may be accompanied by an automount
unit, to allow on-demand or parallelized mounting. See
[systemd.automount(5)](systemd.automount.html#).

Mount points created at runtime (independently of unit files
or `/etc/fstab`) will be monitored by systemd
and appear like any other mount unit in systemd. See
`/proc/self/mountinfo` description in
[proc(5)](https://man7.org/linux/man-pages/man5/proc.5.html).


Some file systems have special semantics as API file systems
for kernel-to-userspace and userspace-to-userspace interfaces. Some
of them may not be changed via mount units, and cannot be
disabled. For a longer discussion see [API\
File Systems](https://systemd.io/API_FILE_SYSTEMS).

The
[systemd-mount(1)](systemd-mount.html#) command
allows creating `.mount` and `.automount` units dynamically and
transiently from the command line.

*Based on [systemd.mount(5)](https://www.freedesktop.org/software/systemd/man/systemd.mount.html) official documentation.*

### What=

Takes an absolute path or a fstab-style identifier of a device node, file or
other resource to mount. See [mount(8)](https://man7.org/linux/man-pages/man8/mount.8.html) for
details. If this refers to a device node, a dependency on the respective device unit is automatically
created. (See
[systemd.device(5)](systemd.device.html#)
for more information.) This option is mandatory. Note that the usual specifier expansion is applied
to this setting, literal percent characters should hence be written as " `%%`". If this mount is a bind mount and the specified path does not exist
yet it is created as directory.

### Where=

Takes an absolute path of a file or directory for the mount point; in particular, the
destination cannot be a symbolic link. If the mount point does not exist at the time of mounting, it
is created as either a directory or a file. The former is the usual case; the latter is done only if this mount
is a bind mount and the source ( `What=`) is not a directory.
This string must be reflected in the unit filename. (See above.) This option
is mandatory.

### Type=

Takes a string for the file system type. See
[mount(8)](https://man7.org/linux/man-pages/man8/mount.8.html)
for details. This setting is optional.

If the type is " `overlay`", and " `upperdir=`" or
" `workdir=`" are specified as options and the directories do not exist, they will be created.

### Options=

Mount options to use when mounting. This takes a comma-separated list of options. This setting
is optional. Note that the usual specifier expansion is applied to this setting, literal percent characters
should hence be written as " `%%`".

### SloppyOptions=

Takes a boolean argument. If true, parsing of
the options specified in `Options=` is
relaxed, and unknown mount options are tolerated. This
corresponds with
[mount(8)](https://man7.org/linux/man-pages/man8/mount.8.html)'s
_`-s`_ switch. Defaults to
off.

Added in version 215.

### LazyUnmount=

Takes a boolean argument. If true, detach the
filesystem from the filesystem hierarchy at time of the unmount
operation, and clean up all references to the filesystem as
soon as they are not busy anymore.
This corresponds with
[umount(8)](https://man7.org/linux/man-pages/man8/umount.8.html)'s
_`-l`_ switch. Defaults to
off.

Added in version 232.

### ReadWriteOnly=

Takes a boolean argument. If false, a mount
point that shall be mounted read-write but cannot be mounted
so is retried to be mounted read-only. If true the operation
will fail immediately after the read-write mount attempt did
not succeed. This corresponds with
[mount(8)](https://man7.org/linux/man-pages/man8/mount.8.html)'s
_`-w`_ switch. Defaults to
off.

Added in version 246.

### ForceUnmount=

Takes a boolean argument. If true, force an
unmount (in case of an unreachable NFS system).
This corresponds with
[umount(8)](https://man7.org/linux/man-pages/man8/umount.8.html)'s
_`-f`_ switch. Defaults to
off.

Added in version 232.

### DirectoryMode=

Directories of mount points (and any parent
directories) are automatically created if needed. This option
specifies the file system access mode used when creating these
directories. Takes an access mode in octal notation. Defaults
to 0755.

### TimeoutSec=

Configures the time to wait for the mount
command to finish. If a command does not exit within the
configured time, the mount will be considered failed and be
shut down again. All commands still running will be terminated
forcibly via `SIGTERM`, and after another
delay of this time with `SIGKILL`. (See
`KillMode=` in
[systemd.kill(5)](systemd.kill.html#).)
Takes a unit-less value in seconds, or a time span value such
as "5min 20s". Pass 0 to disable the timeout logic. The
default value is set from `DefaultTimeoutStartSec=` option in
[systemd-system.conf(5)](systemd-system.conf.html#).

