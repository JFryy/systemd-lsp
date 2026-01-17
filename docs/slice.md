# [Slice] Section

A unit configuration file whose name ends in " `.slice`" encodes information about a slice
unit. A slice unit is a concept for hierarchically managing resources of a group of processes. This management is
performed by creating a node in the Linux Control Group (cgroup) tree. Units that manage processes (primarily scope
and service units) may be assigned to a specific slice. For each slice, certain resource limits may be set that
apply to all processes of all units contained in that slice. Slices are organized hierarchically in a tree. The
name of the slice encodes the location in the tree. The name consists of a dash-separated series of names, which
describes the path to the slice from the root slice. The root slice is named `-.slice`. Example:
`foo-bar.slice` is a slice that is located within `foo.slice`, which in turn
is located in the root slice `-.slice`.


Note that slice units cannot be templated, nor is possible to add multiple names to a slice unit by creating
additional symlinks to its unit file.

By default, service and scope units are placed in
`system.slice`, virtual machines and containers
registered with
[systemd-machined(8)](systemd-machined.html#)
are found in `machine.slice`, and user sessions
handled by
[systemd-logind(8)](systemd-logind.html#)
in `user.slice`. See
[systemd.special(7)](systemd.special.html#)
for more information.

See
[systemd.unit(5)](systemd.unit.html#)
for the common options of all unit configuration
files. The common configuration items are configured
in the generic \[Unit\] and \[Install\] sections. The
slice specific configuration options are configured in
the \[Slice\] section. Currently, only generic resource control settings
as described in
[systemd.resource-control(5)](systemd.resource-control.html#) are allowed.


See the [New\
Control Group Interfaces](https://systemd.io/CONTROL_GROUP_INTERFACE) for an introduction on how to make
use of slice units from programs.

*Based on [systemd.slice(5)](https://www.freedesktop.org/software/systemd/man/systemd.slice.html) official documentation.*

### ConcurrencyHardMax=

Configures a hard and a soft limit on the maximum number of units assigned to this
slice (or any descendent slices) that may be active at the same time. If the hard limit is reached no
further units associated with the slice may be activated, and their activation will fail with an
error. If the soft limit is reached any further requested activation of units will be queued, but no
immediate error is generated. The queued activation job will remain queued until the number of
concurrent active units within the slice is below the limit again.

If the special value " `infinity`" is specified, no concurrency limit is
enforced. This is the default.

Note that if multiple start jobs are queued for units, and all their dependencies are fulfilled
they'll be processed in an order that is dependent on the unit type, the CPU weight (for unit types
that know the concept, such as services), the nice level (similar), and finally in alphabetical order
by the unit name. This may be used to influence dispatching order when using
`ConcurrencySoftMax=` to pace concurrency within a slice unit.

Note that these options have a hierarchial effect: a limit set for a slice unit will apply to
both the units immediately within the slice, but also all units further down the slice tree. Also
note that each sub-slice unit counts as one unit each too, and thus when choosing a limit for a slice
hierarchy the limit must provide room for both the payload units (i.e. services, mounts, …) and
structural units (i.e. slice units), if any are defined.

Added in version 258.

### ConcurrencySoftMax=

Configures a hard and a soft limit on the maximum number of units assigned to this
slice (or any descendent slices) that may be active at the same time. If the hard limit is reached no
further units associated with the slice may be activated, and their activation will fail with an
error. If the soft limit is reached any further requested activation of units will be queued, but no
immediate error is generated. The queued activation job will remain queued until the number of
concurrent active units within the slice is below the limit again.

If the special value " `infinity`" is specified, no concurrency limit is
enforced. This is the default.

Note that if multiple start jobs are queued for units, and all their dependencies are fulfilled
they'll be processed in an order that is dependent on the unit type, the CPU weight (for unit types
that know the concept, such as services), the nice level (similar), and finally in alphabetical order
by the unit name. This may be used to influence dispatching order when using
`ConcurrencySoftMax=` to pace concurrency within a slice unit.

Note that these options have a hierarchial effect: a limit set for a slice unit will apply to
both the units immediately within the slice, but also all units further down the slice tree. Also
note that each sub-slice unit counts as one unit each too, and thus when choosing a limit for a slice
hierarchy the limit must provide room for both the payload units (i.e. services, mounts, …) and
structural units (i.e. slice units), if any are defined.

Added in version 258.

