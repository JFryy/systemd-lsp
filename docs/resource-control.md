# [ResourceControl] Section

Unit configuration files for services, slices, scopes, sockets, mount points, and swap devices share a subset
of configuration options for resource control of spawned processes. Internally, this relies on the Linux Control
Groups (cgroups) kernel concept for organizing processes in a hierarchical tree of named groups for the purpose of
resource management.

This man page lists the configuration options shared by
those six unit types. See
[systemd.unit(5)](systemd.unit.html#)
for the common options of all unit configuration files, and
[systemd.slice(5)](systemd.slice.html#),
[systemd.scope(5)](systemd.scope.html#),
[systemd.service(5)](systemd.service.html#),
[systemd.socket(5)](systemd.socket.html#),
[systemd.mount(5)](systemd.mount.html#),
and
[systemd.swap(5)](systemd.swap.html#)
for more information on the specific unit configuration files. The
resource control configuration options are configured in the
\[Slice\], \[Scope\], \[Service\], \[Socket\], \[Mount\], or \[Swap\]
sections, depending on the unit type.

In addition, options which control resources available to programs
_executed_ by systemd are listed in
[systemd.exec(5)](systemd.exec.html#).
Those options complement options listed here.

### Enabling and disabling controllers [¶](\#Enabling%20and%20disabling%20controllers "Permalink to this headline")

Controllers in the cgroup hierarchy are hierarchical, and resource control is realized by
distributing resource assignments between siblings in branches of the cgroup hierarchy. There is no
need to explicitly _enable_ a cgroup controller for a unit.
**systemd** will instruct the kernel to enable a controller for a given unit when this
unit has configuration for a given controller. For example, when `CPUWeight=` is set,
the `cpu` controller will be enabled, and when `TasksMax=` are set, the
`pids` controller will be enabled. In addition, various controllers may be also be
enabled explicitly via the
`MemoryAccounting=`/ `TasksAccounting=`/ `IOAccounting=`
settings. Because of how the cgroup hierarchy works, controllers will be automatically enabled for all
parent units and for any sibling units starting with the lowest level at which a controller is enabled.
Units for which a controller is enabled may be subject to resource control even if they do not have any
explicit configuration.

Setting `Delegate=` enables any delegated controllers for that unit (see below).
The delegatee may then enable controllers for its children as appropriate. In particular, if the
delegatee is **systemd** (in the `user@.service` unit), it will
repeat the same logic as the system instance and enable controllers for user units which have resource
limits configured, and their siblings and parents and parents' siblings.

Controllers may be _disabled_ for parts of the cgroup hierarchy with
`DisableControllers=` (see below).

**Example 1. Enabling and disabling controllers**

```
                      -.slice
                     /       \
              /-----/         \--------------\
             /                                \
      system.slice                       user.slice
        /       \                          /      \
       /         \                        /        \
      /           \              user@42.service  user@1000.service
     /             \             Delegate=        Delegate=yes
a.service       b.slice                             /        \
CPUWeight=20   DisableControllers=cpu              /          \
                 /  \                      app.slice      session.slice
                /    \                     CPUWeight=100  CPUWeight=100
               /      \
       b1.service   b2.service
                    CPUWeight=1000

```

In this hierarchy, the `cpu` controller is enabled for all units shown except
`b1.service` and `b2.service`. Because there is no explicit
configuration for `system.slice` and `user.slice`, CPU
resources will be split equally between them. Similarly, resources are allocated equally between
children of `user.slice` and between the child slices beneath
`user@1000.service`. Assuming that there is no further configuration of resources
or delegation below slices `app.slice` or `session.slice`, the
`cpu` controller would not be enabled for units in those slices and CPU resources
would be further allocated using other mechanisms, e.g. based on nice levels. The manager for user
42 has delegation enabled without any controllers, i.e. it can manipulate its subtree of the cgroup
hierarchy, but without resource control.

In the slice `system.slice`, CPU resources are split 1:6 for service
`a.service`, and 5:6 for slice `b.slice`, because slice
`b.slice` gets the default value of 100 for `cpu.weight` when
`CPUWeight=` is not set.

`CPUWeight=` setting in service `b2.service` is neutralized
by `DisableControllers=` in slice `b.slice`, so the
`cpu` controller would not be enabled for services `b1.service` and
`b2.service`, and CPU resources would be further allocated using other mechanisms,
e.g. based on nice levels.

### Setting resource controls for a group of related units [¶](\#Setting%20resource%20controls%20for%20a%20group%20of%20related%20units "Permalink to this headline")

As described in
[systemd.unit(5)](systemd.unit.html#), the
settings listed here may be set through the main file of a unit and drop-in snippets in
`*.d/` directories. The list of directories searched for drop-ins
includes names formed by repeatedly truncating the unit name after all dashes. This is particularly
convenient to set resource limits for a group of units with similar names.

For example, every user gets their own slice
`user-nnn.slice`. Drop-ins with local configuration that
affect user 1000 may be placed in
`/etc/systemd/system/user-1000.slice`,
`/etc/systemd/system/user-1000.slice.d/*.conf`, but also
`/etc/systemd/system/user-.slice.d/*.conf`. This last directory
applies to all user slices.

### ¶

See the [New\
Control Group Interfaces](https://systemd.io/CONTROL_GROUP_INTERFACE) for an introduction on how to make
use of resource control APIs from programs.

*Based on [systemd.resource-control(5)](https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html) official documentation.*

### CPUWeight=

These settings control the `cpu` controller in the unified hierarchy.

These options accept an integer value or the special string "idle":

- If set to an integer value, assign the specified CPU time weight to the processes
executed, if the unified control group hierarchy is used on the system. These options control
the " `cpu.weight`" control group attribute. The allowed range is 1 to 10000.
Defaults to unset, but the kernel default is 100. For details about this control group
attribute, see [Control Groups\
v2](https://docs.kernel.org/admin-guide/cgroup-v2.html) and [CFS\
Scheduler](https://docs.kernel.org/scheduler/sched-design-CFS.html). The available CPU time is split up among all units within one slice
relative to their CPU time weight. A higher weight means more CPU time, a lower weight means
less.

- If set to the special string "idle", mark the cgroup for "idle scheduling", which means
that it will get CPU resources only when there are no processes not marked in this way to execute in this
cgroup or its siblings. This setting corresponds to the " `cpu.idle`" cgroup attribute.

Note that this value only has an effect on cgroup-v2, for cgroup-v1 it is equivalent to the minimum weight.


While `StartupCPUWeight=` applies to the startup and shutdown phases of the system,
`CPUWeight=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupCPUWeight=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

In addition to the resource allocation performed by the `cpu` controller, the
kernel may automatically divide resources based on session-id grouping, see "The autogroup feature"
in [sched(7)](https://man7.org/linux/man-pages/man7/sched.7.html).
The effect of this feature is similar to the `cpu` controller with no explicit
configuration, so users should be careful to not mistake one for the other.

Added in version 232.

### StartupCPUWeight=

These settings control the `cpu` controller in the unified hierarchy.

These options accept an integer value or the special string "idle":

- If set to an integer value, assign the specified CPU time weight to the processes
executed, if the unified control group hierarchy is used on the system. These options control
the " `cpu.weight`" control group attribute. The allowed range is 1 to 10000.
Defaults to unset, but the kernel default is 100. For details about this control group
attribute, see [Control Groups\
v2](https://docs.kernel.org/admin-guide/cgroup-v2.html) and [CFS\
Scheduler](https://docs.kernel.org/scheduler/sched-design-CFS.html). The available CPU time is split up among all units within one slice
relative to their CPU time weight. A higher weight means more CPU time, a lower weight means
less.

- If set to the special string "idle", mark the cgroup for "idle scheduling", which means
that it will get CPU resources only when there are no processes not marked in this way to execute in this
cgroup or its siblings. This setting corresponds to the " `cpu.idle`" cgroup attribute.

Note that this value only has an effect on cgroup-v2, for cgroup-v1 it is equivalent to the minimum weight.


While `StartupCPUWeight=` applies to the startup and shutdown phases of the system,
`CPUWeight=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupCPUWeight=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

In addition to the resource allocation performed by the `cpu` controller, the
kernel may automatically divide resources based on session-id grouping, see "The autogroup feature"
in [sched(7)](https://man7.org/linux/man-pages/man7/sched.7.html).
The effect of this feature is similar to the `cpu` controller with no explicit
configuration, so users should be careful to not mistake one for the other.

Added in version 232.

### CPUQuota=

This setting controls the `cpu` controller in the unified hierarchy.

Assign the specified CPU time quota to the processes executed. Takes a percentage value, suffixed with
"%". The percentage specifies how much CPU time the unit shall get at maximum, relative to the total CPU time
available on one CPU. Use values > 100% for allotting CPU time on more than one CPU. This controls the
" `cpu.max`" attribute on the unified control group hierarchy and
" `cpu.cfs_quota_us`" on legacy. For details about these control group attributes, see [Control Groups v2](https://docs.kernel.org/admin-guide/cgroup-v2.html) and [CFS Bandwidth Control](https://docs.kernel.org/scheduler/sched-bwc.html).
Setting `CPUQuota=` to an empty value unsets the quota.

Example: `CPUQuota=20%` ensures that the executed processes will never get more than
20% CPU time on one CPU.

Added in version 213.

### CPUQuotaPeriodSec=

This setting controls the `cpu` controller in the unified hierarchy.

Assign the duration over which the CPU time quota specified by `CPUQuota=` is measured.
Takes a time duration value in seconds, with an optional suffix such as "ms" for milliseconds (or "s" for seconds.)
The default setting is 100ms. The period is clamped to the range supported by the kernel, which is \[1ms, 1000ms\].
Additionally, the period is adjusted up so that the quota interval is also at least 1ms.
Setting `CPUQuotaPeriodSec=` to an empty value resets it to the default.

This controls the second field of " `cpu.max`" attribute on the unified control group hierarchy
and " `cpu.cfs_period_us`" on legacy. For details about these control group attributes, see
[Control Groups v2](https://docs.kernel.org/admin-guide/cgroup-v2.html) and
[CFS Scheduler](https://docs.kernel.org/scheduler/sched-design-CFS.html).

Example: `CPUQuotaPeriodSec=10ms` to request that the CPU quota is measured in periods of 10ms.

Added in version 242.

### AllowedCPUs=

This setting controls the `cpuset` controller in the unified hierarchy.

Restrict processes to be executed on specific CPUs. Takes a list of CPU indices or ranges separated by either
whitespace or commas. CPU ranges are specified by the lower and upper CPU indices separated by a dash.

Setting `AllowedCPUs=` or `StartupAllowedCPUs=` does not guarantee that all
of the CPUs will be used by the processes as it may be limited by parent units. The effective configuration is
reported as `EffectiveCPUs=`.

While `StartupAllowedCPUs=` applies to the startup and shutdown phases of the system,
`AllowedCPUs=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupAllowedCPUs=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

This setting is supported only with the unified control group hierarchy.

Added in version 244.

### StartupAllowedCPUs=

This setting controls the `cpuset` controller in the unified hierarchy.

Restrict processes to be executed on specific CPUs. Takes a list of CPU indices or ranges separated by either
whitespace or commas. CPU ranges are specified by the lower and upper CPU indices separated by a dash.

Setting `AllowedCPUs=` or `StartupAllowedCPUs=` does not guarantee that all
of the CPUs will be used by the processes as it may be limited by parent units. The effective configuration is
reported as `EffectiveCPUs=`.

While `StartupAllowedCPUs=` applies to the startup and shutdown phases of the system,
`AllowedCPUs=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupAllowedCPUs=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

This setting is supported only with the unified control group hierarchy.

Added in version 244.

### MemoryAccounting=

This setting controls the `memory` controller in the unified hierarchy.

Turn on process and kernel memory accounting for this
unit. Takes a boolean argument. Note that turning on memory
accounting for one unit will also implicitly turn it on for
all units contained in the same slice and for all its parent
slices and the units contained therein. The system default
for this setting may be controlled with
`DefaultMemoryAccounting=` in
[systemd-system.conf(5)](systemd-system.conf.html#).

Added in version 208.

### MemoryMin=

These settings control the `memory` controller in the unified hierarchy.

Specify the memory usage protection of the executed processes in this unit.
When reclaiming memory, the unit is treated as if it was using less memory resulting in memory
to be preferentially reclaimed from unprotected units.
Using `MemoryLow=` results in a weaker protection where memory may still
be reclaimed to avoid invoking the OOM killer in case there is no other reclaimable memory.

For a protection to be effective, it is generally required to set a corresponding
allocation on all ancestors, which is then distributed between children
(with the exception of the root slice).
Any `MemoryMin=` or `MemoryLow=` allocation that is not
explicitly distributed to specific children is used to create a shared protection for all children.
As this is a shared protection, the children will freely compete for the memory.

Takes a memory size in bytes. If the value is suffixed with K, M, G or T, the specified memory size is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes (with the base 1024), respectively. Alternatively, a
percentage value may be specified, which is taken relative to the installed physical memory on the
system. If assigned the special value " `infinity`", all available memory is protected, which may be
useful in order to always inherit all of the protection afforded by ancestors.
This controls the " `memory.min`" or " `memory.low`" control group attribute.
For details about this control group attribute, see [Memory Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files).

Units may have their children use a default " `memory.min`" or
" `memory.low`" value by specifying `DefaultMemoryMin=` or
`DefaultMemoryLow=`, which has the same semantics as
`MemoryMin=` and `MemoryLow=`, or `DefaultStartupMemoryLow=`
which has the same semantics as `StartupMemoryLow=`.
This setting does not affect " `memory.min`" or " `memory.low`"
in the unit itself.
Using it to set a default child allocation is only useful on kernels older than 5.7,
which do not support the " `memory_recursiveprot`" cgroup2 mount option.

While `StartupMemoryLow=` applies to the startup and shutdown phases of the system,
`MemoryMin=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupMemoryLow=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

Added in version 240.

### MemoryLow=

These settings control the `memory` controller in the unified hierarchy.

Specify the memory usage protection of the executed processes in this unit.
When reclaiming memory, the unit is treated as if it was using less memory resulting in memory
to be preferentially reclaimed from unprotected units.
Using `MemoryLow=` results in a weaker protection where memory may still
be reclaimed to avoid invoking the OOM killer in case there is no other reclaimable memory.

For a protection to be effective, it is generally required to set a corresponding
allocation on all ancestors, which is then distributed between children
(with the exception of the root slice).
Any `MemoryMin=` or `MemoryLow=` allocation that is not
explicitly distributed to specific children is used to create a shared protection for all children.
As this is a shared protection, the children will freely compete for the memory.

Takes a memory size in bytes. If the value is suffixed with K, M, G or T, the specified memory size is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes (with the base 1024), respectively. Alternatively, a
percentage value may be specified, which is taken relative to the installed physical memory on the
system. If assigned the special value " `infinity`", all available memory is protected, which may be
useful in order to always inherit all of the protection afforded by ancestors.
This controls the " `memory.min`" or " `memory.low`" control group attribute.
For details about this control group attribute, see [Memory Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files).

Units may have their children use a default " `memory.min`" or
" `memory.low`" value by specifying `DefaultMemoryMin=` or
`DefaultMemoryLow=`, which has the same semantics as
`MemoryMin=` and `MemoryLow=`, or `DefaultStartupMemoryLow=`
which has the same semantics as `StartupMemoryLow=`.
This setting does not affect " `memory.min`" or " `memory.low`"
in the unit itself.
Using it to set a default child allocation is only useful on kernels older than 5.7,
which do not support the " `memory_recursiveprot`" cgroup2 mount option.

While `StartupMemoryLow=` applies to the startup and shutdown phases of the system,
`MemoryMin=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupMemoryLow=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

Added in version 240.

### StartupMemoryLow=

These settings control the `memory` controller in the unified hierarchy.

Specify the memory usage protection of the executed processes in this unit.
When reclaiming memory, the unit is treated as if it was using less memory resulting in memory
to be preferentially reclaimed from unprotected units.
Using `MemoryLow=` results in a weaker protection where memory may still
be reclaimed to avoid invoking the OOM killer in case there is no other reclaimable memory.

For a protection to be effective, it is generally required to set a corresponding
allocation on all ancestors, which is then distributed between children
(with the exception of the root slice).
Any `MemoryMin=` or `MemoryLow=` allocation that is not
explicitly distributed to specific children is used to create a shared protection for all children.
As this is a shared protection, the children will freely compete for the memory.

Takes a memory size in bytes. If the value is suffixed with K, M, G or T, the specified memory size is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes (with the base 1024), respectively. Alternatively, a
percentage value may be specified, which is taken relative to the installed physical memory on the
system. If assigned the special value " `infinity`", all available memory is protected, which may be
useful in order to always inherit all of the protection afforded by ancestors.
This controls the " `memory.min`" or " `memory.low`" control group attribute.
For details about this control group attribute, see [Memory Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files).

Units may have their children use a default " `memory.min`" or
" `memory.low`" value by specifying `DefaultMemoryMin=` or
`DefaultMemoryLow=`, which has the same semantics as
`MemoryMin=` and `MemoryLow=`, or `DefaultStartupMemoryLow=`
which has the same semantics as `StartupMemoryLow=`.
This setting does not affect " `memory.min`" or " `memory.low`"
in the unit itself.
Using it to set a default child allocation is only useful on kernels older than 5.7,
which do not support the " `memory_recursiveprot`" cgroup2 mount option.

While `StartupMemoryLow=` applies to the startup and shutdown phases of the system,
`MemoryMin=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupMemoryLow=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

Added in version 240.

### DefaultStartupMemoryLow=

These settings control the `memory` controller in the unified hierarchy.

Specify the memory usage protection of the executed processes in this unit.
When reclaiming memory, the unit is treated as if it was using less memory resulting in memory
to be preferentially reclaimed from unprotected units.
Using `MemoryLow=` results in a weaker protection where memory may still
be reclaimed to avoid invoking the OOM killer in case there is no other reclaimable memory.

For a protection to be effective, it is generally required to set a corresponding
allocation on all ancestors, which is then distributed between children
(with the exception of the root slice).
Any `MemoryMin=` or `MemoryLow=` allocation that is not
explicitly distributed to specific children is used to create a shared protection for all children.
As this is a shared protection, the children will freely compete for the memory.

Takes a memory size in bytes. If the value is suffixed with K, M, G or T, the specified memory size is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes (with the base 1024), respectively. Alternatively, a
percentage value may be specified, which is taken relative to the installed physical memory on the
system. If assigned the special value " `infinity`", all available memory is protected, which may be
useful in order to always inherit all of the protection afforded by ancestors.
This controls the " `memory.min`" or " `memory.low`" control group attribute.
For details about this control group attribute, see [Memory Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files).

Units may have their children use a default " `memory.min`" or
" `memory.low`" value by specifying `DefaultMemoryMin=` or
`DefaultMemoryLow=`, which has the same semantics as
`MemoryMin=` and `MemoryLow=`, or `DefaultStartupMemoryLow=`
which has the same semantics as `StartupMemoryLow=`.
This setting does not affect " `memory.min`" or " `memory.low`"
in the unit itself.
Using it to set a default child allocation is only useful on kernels older than 5.7,
which do not support the " `memory_recursiveprot`" cgroup2 mount option.

While `StartupMemoryLow=` applies to the startup and shutdown phases of the system,
`MemoryMin=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupMemoryLow=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

Added in version 240.

### MemoryHigh=

These settings control the `memory` controller in the unified hierarchy.

Specify the throttling limit on memory usage of the executed processes in this unit. Memory usage may go
above the limit if unavoidable, but the processes are heavily slowed down and memory is taken away
aggressively in such cases. This is the main mechanism to control memory usage of a unit.

Takes a memory size in bytes. If the value is suffixed with K, M, G or T, the specified memory size is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes (with the base 1024), respectively. Alternatively, a
percentage value may be specified, which is taken relative to the installed physical memory on the
system. If assigned the
special value " `infinity`", no memory throttling is applied. This controls the
" `memory.high`" control group attribute. For details about this control group attribute, see
[Memory Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files).
The effective configuration is reported as `EffectiveMemoryHigh=`
(see also `EffectiveMemoryMax=`).

While `StartupMemoryHigh=` applies to the startup and shutdown phases of the system,
`MemoryHigh=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupMemoryHigh=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

Added in version 231.

### StartupMemoryHigh=

These settings control the `memory` controller in the unified hierarchy.

Specify the throttling limit on memory usage of the executed processes in this unit. Memory usage may go
above the limit if unavoidable, but the processes are heavily slowed down and memory is taken away
aggressively in such cases. This is the main mechanism to control memory usage of a unit.

Takes a memory size in bytes. If the value is suffixed with K, M, G or T, the specified memory size is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes (with the base 1024), respectively. Alternatively, a
percentage value may be specified, which is taken relative to the installed physical memory on the
system. If assigned the
special value " `infinity`", no memory throttling is applied. This controls the
" `memory.high`" control group attribute. For details about this control group attribute, see
[Memory Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files).
The effective configuration is reported as `EffectiveMemoryHigh=`
(see also `EffectiveMemoryMax=`).

While `StartupMemoryHigh=` applies to the startup and shutdown phases of the system,
`MemoryHigh=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupMemoryHigh=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

Added in version 231.

### MemoryMax=

These settings control the `memory` controller in the unified hierarchy.

Specify the absolute limit on memory usage of the executed processes in this unit. If memory usage
cannot be contained under the limit, out-of-memory killer is invoked inside the unit. It is recommended to
use `MemoryHigh=` as the main control mechanism and use `MemoryMax=` as the
last line of defense.

Takes a memory size in bytes. If the value is suffixed with K, M, G or T, the specified memory size is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes (with the base 1024), respectively. Alternatively, a
percentage value may be specified, which is taken relative to the installed physical memory on the system. If
assigned the special value " `infinity`", no memory limit is applied. This controls the
" `memory.max`" control group attribute. For details about this control group attribute, see
[Memory Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files).
The effective configuration is reported as `EffectiveMemoryMax=` (the value is
the most stringent limit of the unit and parent slices and it is capped by physical memory).

While `StartupMemoryMax=` applies to the startup and shutdown phases of the system,
`MemoryMax=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupMemoryMax=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

Added in version 231.

### StartupMemoryMax=

These settings control the `memory` controller in the unified hierarchy.

Specify the absolute limit on memory usage of the executed processes in this unit. If memory usage
cannot be contained under the limit, out-of-memory killer is invoked inside the unit. It is recommended to
use `MemoryHigh=` as the main control mechanism and use `MemoryMax=` as the
last line of defense.

Takes a memory size in bytes. If the value is suffixed with K, M, G or T, the specified memory size is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes (with the base 1024), respectively. Alternatively, a
percentage value may be specified, which is taken relative to the installed physical memory on the system. If
assigned the special value " `infinity`", no memory limit is applied. This controls the
" `memory.max`" control group attribute. For details about this control group attribute, see
[Memory Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files).
The effective configuration is reported as `EffectiveMemoryMax=` (the value is
the most stringent limit of the unit and parent slices and it is capped by physical memory).

While `StartupMemoryMax=` applies to the startup and shutdown phases of the system,
`MemoryMax=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupMemoryMax=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

Added in version 231.

### MemorySwapMax=

These settings control the `memory` controller in the unified hierarchy.

Specify the absolute limit on swap usage of the executed processes in this unit.

Takes a swap size in bytes. If the value is suffixed with K, M, G or T, the specified swap size is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes (with the base 1024), respectively. Alternatively, a
percentage value may be specified, which is taken relative to the specified swap size on the system. If assigned the
special value " `infinity`", no swap limit is applied. These settings control the
" `memory.swap.max`" control group attribute. For details about this control group attribute,
see [Memory Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files).

While `StartupMemorySwapMax=` applies to the startup and shutdown phases of the system,
`MemorySwapMax=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupMemorySwapMax=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

Added in version 232.

### StartupMemorySwapMax=

These settings control the `memory` controller in the unified hierarchy.

Specify the absolute limit on swap usage of the executed processes in this unit.

Takes a swap size in bytes. If the value is suffixed with K, M, G or T, the specified swap size is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes (with the base 1024), respectively. Alternatively, a
percentage value may be specified, which is taken relative to the specified swap size on the system. If assigned the
special value " `infinity`", no swap limit is applied. These settings control the
" `memory.swap.max`" control group attribute. For details about this control group attribute,
see [Memory Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files).

While `StartupMemorySwapMax=` applies to the startup and shutdown phases of the system,
`MemorySwapMax=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupMemorySwapMax=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

Added in version 232.

### MemoryZSwapMax=

These settings control the `memory` controller in the unified hierarchy.

Specify the absolute limit on zswap usage of the processes in this unit. Zswap is a lightweight compressed
cache for swap pages. It takes pages that are in the process of being swapped out and attempts to compress them into a
dynamically allocated RAM-based memory pool. If the limit specified is hit, no entries from this unit will be
stored in the pool until existing entries are faulted back or written out to disk. See the kernel's
[Zswap](https://docs.kernel.org/admin-guide/mm/zswap.html) documentation for more details.

Takes a size in bytes. If the value is suffixed with K, M, G or T, the specified size is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes (with the base 1024), respectively. If assigned the
special value " `infinity`", no limit is applied. These settings control the
" `memory.zswap.max`" control group attribute. For details about this control group attribute,
see [Memory Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files).

While `StartupMemoryZSwapMax=` applies to the startup and shutdown phases of the system,
`MemoryZSwapMax=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupMemoryZSwapMax=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

Added in version 253.

### StartupMemoryZSwapMax=

These settings control the `memory` controller in the unified hierarchy.

Specify the absolute limit on zswap usage of the processes in this unit. Zswap is a lightweight compressed
cache for swap pages. It takes pages that are in the process of being swapped out and attempts to compress them into a
dynamically allocated RAM-based memory pool. If the limit specified is hit, no entries from this unit will be
stored in the pool until existing entries are faulted back or written out to disk. See the kernel's
[Zswap](https://docs.kernel.org/admin-guide/mm/zswap.html) documentation for more details.

Takes a size in bytes. If the value is suffixed with K, M, G or T, the specified size is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes (with the base 1024), respectively. If assigned the
special value " `infinity`", no limit is applied. These settings control the
" `memory.zswap.max`" control group attribute. For details about this control group attribute,
see [Memory Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#memory-interface-files).

While `StartupMemoryZSwapMax=` applies to the startup and shutdown phases of the system,
`MemoryZSwapMax=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupMemoryZSwapMax=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

Added in version 253.

### MemoryZSwapWriteback=

This setting controls the `memory` controller in the unified hierarchy.

Takes a boolean argument. When true, pages stored in the Zswap cache are permitted to be
written to the backing storage, false otherwise. Defaults to true. This allows disabling
writeback of swap pages for IO-intensive applications, while retaining the ability to store
compressed pages in Zswap. See the kernel's
[Zswap](https://docs.kernel.org/admin-guide/mm/zswap.html) documentation
for more details.

Added in version 256.

### AllowedMemoryNodes=

These settings control the `cpuset` controller in the unified hierarchy.

Restrict processes to be executed on specific memory NUMA nodes. Takes a list of memory NUMA nodes indices
or ranges separated by either whitespace or commas. Memory NUMA nodes ranges are specified by the lower and upper
NUMA nodes indices separated by a dash.

Setting `AllowedMemoryNodes=` or `StartupAllowedMemoryNodes=` does not
guarantee that all of the memory NUMA nodes will be used by the processes as it may be limited by parent units.
The effective configuration is reported as `EffectiveMemoryNodes=`.

While `StartupAllowedMemoryNodes=` applies to the startup and shutdown phases of the system,
`AllowedMemoryNodes=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupAllowedMemoryNodes=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

This setting is supported only with the unified control group hierarchy.

Added in version 244.

### StartupAllowedMemoryNodes=

These settings control the `cpuset` controller in the unified hierarchy.

Restrict processes to be executed on specific memory NUMA nodes. Takes a list of memory NUMA nodes indices
or ranges separated by either whitespace or commas. Memory NUMA nodes ranges are specified by the lower and upper
NUMA nodes indices separated by a dash.

Setting `AllowedMemoryNodes=` or `StartupAllowedMemoryNodes=` does not
guarantee that all of the memory NUMA nodes will be used by the processes as it may be limited by parent units.
The effective configuration is reported as `EffectiveMemoryNodes=`.

While `StartupAllowedMemoryNodes=` applies to the startup and shutdown phases of the system,
`AllowedMemoryNodes=` applies to normal runtime of the system, and if the former is not set also to
the startup and shutdown phases. Using `StartupAllowedMemoryNodes=` allows prioritizing specific services at
boot-up and shutdown differently than during normal runtime.

This setting is supported only with the unified control group hierarchy.

Added in version 244.

### TasksAccounting=

This setting controls the `pids` controller in the unified hierarchy.

Turn on task accounting for this unit. Takes a boolean argument. If enabled, the kernel will
keep track of the total number of tasks in the unit and its children. This number includes both
kernel threads and userspace processes, with each thread counted individually. Note that turning on
tasks accounting for one unit will also implicitly turn it on for all units contained in the same
slice and for all its parent slices and the units contained therein. The system default for this
setting may be controlled with `DefaultTasksAccounting=` in
[systemd-system.conf(5)](systemd-system.conf.html#).

Added in version 227.

### TasksMax=

This setting controls the `pids` controller in the unified hierarchy.

Specify the maximum number of tasks that may be created in the unit. This ensures that the
number of tasks accounted for the unit (see above) stays below a specific limit. This either takes
an absolute number of tasks or a percentage value that is taken relative to the configured maximum
number of tasks on the system. If assigned the special value " `infinity`", no tasks
limit is applied. This controls the " `pids.max`" control group attribute. For
details about this control group attribute, see the
[pids controller](https://docs.kernel.org/admin-guide/cgroup-v2.html#pid).
The effective configuration is reported as `EffectiveTasksMax=`.

The system default for this setting may be controlled with
`DefaultTasksMax=` in
[systemd-system.conf(5)](systemd-system.conf.html#).

Added in version 227.

### IOAccounting=

This setting controls the `io` controller in the unified hierarchy.

Turn on Block I/O accounting for this unit, if the unified control group hierarchy is used on the
system. Takes a boolean argument. Note that turning on block I/O accounting for one unit will also implicitly
turn it on for all units contained in the same slice and all for its parent slices and the units contained
therein. The system default for this setting may be controlled with `DefaultIOAccounting=`
in
[systemd-system.conf(5)](systemd-system.conf.html#).

Added in version 230.

### IOWeight=

These settings control the `io` controller in the unified hierarchy.

Set the default overall block I/O weight for the executed processes, if the unified control
group hierarchy is used on the system. Takes a single weight value (between 1 and 10000) to set the
default block I/O weight. This controls the " `io.weight`" control group attribute,
which defaults to 100. For details about this control group attribute, see [IO\
Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#io-interface-files). The available I/O bandwidth is split up among all units within one slice
relative to their block I/O weight. A higher weight means more I/O bandwidth, a lower weight means
less.

While `StartupIOWeight=` applies
to the startup and shutdown phases of the system,
`IOWeight=` applies to the later runtime of
the system, and if the former is not set also to the startup
and shutdown phases. This allows prioritizing specific services at boot-up
and shutdown differently than during runtime.

Added in version 230.

### StartupIOWeight=

These settings control the `io` controller in the unified hierarchy.

Set the default overall block I/O weight for the executed processes, if the unified control
group hierarchy is used on the system. Takes a single weight value (between 1 and 10000) to set the
default block I/O weight. This controls the " `io.weight`" control group attribute,
which defaults to 100. For details about this control group attribute, see [IO\
Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#io-interface-files). The available I/O bandwidth is split up among all units within one slice
relative to their block I/O weight. A higher weight means more I/O bandwidth, a lower weight means
less.

While `StartupIOWeight=` applies
to the startup and shutdown phases of the system,
`IOWeight=` applies to the later runtime of
the system, and if the former is not set also to the startup
and shutdown phases. This allows prioritizing specific services at boot-up
and shutdown differently than during runtime.

Added in version 230.

### IODeviceWeight=

This setting controls the `io` controller in the unified hierarchy.

Set the per-device overall block I/O weight for the executed processes, if the unified control group
hierarchy is used on the system. Takes a space-separated pair of a file path and a weight value to specify
the device specific weight value, between 1 and 10000. (Example: " `/dev/sda 1000`"). The file
path may be specified as path to a block device node or as any other file, in which case the backing block
device of the file system of the file is determined. This controls the " `io.weight`" control
group attribute, which defaults to 100. Use this option multiple times to set weights for multiple devices.
For details about this control group attribute, see [IO Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#io-interface-files).

The specified device node should reference a block device that has an I/O scheduler
associated, i.e. should not refer to partition or loopback block devices, but to the originating,
physical device. When a path to a regular file or directory is specified it is attempted to
discover the correct originating device backing the file system of the specified path. This works
correctly only for simpler cases, where the file system is directly placed on a partition or
physical block device, or where simple 1:1 encryption using dm-crypt/LUKS is used. This discovery
does not cover complex storage and in particular RAID and volume management storage devices.

Added in version 230.

### IOReadBandwidthMax=

These settings control the `io` controller in the unified hierarchy.

Set the per-device overall block I/O bandwidth maximum limit for the executed processes, if the unified
control group hierarchy is used on the system. This limit is not work-conserving and the executed processes
are not allowed to use more even if the device has idle capacity. Takes a space-separated pair of a file
path and a bandwidth value (in bytes per second) to specify the device specific bandwidth. The file path may
be a path to a block device node, or as any other file in which case the backing block device of the file
system of the file is used. If the bandwidth is suffixed with K, M, G, or T, the specified bandwidth is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes, respectively, to the base of 1000. (Example:
"/dev/disk/by-path/pci-0000:00:1f.2-scsi-0:0:0:0 5M"). This controls the " `io.max`" control
group attributes. Use this option multiple times to set bandwidth limits for multiple devices. For details
about this control group attribute, see [IO Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#io-interface-files).


Similar restrictions on block device discovery as for `IODeviceWeight=` apply, see above.

Added in version 230.

### IOWriteBandwidthMax=

These settings control the `io` controller in the unified hierarchy.

Set the per-device overall block I/O bandwidth maximum limit for the executed processes, if the unified
control group hierarchy is used on the system. This limit is not work-conserving and the executed processes
are not allowed to use more even if the device has idle capacity. Takes a space-separated pair of a file
path and a bandwidth value (in bytes per second) to specify the device specific bandwidth. The file path may
be a path to a block device node, or as any other file in which case the backing block device of the file
system of the file is used. If the bandwidth is suffixed with K, M, G, or T, the specified bandwidth is
parsed as Kilobytes, Megabytes, Gigabytes, or Terabytes, respectively, to the base of 1000. (Example:
"/dev/disk/by-path/pci-0000:00:1f.2-scsi-0:0:0:0 5M"). This controls the " `io.max`" control
group attributes. Use this option multiple times to set bandwidth limits for multiple devices. For details
about this control group attribute, see [IO Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#io-interface-files).


Similar restrictions on block device discovery as for `IODeviceWeight=` apply, see above.

Added in version 230.

### IOReadIOPSMax=

These settings control the `io` controller in the unified hierarchy.

Set the per-device overall block I/O IOs-Per-Second maximum limit for the executed processes, if the
unified control group hierarchy is used on the system. This limit is not work-conserving and the executed
processes are not allowed to use more even if the device has idle capacity. Takes a space-separated pair of
a file path and an IOPS value to specify the device specific IOPS. The file path may be a path to a block
device node, or as any other file in which case the backing block device of the file system of the file is
used. If the IOPS is suffixed with K, M, G, or T, the specified IOPS is parsed as KiloIOPS, MegaIOPS,
GigaIOPS, or TeraIOPS, respectively, to the base of 1000. (Example:
"/dev/disk/by-path/pci-0000:00:1f.2-scsi-0:0:0:0 1K"). This controls the " `io.max`" control
group attributes. Use this option multiple times to set IOPS limits for multiple devices. For details about
this control group attribute, see [IO Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#io-interface-files).


Similar restrictions on block device discovery as for `IODeviceWeight=` apply, see above.

Added in version 230.

### IOWriteIOPSMax=

These settings control the `io` controller in the unified hierarchy.

Set the per-device overall block I/O IOs-Per-Second maximum limit for the executed processes, if the
unified control group hierarchy is used on the system. This limit is not work-conserving and the executed
processes are not allowed to use more even if the device has idle capacity. Takes a space-separated pair of
a file path and an IOPS value to specify the device specific IOPS. The file path may be a path to a block
device node, or as any other file in which case the backing block device of the file system of the file is
used. If the IOPS is suffixed with K, M, G, or T, the specified IOPS is parsed as KiloIOPS, MegaIOPS,
GigaIOPS, or TeraIOPS, respectively, to the base of 1000. (Example:
"/dev/disk/by-path/pci-0000:00:1f.2-scsi-0:0:0:0 1K"). This controls the " `io.max`" control
group attributes. Use this option multiple times to set IOPS limits for multiple devices. For details about
this control group attribute, see [IO Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#io-interface-files).


Similar restrictions on block device discovery as for `IODeviceWeight=` apply, see above.

Added in version 230.

### IODeviceLatencyTargetSec=

This setting controls the `io` controller in the unified hierarchy.

Set the per-device average target I/O latency for the executed processes, if the unified control group
hierarchy is used on the system. Takes a file path and a timespan separated by a space to specify
the device specific latency target. (Example: "/dev/sda 25ms"). The file path may be specified
as path to a block device node or as any other file, in which case the backing block device of the file
system of the file is determined. This controls the " `io.latency`" control group
attribute. Use this option multiple times to set latency target for multiple devices. For details about this
control group attribute, see [IO Interface Files](https://docs.kernel.org/admin-guide/cgroup-v2.html#io-interface-files).

Implies " `IOAccounting=yes`".

These settings are supported only if the unified control group hierarchy is used.

Similar restrictions on block device discovery as for `IODeviceWeight=` apply, see above.

Added in version 240.

### IPAccounting=

Takes a boolean argument. If true, turns on IPv4 and IPv6 network traffic accounting for packets sent
or received by the unit. When this option is turned on, all IPv4 and IPv6 sockets created by any process of
the unit are accounted for.

When this option is used in socket units, it applies to all IPv4 and IPv6 sockets
associated with it (including both listening and connection sockets where this applies). Note that for
socket-activated services, this configuration setting and the accounting data of the service unit and the
socket unit are kept separate, and displayed separately. No propagation of the setting and the collected
statistics is done, in either direction. Moreover, any traffic sent or received on any of the socket unit's
sockets is accounted to the socket unit — and never to the service unit it might have activated, even if the
socket is used by it.

The system default for this setting may be controlled with `DefaultIPAccounting=` in
[systemd-system.conf(5)](systemd-system.conf.html#).

Note that this functionality is currently only available for system services, not for
per-user services.

Added in version 235.

### IPAddressAllow=

Turn on network traffic filtering for IP packets sent and received over
`AF_INET` and `AF_INET6` sockets. Both directives take a
space separated list of IPv4 or IPv6 addresses, each optionally suffixed with an address prefix
length in bits after a " `/`" character. If the suffix is omitted, the address is
considered a host address, i.e. the filter covers the whole address (32 bits for IPv4, 128 bits for
IPv6).

The access lists configured with this option are applied to all sockets created by processes
of this unit (or in the case of socket units, associated with it). The lists are implicitly
combined with any lists configured for any of the parent slice units this unit might be a member
of. By default, both access lists are empty. Both ingress and egress traffic is filtered by these
settings. In case of ingress traffic the source IP address is checked against these access lists,
in case of egress traffic the destination IP address is checked. The following rules are applied in
turn:

- Access is granted when the checked IP address matches an entry in the
`IPAddressAllow=` list.

- Otherwise, access is denied when the checked IP address matches an entry in the
`IPAddressDeny=` list.

- Otherwise, access is granted.


In order to implement an allow-listing IP firewall, it is recommended to use a
`IPAddressDeny=` `any` setting on an upper-level slice unit
(such as the root slice `-.slice` or the slice containing all system services
`system.slice` – see
[systemd.special(7)](systemd.special.html#)
for details on these slice units), plus individual per-service `IPAddressAllow=`
lines permitting network access to relevant services, and only them.

Note that for socket-activated services, the IP access list configured on the socket unit
applies to all sockets associated with it directly, but not to any sockets created by the
ultimately activated services for it. Conversely, the IP access list configured for the service is
not applied to any sockets passed into the service via socket activation. Thus, it is usually a
good idea to replicate the IP access lists on both the socket and the service unit. Nevertheless,
it may make sense to maintain one list more open and the other one more restricted, depending on
the use case.

If these settings are used multiple times in the same unit the specified lists are combined. If an
empty string is assigned to these settings the specific access list is reset and all previous settings undone.

In place of explicit IPv4 or IPv6 address and prefix length specifications a small set of symbolic
names may be used. The following names are defined:

**Table 1. Special address/network names**

Symbolic NameDefinitionMeaning`any`0.0.0.0/0 ::/0Any host`localhost`127.0.0.0/8 ::1/128All addresses on the local loopback`link-local`169.254.0.0/16 fe80::/64All link-local IP addresses`multicast`224.0.0.0/4 ff00::/8All IP multicasting addresses

Note that these settings might not be supported on some systems (for example if eBPF control group
support is not enabled in the underlying kernel or container manager). These settings will have no effect in
that case. If compatibility with such systems is desired it is hence recommended to not exclusively rely on
them for IP security.

This option cannot be bypassed by prefixing " `+`" to the executable path
in the service unit, as it applies to the whole control group.

Added in version 235.

### IPAddressDeny=

Turn on network traffic filtering for IP packets sent and received over
`AF_INET` and `AF_INET6` sockets. Both directives take a
space separated list of IPv4 or IPv6 addresses, each optionally suffixed with an address prefix
length in bits after a " `/`" character. If the suffix is omitted, the address is
considered a host address, i.e. the filter covers the whole address (32 bits for IPv4, 128 bits for
IPv6).

The access lists configured with this option are applied to all sockets created by processes
of this unit (or in the case of socket units, associated with it). The lists are implicitly
combined with any lists configured for any of the parent slice units this unit might be a member
of. By default, both access lists are empty. Both ingress and egress traffic is filtered by these
settings. In case of ingress traffic the source IP address is checked against these access lists,
in case of egress traffic the destination IP address is checked. The following rules are applied in
turn:

- Access is granted when the checked IP address matches an entry in the
`IPAddressAllow=` list.

- Otherwise, access is denied when the checked IP address matches an entry in the
`IPAddressDeny=` list.

- Otherwise, access is granted.


In order to implement an allow-listing IP firewall, it is recommended to use a
`IPAddressDeny=` `any` setting on an upper-level slice unit
(such as the root slice `-.slice` or the slice containing all system services
`system.slice` – see
[systemd.special(7)](systemd.special.html#)
for details on these slice units), plus individual per-service `IPAddressAllow=`
lines permitting network access to relevant services, and only them.

Note that for socket-activated services, the IP access list configured on the socket unit
applies to all sockets associated with it directly, but not to any sockets created by the
ultimately activated services for it. Conversely, the IP access list configured for the service is
not applied to any sockets passed into the service via socket activation. Thus, it is usually a
good idea to replicate the IP access lists on both the socket and the service unit. Nevertheless,
it may make sense to maintain one list more open and the other one more restricted, depending on
the use case.

If these settings are used multiple times in the same unit the specified lists are combined. If an
empty string is assigned to these settings the specific access list is reset and all previous settings undone.

In place of explicit IPv4 or IPv6 address and prefix length specifications a small set of symbolic
names may be used. The following names are defined:

**Table 1. Special address/network names**

Symbolic NameDefinitionMeaning`any`0.0.0.0/0 ::/0Any host`localhost`127.0.0.0/8 ::1/128All addresses on the local loopback`link-local`169.254.0.0/16 fe80::/64All link-local IP addresses`multicast`224.0.0.0/4 ff00::/8All IP multicasting addresses

Note that these settings might not be supported on some systems (for example if eBPF control group
support is not enabled in the underlying kernel or container manager). These settings will have no effect in
that case. If compatibility with such systems is desired it is hence recommended to not exclusively rely on
them for IP security.

This option cannot be bypassed by prefixing " `+`" to the executable path
in the service unit, as it applies to the whole control group.

Added in version 235.

### SocketBindAllow=

Configures restrictions on the ability of unit processes to invoke [bind(2)](https://man7.org/linux/man-pages/man2/bind.2.html) on a
socket. Both allow and deny rules to be defined that restrict which addresses a socket may be bound
to.

_`bind-rule`_ describes socket properties such as _`address-family`_,
_`transport-protocol`_ and _`ip-ports`_.

_`bind-rule`_ :=
{ \[ _`address-family`_ `:`\]\[ _`transport-protocol`_ `:`\]\[ _`ip-ports`_\] \| `any` }

_`address-family`_ := { `ipv4` \| `ipv6` }

_`transport-protocol`_ := { `tcp` \| `udp` }

_`ip-ports`_ := { _`ip-port`_ \| _`ip-port-range`_ }

An optional _`address-family`_ expects `ipv4` or `ipv6` values.
If not specified, a rule will be matched for both IPv4 and IPv6 addresses and applied depending on other socket fields, e.g. _`transport-protocol`_,
_`ip-port`_.

An optional _`transport-protocol`_ expects `tcp` or `udp` transport protocol names.
If not specified, a rule will be matched for any transport protocol.

An optional _`ip-port`_ value must lie within 1…65535 interval inclusively, i.e.
dynamic port `0` is not allowed. A range of sequential ports is described by
_`ip-port-range`_ := _`ip-port-low`_ `-` _`ip-port-high`_,
where _`ip-port-low`_ is smaller than or equal to _`ip-port-high`_
and both are within 1…65535 inclusively.

A special value `any` can be used to apply a rule to any address family, transport protocol and any port with a positive value.

To allow multiple rules assign `SocketBindAllow=` or `SocketBindDeny=` multiple times.
To clear the existing assignments pass an empty `SocketBindAllow=` or `SocketBindDeny=`
assignment.

For each of `SocketBindAllow=` and `SocketBindDeny=`, maximum allowed number of assignments is
`128`.

- Binding to a socket is allowed when a socket address matches an entry in the
`SocketBindAllow=` list.

- Otherwise, binding is denied when the socket address matches an entry in the
`SocketBindDeny=` list.

- Otherwise, binding is allowed.


The feature is implemented with `cgroup/bind4` and `cgroup/bind6` cgroup-bpf hooks.

Note that these settings apply to any [bind(2)](https://man7.org/linux/man-pages/man2/bind.2.html)
system call invocation by the unit processes, regardless in which network namespace they are
placed. Or in other words: changing the network namespace is not a suitable mechanism for escaping
these restrictions on `bind()`.

Examples:

```
…
# Allow binding IPv6 socket addresses with a port greater than or equal to 10000.
[Service]
SocketBindAllow=ipv6:10000-65535
SocketBindDeny=any
…
# Allow binding IPv4 and IPv6 socket addresses with 1234 and 4321 ports.
[Service]
SocketBindAllow=1234
SocketBindAllow=4321
SocketBindDeny=any
…
# Deny binding IPv6 socket addresses.
[Service]
SocketBindDeny=ipv6
…
# Deny binding IPv4 and IPv6 socket addresses.
[Service]
SocketBindDeny=any
…
# Allow binding only over TCP
[Service]
SocketBindAllow=tcp
SocketBindDeny=any
…
# Allow binding only over IPv6/TCP
[Service]
SocketBindAllow=ipv6:tcp
SocketBindDeny=any
…
# Allow binding ports within 10000-65535 range over IPv4/UDP.
[Service]
SocketBindAllow=ipv4:udp:10000-65535
SocketBindDeny=any
…
```

This option cannot be bypassed by prefixing " `+`" to the executable path
in the service unit, as it applies to the whole control group.

Added in version 249.

### SocketBindDeny=

Configures restrictions on the ability of unit processes to invoke [bind(2)](https://man7.org/linux/man-pages/man2/bind.2.html) on a
socket. Both allow and deny rules to be defined that restrict which addresses a socket may be bound
to.

_`bind-rule`_ describes socket properties such as _`address-family`_,
_`transport-protocol`_ and _`ip-ports`_.

_`bind-rule`_ :=
{ \[ _`address-family`_ `:`\]\[ _`transport-protocol`_ `:`\]\[ _`ip-ports`_\] \| `any` }

_`address-family`_ := { `ipv4` \| `ipv6` }

_`transport-protocol`_ := { `tcp` \| `udp` }

_`ip-ports`_ := { _`ip-port`_ \| _`ip-port-range`_ }

An optional _`address-family`_ expects `ipv4` or `ipv6` values.
If not specified, a rule will be matched for both IPv4 and IPv6 addresses and applied depending on other socket fields, e.g. _`transport-protocol`_,
_`ip-port`_.

An optional _`transport-protocol`_ expects `tcp` or `udp` transport protocol names.
If not specified, a rule will be matched for any transport protocol.

An optional _`ip-port`_ value must lie within 1…65535 interval inclusively, i.e.
dynamic port `0` is not allowed. A range of sequential ports is described by
_`ip-port-range`_ := _`ip-port-low`_ `-` _`ip-port-high`_,
where _`ip-port-low`_ is smaller than or equal to _`ip-port-high`_
and both are within 1…65535 inclusively.

A special value `any` can be used to apply a rule to any address family, transport protocol and any port with a positive value.

To allow multiple rules assign `SocketBindAllow=` or `SocketBindDeny=` multiple times.
To clear the existing assignments pass an empty `SocketBindAllow=` or `SocketBindDeny=`
assignment.

For each of `SocketBindAllow=` and `SocketBindDeny=`, maximum allowed number of assignments is
`128`.

- Binding to a socket is allowed when a socket address matches an entry in the
`SocketBindAllow=` list.

- Otherwise, binding is denied when the socket address matches an entry in the
`SocketBindDeny=` list.

- Otherwise, binding is allowed.


The feature is implemented with `cgroup/bind4` and `cgroup/bind6` cgroup-bpf hooks.

Note that these settings apply to any [bind(2)](https://man7.org/linux/man-pages/man2/bind.2.html)
system call invocation by the unit processes, regardless in which network namespace they are
placed. Or in other words: changing the network namespace is not a suitable mechanism for escaping
these restrictions on `bind()`.

Examples:

```
…
# Allow binding IPv6 socket addresses with a port greater than or equal to 10000.
[Service]
SocketBindAllow=ipv6:10000-65535
SocketBindDeny=any
…
# Allow binding IPv4 and IPv6 socket addresses with 1234 and 4321 ports.
[Service]
SocketBindAllow=1234
SocketBindAllow=4321
SocketBindDeny=any
…
# Deny binding IPv6 socket addresses.
[Service]
SocketBindDeny=ipv6
…
# Deny binding IPv4 and IPv6 socket addresses.
[Service]
SocketBindDeny=any
…
# Allow binding only over TCP
[Service]
SocketBindAllow=tcp
SocketBindDeny=any
…
# Allow binding only over IPv6/TCP
[Service]
SocketBindAllow=ipv6:tcp
SocketBindDeny=any
…
# Allow binding ports within 10000-65535 range over IPv4/UDP.
[Service]
SocketBindAllow=ipv4:udp:10000-65535
SocketBindDeny=any
…
```

This option cannot be bypassed by prefixing " `+`" to the executable path
in the service unit, as it applies to the whole control group.

Added in version 249.

### RestrictNetworkInterfaces=

Takes a list of space-separated network interface names. This option restricts the network
interfaces that processes of this unit can use. By default, processes can only use the network interfaces
listed (allow-list). If the first character of the rule is " `~`", the effect is inverted:
the processes can only use network interfaces not listed (deny-list).


This option can appear multiple times, in which case the network interface names are merged. If the
empty string is assigned the set is reset, all prior assignments will have not effect.


If you specify both types of this option (i.e. allow-listing and deny-listing), the first encountered
will take precedence and will dictate the default action (allow vs deny). Then the next occurrences of this
option will add or delete the listed network interface names from the set, depending of its type and the
default action.


The loopback interface ("lo") is not treated in any special way, you have to configure it explicitly
in the unit file.


Example 1: allow-list


```
RestrictNetworkInterfaces=eth1
RestrictNetworkInterfaces=eth2
```

Programs in the unit will be only able to use the eth1 and eth2 network
interfaces.


Example 2: deny-list


```
RestrictNetworkInterfaces=~eth1 eth2
```

Programs in the unit will be able to use any network interface but eth1 and eth2.


Example 3: mixed


```
RestrictNetworkInterfaces=eth1 eth2
RestrictNetworkInterfaces=~eth1
```

Programs in the unit will be only able to use the eth2 network interface.


This option cannot be bypassed by prefixing " `+`" to the executable path
in the service unit, as it applies to the whole control group.

Added in version 250.

### NFTSet=

This setting provides a method for integrating dynamic cgroup, user and group IDs into
firewall rules with [NFT](https://netfilter.org/projects/nftables/index.html)
sets. The benefit of using this setting is to be able to use the IDs as selectors in firewall rules
easily and this in turn allows more fine grained filtering. NFT rules for cgroup matching use
numeric cgroup IDs, which change every time a service is restarted, making them hard to use in
systemd environment otherwise. Dynamic and random IDs used by `DynamicUser=` can
be also integrated with this setting.

This option expects a whitespace separated list of NFT set definitions. Each definition
consists of a colon-separated tuple of source type (one of " `cgroup`",
" `user`" or " `group`"), NFT address family (one of
" `arp`", " `bridge`", " `inet`", " `ip`",
" `ip6`", or " `netdev`"), table name and set name. The names of tables
and sets must conform to lexicographic restrictions of NFT table names. The type of the element used in
the NFT filter must match the type implied by the directive (" `cgroup`",
" `user`" or " `group`") as shown in the table below. When a control
group or a unit is realized, the corresponding ID will be appended to the NFT sets and it will be
be removed when the control group or unit is removed. **systemd** only inserts
elements to (or removes from) the sets, so the related NFT rules, tables and sets must be prepared
elsewhere in advance. Failures to manage the sets will be ignored.

**Table 2. Defined `source type` values**

Source typeDescriptionCorresponding NFT type name" `cgroup`"control group ID" `cgroupsv2`"" `user`"user ID" `meta skuid`"" `group`"group ID" `meta skgid`"

If the firewall rules are reinstalled so that the contents of NFT sets are destroyed, command
**systemctl daemon-reload** can be used to refill the sets.

Example:


```
[Unit]
NFTSet=cgroup:inet:filter:my_service user:inet:filter:serviceuser

```

Corresponding NFT rules:


```
table inet filter {
        set my_service {
                type cgroupsv2
        }
        set serviceuser {
                typeof meta skuid
        }
        chain x {
                socket cgroupv2 level 2 @my_service accept
                drop
        }
        chain y {
                meta skuid @serviceuser accept
                drop
        }
}
```

This option is only available for system services and is not supported for services
running in per-user instances of the service manager.

Added in version 255.

### IPIngressFilterPath=

Add custom network traffic filters implemented as BPF programs, applying to all IP packets
sent and received over `AF_INET` and `AF_INET6` sockets.
Takes an absolute path to a pinned BPF program in the BPF virtual filesystem ( `/sys/fs/bpf/`).


The filters configured with this option are applied to all sockets created by processes
of this unit (or in the case of socket units, associated with it). The filters are loaded in addition
to filters any of the parent slice units this unit might be a member of as well as any
`IPAddressAllow=` and `IPAddressDeny=` filters in any of these units.
By default, there are no filters specified.

If these settings are used multiple times in the same unit all the specified programs are attached. If an
empty string is assigned to these settings the program list is reset and all previous specified programs ignored.

If the path _`BPF_FS_PROGRAM_PATH`_ in `IPIngressFilterPath=` assignment
is already being handled by `BPFProgram=` ingress hook, e.g.
`BPFProgram=` `ingress`: _`BPF_FS_PROGRAM_PATH`_,
the assignment will be still considered valid and the program will be attached to a cgroup. Same for
`IPEgressFilterPath=` path and `egress` hook.

Note that for socket-activated services, the IP filter programs configured on the socket unit apply to
all sockets associated with it directly, but not to any sockets created by the ultimately activated services
for it. Conversely, the IP filter programs configured for the service are not applied to any sockets passed into
the service via socket activation. Thus, it is usually a good idea, to replicate the IP filter programs on both
the socket and the service unit, however it often makes sense to maintain one configuration more open and the other
one more restricted, depending on the use case.

Note that these settings might not be supported on some systems (for example if eBPF control group
support is not enabled in the underlying kernel or container manager). These settings will have no effect in
that case. If compatibility with such systems is desired it is hence recommended to attach your filter manually
(requires `Delegate=` `yes`) instead of using this setting.

Added in version 243.

### IPEgressFilterPath=

Add custom network traffic filters implemented as BPF programs, applying to all IP packets
sent and received over `AF_INET` and `AF_INET6` sockets.
Takes an absolute path to a pinned BPF program in the BPF virtual filesystem ( `/sys/fs/bpf/`).


The filters configured with this option are applied to all sockets created by processes
of this unit (or in the case of socket units, associated with it). The filters are loaded in addition
to filters any of the parent slice units this unit might be a member of as well as any
`IPAddressAllow=` and `IPAddressDeny=` filters in any of these units.
By default, there are no filters specified.

If these settings are used multiple times in the same unit all the specified programs are attached. If an
empty string is assigned to these settings the program list is reset and all previous specified programs ignored.

If the path _`BPF_FS_PROGRAM_PATH`_ in `IPIngressFilterPath=` assignment
is already being handled by `BPFProgram=` ingress hook, e.g.
`BPFProgram=` `ingress`: _`BPF_FS_PROGRAM_PATH`_,
the assignment will be still considered valid and the program will be attached to a cgroup. Same for
`IPEgressFilterPath=` path and `egress` hook.

Note that for socket-activated services, the IP filter programs configured on the socket unit apply to
all sockets associated with it directly, but not to any sockets created by the ultimately activated services
for it. Conversely, the IP filter programs configured for the service are not applied to any sockets passed into
the service via socket activation. Thus, it is usually a good idea, to replicate the IP filter programs on both
the socket and the service unit, however it often makes sense to maintain one configuration more open and the other
one more restricted, depending on the use case.

Note that these settings might not be supported on some systems (for example if eBPF control group
support is not enabled in the underlying kernel or container manager). These settings will have no effect in
that case. If compatibility with such systems is desired it is hence recommended to attach your filter manually
(requires `Delegate=` `yes`) instead of using this setting.

Added in version 243.

### BPFProgram=

`BPFProgram=` allows attaching custom BPF programs to the cgroup of a
unit. (This generalizes the functionality exposed via `IPEgressFilterPath=` and
`IPIngressFilterPath=` for other hooks.) Cgroup-bpf hooks in the form of BPF
programs loaded to the BPF filesystem are attached with cgroup-bpf attach flags determined by the
unit. For details about attachment types and flags see [`bpf.h`](https://git.kernel.org/pub/scm/linux/kernel/git/torvalds/linux.git/plain/include/uapi/linux/bpf.h). Also
refer to the general [BPF documentation](https://docs.kernel.org/bpf/).

The specification of BPF program consists of a pair of BPF program type and program path in
the file system, with " `:`" as the separator:
_`type`_: _`program-path`_.

The BPF program type is equivalent to the BPF attach type used in
[bpftool(8)](https://www.mankier.com/8/bpftool)
It may be one of
`egress`,
`ingress`,
`sock_create`,
`sock_ops`,
`device`,
`bind4`,
`bind6`,
`connect4`,
`connect6`,
`post_bind4`,
`post_bind6`,
`sendmsg4`,
`sendmsg6`,
`sysctl`,
`recvmsg4`,
`recvmsg6`,
`getsockopt`,
or `setsockopt`.


The specified program path must be an absolute path referencing a BPF program inode in the
bpffs file system (which generally means it must begin with `/sys/fs/bpf/`). If
a specified program does not exist (i.e. has not been uploaded to the BPF subsystem of the kernel
yet), it will not be installed but unit activation will continue (a warning will be printed to the
logs).

Setting `BPFProgram=` to an empty value makes previous assignments
ineffective.

Multiple assignments of the same program type/path pair have the same effect as a single
assignment: the program will be attached just once.

If BPF `egress` pinned to _`program-path`_ path is already being
handled by `IPEgressFilterPath=`, `BPFProgram=`
assignment will be considered valid and `BPFProgram=` will be attached to a cgroup.
Similarly for `ingress` hook and `IPIngressFilterPath=` assignment.

BPF programs passed with `BPFProgram=` are attached to the cgroup of a unit
with BPF attach flag `multi`, that allows further attachments of the same
_`type`_ within cgroup hierarchy topped by the unit cgroup.

Examples:

```
BPFProgram=egress:/sys/fs/bpf/egress-hook
BPFProgram=bind6:/sys/fs/bpf/sock-addr-hook

```

Added in version 249.

### DeviceAllow=

Control access to specific device nodes by the executed processes. Takes two space-separated
strings: a device node specifier followed by a combination of `r`,
`w`, `m` to control _r_eading,
_w_riting, or creation of the specific device nodes by the unit
(_m_knod), respectively. This functionality is implemented using eBPF
filtering.

When access to _all_ physical devices should be disallowed,
`PrivateDevices=` may be used instead. See
[systemd.exec(5)](systemd.exec.html#).


The device node specifier is either a path to a device node in the file system, starting with
`/dev/`, or a string starting with either " `char-`" or
" `block-`" followed by a device group name, as listed in
`/proc/devices`. The latter is useful to allow-list all current and future
devices belonging to a specific device group at once. The device group is matched according to
filename globbing rules, you may hence use the " `*`" and " `?`"
wildcards. (Note that such globbing wildcards are not available for device node path
specifications!) In order to match device nodes by numeric major/minor, use device node paths in
the `/dev/char/` and `/dev/block/` directories. However,
matching devices by major/minor is generally not recommended as assignments are neither stable nor
portable between systems or different kernel versions.

Examples: `/dev/sda5` is a path to a device node, referring to an ATA or
SCSI block device. " `char-pts`" and " `char-alsa`" are specifiers for
all pseudo TTYs and all ALSA sound devices, respectively. " `char-cpu/*`" is a
specifier matching all CPU related device groups.

Note that allow lists defined this way should only reference device groups which are
resolvable at the time the unit is started. Any device groups not resolvable then are not added to
the device allow list. In order to work around this limitation, consider extending service units
with a pair of **After=modprobe@xyz.service** and
**Wants=modprobe@xyz.service** lines that load the necessary kernel module
implementing the device group if missing.
Example:

```
…
[Unit]
Wants=modprobe@loop.service
After=modprobe@loop.service

[Service]
DeviceAllow=block-loop
DeviceAllow=/dev/loop-control
…
```

This option cannot be bypassed by prefixing " `+`" to the executable path
in the service unit, as it applies to the whole control group.

Added in version 208.

### DevicePolicy=

Control the policy for allowing device access:


`strict` [¶](#strict "Permalink to this term")

means to only allow types of access that are
explicitly specified.

Added in version 208.

`closed` [¶](#closed "Permalink to this term")

in addition, allows access to standard pseudo
devices including
`/dev/null`,
`/dev/zero`,
`/dev/full`,
`/dev/random`, and
`/dev/urandom`.


Added in version 208.

`auto` [¶](#auto "Permalink to this term")

in addition, allows access to all devices if no
explicit `DeviceAllow=` is present.
This is the default.


Added in version 208.

This option cannot be bypassed by prefixing " `+`" to the executable path
in the service unit, as it applies to the whole control group.

Added in version 208.

### Slice=

The name of the slice unit to place the unit
in. Defaults to `system.slice` for all
non-instantiated units of all unit types (except for slice
units themselves see below). Instance units are by default
placed in a subslice of `system.slice`
that is named after the template name.

This option may be used to arrange systemd units in a
hierarchy of slices each of which might have resource
settings applied.

For units of type slice, the only accepted value for
this setting is the parent slice. Since the name of a slice
unit implies the parent slice, it is hence redundant to ever
set this parameter directly for slice units.

Special care should be taken when relying on the default slice assignment in templated service units
that have `DefaultDependencies=no` set, see
[systemd.service(5)](systemd.service.html#), section
"Default Dependencies" for details.

Added in version 208.

### Delegate=

Turns on delegation of further resource control partitioning to processes of the unit. Units
where this is enabled may create and manage their own private subhierarchy of control groups below
the control group of the unit itself. For unprivileged services (i.e. those using the
`User=` setting) the unit's control group will be made accessible to the relevant
user.

When enabled the service manager will refrain from manipulating control groups or moving
processes below the unit's control group, so that a clear concept of ownership is established: the
control group tree at the level of the unit's control group and above (i.e. towards the root
control group) is owned and managed by the service manager of the host, while the control group
tree below the unit's control group is owned and managed by the unit itself.

Takes either a boolean argument or a (possibly empty) list of control group controller names.
If true, delegation is turned on, and all supported controllers are enabled for the unit, making
them available to the unit's processes for management. If false, delegation is turned off entirely
(and no additional controllers are enabled). If set to a list of controllers, delegation is turned
on, and the specified controllers are enabled for the unit. Assigning the empty string will enable
delegation, but reset the list of controllers, and all assignments prior to this will have no
effect. Note that additional controllers other than the ones specified might be made available as
well, depending on configuration of the containing slice unit or other units contained in it.
Defaults to false.

Note that controller delegation to less privileged code is only safe on the unified control
group hierarchy. Accordingly, access to the specified controllers will not be granted to
unprivileged services on the legacy hierarchy, even when requested.

The following controller names may be specified: `cpu`,
`cpuset`, `io`, `memory`, `pids`,
`bpf-firewall`, `bpf-devices`, `bpf-foreign`,
`bpf-socket-bind`, and `bpf-restrict-network-interfaces`.

Not all of these controllers are available on all kernels however, and some are specific to
the unified hierarchy while others are specific to the legacy hierarchy. Also note that the kernel
might support further controllers, which are not covered here yet, as delegation is either not
supported at all for them or not defined cleanly.

Note that because of the hierarchical nature of cgroup hierarchy, any controllers that are
delegated will be enabled for the parent and sibling units of the unit with delegation.

For further details on the delegation model consult [Control Group APIs and Delegation](https://systemd.io/CGROUP_DELEGATION).

Added in version 218.

### DelegateSubgroup=

Place unit processes in the specified subgroup of the unit's control group. Takes a valid
control group name (not a path!) as parameter, or an empty string to turn this feature
off. Defaults to off. The control group name must be usable as filename and avoid conflicts with
the kernel's control group attribute files (i.e. `cgroup.procs` is not an
acceptable name, since the kernel exposes a native control group attribute file by that name). This
option has no effect unless control group delegation is turned on via `Delegate=`,
see above. Note that this setting only applies to "main" processes of a unit, i.e. for services to
`ExecStart=`, but not for `ExecReload=` and similar. If
delegation is enabled, the latter are always placed inside a subgroup named
`.control`. The specified subgroup is automatically created (and potentially
ownership is passed to the unit's configured user/group) when a process is started in it.

This option is useful to avoid manually moving the invoked process into a subgroup after it
has been started. Since no processes should live in inner nodes of the control group tree it is
almost always necessary to run the main ("supervising") process of a unit that has delegation
turned on in a subgroup.

Added in version 254.

### DisableControllers=

Disables controllers from being enabled for a unit's children. If a controller listed is
already in use in its subtree, the controller will be removed from the subtree. This can be used to
avoid configuration in child units from being able to implicitly or explicitly enable a controller.
Defaults to empty.

Multiple controllers may be specified, separated by spaces. You may also pass
`DisableControllers=` multiple times, in which case each new instance adds another controller
to disable. Passing `DisableControllers=` by itself with no controller name present resets
the disabled controller list.

It may not be possible to disable a controller after units have been started, if the unit or
any child of the unit in question delegates controllers to its children, as any delegated subtree
of the cgroup hierarchy is unmanaged by systemd.

The following controller names may be specified: `cpu`,
`cpuset`, `io`, `memory`, `pids`,
`bpf-firewall`, `bpf-devices`, `bpf-foreign`,
`bpf-socket-bind`, and `bpf-restrict-network-interfaces`.

Added in version 240.

### ManagedOOMSwap=

Specifies how
[systemd-oomd.service(8)](systemd-oomd.service.html#)
will act on this unit's cgroups. Defaults to `auto`.

When set to `kill`, the unit becomes a candidate for monitoring by
**systemd-oomd**. If the cgroup passes the limits set by
[oomd.conf(5)](oomd.conf.html#) or
the unit configuration, **systemd-oomd** will select a descendant cgroup and send
`SIGKILL` to all of the processes under it. You can find more details on
candidates and kill behavior at
[systemd-oomd.service(8)](systemd-oomd.service.html#)
and
[oomd.conf(5)](oomd.conf.html#).

Setting either of these properties to `kill` will also result in
`After=` and `Wants=` dependencies on
`systemd-oomd.service` unless `DefaultDependencies=no`.

When set to `auto`, **systemd-oomd** will not actively use this
cgroup's data for monitoring and detection. However, if an ancestor cgroup has one of these
properties set to `kill`, a unit with `auto` can still be a candidate
for **systemd-oomd** to terminate.

Added in version 247.

### ManagedOOMMemoryPressure=

Specifies how
[systemd-oomd.service(8)](systemd-oomd.service.html#)
will act on this unit's cgroups. Defaults to `auto`.

When set to `kill`, the unit becomes a candidate for monitoring by
**systemd-oomd**. If the cgroup passes the limits set by
[oomd.conf(5)](oomd.conf.html#) or
the unit configuration, **systemd-oomd** will select a descendant cgroup and send
`SIGKILL` to all of the processes under it. You can find more details on
candidates and kill behavior at
[systemd-oomd.service(8)](systemd-oomd.service.html#)
and
[oomd.conf(5)](oomd.conf.html#).

Setting either of these properties to `kill` will also result in
`After=` and `Wants=` dependencies on
`systemd-oomd.service` unless `DefaultDependencies=no`.

When set to `auto`, **systemd-oomd** will not actively use this
cgroup's data for monitoring and detection. However, if an ancestor cgroup has one of these
properties set to `kill`, a unit with `auto` can still be a candidate
for **systemd-oomd** to terminate.

Added in version 247.

### ManagedOOMMemoryPressureLimit=

Overrides the default memory pressure limit set by
[oomd.conf(5)](oomd.conf.html#) for
the cgroup of this unit. Takes a percentage value between 0% and 100%, inclusive. Defaults to 0%,
which means to use the default set by
[oomd.conf(5)](oomd.conf.html#).
This property is ignored unless `ManagedOOMMemoryPressure=` `kill`.


Added in version 247.

### ManagedOOMMemoryPressureDurationSec=

Overrides the default memory pressure duration set by
[oomd.conf(5)](oomd.conf.html#) for
the cgroup of this unit. The specified value supports a time unit such as " `ms`" or
" `μs`", see
[systemd.time(7)](systemd.time.html#)
for details on the permitted syntax. Must be set to either empty or a value of at least 1s. Defaults
to empty, which means to use the default set by
[oomd.conf(5)](oomd.conf.html#).
This property is ignored unless `ManagedOOMMemoryPressure=` `kill`.


Added in version 257.

### ManagedOOMPreference=

Allows deprioritizing or omitting this unit's cgroup as a candidate when
**systemd-oomd** needs to act. Requires support for extended attributes (see
[xattr(7)](https://man7.org/linux/man-pages/man7/xattr.7.html))
in order to use `avoid` or `omit`.

When calculating candidates to relieve swap usage, **systemd-oomd** will
only respect these extended attributes if the unit's cgroup is owned by root.

When calculating candidates to relieve memory pressure, **systemd-oomd**
will only respect these extended attributes if the unit's cgroup is owned by root, or if the
unit's cgroup owner, and the owner of the monitored ancestor cgroup are the same. For example,
if **systemd-oomd** is calculating candidates for `-.slice`,
then extended attributes set on descendants of `/user.slice/user-1000.slice/user@1000.service/`
will be ignored because the descendants are owned by UID 1000, and `-.slice`
is owned by UID 0. But, if calculating candidates for
`/user.slice/user-1000.slice/user@1000.service/`, then extended attributes set
on the descendants would be respected.

If this property is set to `avoid`, the service manager will convey this to
**systemd-oomd**, which will only select this cgroup if there are no other viable
candidates.

If this property is set to `omit`, the service manager will convey this to
**systemd-oomd**, which will ignore this cgroup as a candidate and will not perform
any actions on it.

It is recommended to use `avoid` and `omit` sparingly, as it
can adversely affect **systemd-oomd**'s kill behavior. Also note that these extended
attributes are not applied recursively to cgroups under this unit's cgroup.

Defaults to `none` which means **systemd-oomd** will rank this
unit's cgroup as defined in
[systemd-oomd.service(8)](systemd-oomd.service.html#)
and [oomd.conf(5)](oomd.conf.html#).


Added in version 248.

### MemoryPressureWatch=

Controls memory pressure monitoring for invoked processes. Takes a boolean or one of
" `auto`" and " `skip`". If " `no`", tells the service not
to watch for memory pressure events, by setting the `$MEMORY_PRESSURE_WATCH`
environment variable to the literal string `/dev/null`. If " `yes`",
tells the service to watch for memory pressure events. This enables memory accounting for the
service, and ensures the `memory.pressure` cgroup attribute file is accessible for
reading and writing by the service's user. It then sets the `$MEMORY_PRESSURE_WATCH`
environment variable for processes invoked by the unit to the file system path to this file. The
threshold information configured with `MemoryPressureThresholdSec=` is encoded in
the `$MEMORY_PRESSURE_WRITE` environment variable. If the " `auto`"
value is set the protocol is enabled if memory accounting is anyway enabled for the unit, and
disabled otherwise. If set to " `skip`" the logic is neither enabled, nor disabled and
the two environment variables are not set.

Note that services are free to use the two environment variables, but it is unproblematic if
they ignore them. Memory pressure handling must be implemented individually in each service, and
usually means different things for different software. For further details on memory pressure
handling see [Memory Pressure Handling in\
systemd](https://systemd.io/MEMORY_PRESSURE).

Services implemented using
[sd-event(3)](sd-event.html#) may use
[sd\_event\_add\_memory\_pressure(3)](sd_event_add_memory_pressure.html#)
to watch for and handle memory pressure events.

If not explicit set, defaults to the `DefaultMemoryPressureWatch=` setting in
[systemd-system.conf(5)](systemd-system.conf.html#).

Added in version 254.

### MemoryPressureThresholdSec=

Sets the memory pressure threshold time for memory pressure monitor as configured via
`MemoryPressureWatch=`. Specifies the maximum allocation latency before a memory
pressure event is signalled to the service, per 2s window. If not specified, defaults to the
`DefaultMemoryPressureThresholdSec=` setting in
[systemd-system.conf(5)](systemd-system.conf.html#)
(which in turn defaults to 200ms). The specified value expects a time unit such as
" `ms`" or " `μs`", see
[systemd.time(7)](systemd.time.html#) for
details on the permitted syntax.

Added in version 254.

### CoredumpReceive=

Takes a boolean argument. This setting is used to enable coredump forwarding for containers
that belong to this unit's cgroup. Units with `CoredumpReceive=yes` must also be configured
with `Delegate=yes`. Defaults to false.

When **systemd-coredump** is handling a coredump for a process from a container,
if the container's leader process is a descendant of a cgroup with `CoredumpReceive=yes`
and `Delegate=yes`, then **systemd-coredump** will attempt to forward
the coredump to **systemd-coredump** within the container. See also
[systemd-coredump(8)](systemd-coredump.html#).

Added in version 255.

