# [Unit] Section

A unit file is a plain text ini-style file that encodes information about a service, a
socket, a device, a mount point, an automount point, a swap file or partition, a start-up
target, a watched file system path, a timer controlled and supervised by
[systemd(1)](systemd.html#), a
resource management slice or a group of externally created processes. See
[systemd.syntax(7)](systemd.syntax.html#)
for a general description of the syntax.

This man page lists the common configuration options of all
the unit types. These options need to be configured in the \[Unit\]
or \[Install\] sections of the unit files.

In addition to the generic \[Unit\] and \[Install\] sections
described here, each unit may have a type-specific section, e.g.
\[Service\] for a service unit. See the respective man pages for
more information:
[systemd.service(5)](systemd.service.html#),
[systemd.socket(5)](systemd.socket.html#),
[systemd.device(5)](systemd.device.html#),
[systemd.mount(5)](systemd.mount.html#),
[systemd.automount(5)](systemd.automount.html#),
[systemd.swap(5)](systemd.swap.html#),
[systemd.target(5)](systemd.target.html#),
[systemd.path(5)](systemd.path.html#),
[systemd.timer(5)](systemd.timer.html#),
[systemd.slice(5)](systemd.slice.html#),
[systemd.scope(5)](systemd.scope.html#).


Unit files are loaded from a set of paths determined during compilation, described in the next
section.

Valid unit names consist of a "unit name prefix", and a suffix specifying the unit type which
begins with a dot. The "unit name prefix" must consist of one or more valid characters (ASCII letters,
digits, " `:`", " `-`", " `_`", " `.`", and
" `\`"). The total length of the unit name including the suffix must not exceed 255
characters. The unit type suffix must be one of " `.service`", " `.socket`",
" `.device`", " `.mount`", " `.automount`",
" `.swap`", " `.target`", " `.path`",
" `.timer`", " `.slice`", or " `.scope`".

Unit names can be parameterized by a single argument called the "instance name". The unit is then
constructed based on a "template file" which serves as the definition of multiple services or other
units. A template unit must have a single " `@`" at the end of the unit name prefix (right
before the type suffix). The name of the full unit is formed by inserting the instance name between
" `@`" and the unit type suffix. In the unit file itself, the instance parameter may be
referred to using " `%i`" and other specifiers, see below.

Unit files may contain additional options on top of those listed here. If systemd encounters an
unknown option, it will write a warning log message but continue loading the unit. If an option or
section name is prefixed with `X-`, it is ignored completely by systemd. Options within an
ignored section do not need the prefix. Applications may use this to include additional information in
the unit files. To access those options, applications need to parse the unit files on their own.

Units can be aliased (have an alternative name), by creating a symlink from the new name to the
existing name in one of the unit search paths. For example, `systemd-networkd.service`
has the alias `dbus-org.freedesktop.network1.service`, created during installation as
a symlink, so when **systemd** is asked through D-Bus to load
`dbus-org.freedesktop.network1.service`, it'll load
`systemd-networkd.service`. As another example, `default.target` —
the default system target started at boot — is commonly aliased to either
`multi-user.target` or `graphical.target` to select what is started
by default. Alias names may be used in commands like **disable**,
**start**, **stop**, **status**, and similar, and in all
unit dependency directives, including `Wants=`, `Requires=`,
`Before=`, `After=`. Aliases cannot be used with the
**preset** command.

Aliases obey the following restrictions: a unit of a certain type (" `.service`",
" `.socket`", …) can only be aliased by a name with the same type suffix. A plain unit (not
a template or an instance), may only be aliased by a plain name. A template instance may only be aliased
by another template instance, and the instance part must be identical. A template may be aliased by
another template (in which case the alias applies to all instances of the template). As a special case, a
template instance (e.g. " `alias@inst.service`") may be a symlink to different template
(e.g. " `template@inst.service`"). In that case, just this specific instance is aliased,
while other instances of the template (e.g. " `alias@foo.service`",
" `alias@bar.service`") are not aliased. Those rules preserve the requirement that the
instance (if any) is always uniquely defined for a given unit and all its aliases. The target of alias
symlink must point to a valid unit file location, i.e. the symlink target name must match the symlink
source name as described, and the destination path must be in one of the unit search paths, see UNIT FILE
LOAD PATH section below for more details. Note that the target file might not exist, i.e. the symlink may
be dangling.

Unit files may specify aliases through the `Alias=` directive in the \[Install\]
section. When the unit is enabled, symlinks will be created for those names, and removed when the unit is
disabled. For example, `reboot.target` specifies
`Alias=ctrl-alt-del.target`, so when enabled, the symlink
`/etc/systemd/system/ctrl-alt-del.target` pointing to the
`reboot.target` file will be created, and when
**Ctrl**+**Alt**+**Del** is invoked,
**systemd** will look for `ctrl-alt-del.target`, follow the symlink to
`reboot.target`, and execute `reboot.service` as part of that target.
**systemd** does not look at the \[Install\] section at all during normal operation, so any
directives in that section only have an effect through the symlinks created during enablement.

Along with a unit file `foo.service`, the directory
`foo.service.wants/` may exist. All unit files symlinked from such a directory are
implicitly added as dependencies of type `Wants=` to the unit. Similar functionality
exists for `Requires=` type dependencies as well, the directory suffix is
`.requires/` in this case. This functionality is useful to hook units into the
start-up of other units, without having to modify their unit files. For details about the semantics of
`Wants=` and `Requires=`, see below. The preferred way to create
symlinks in the `.wants/` or `.requires/` directories is by
specifying the dependency in \[Install\] section of the target unit, and creating the symlink in the file
system with the **enable** or **preset** commands of
[systemctl(1)](systemctl.html#). The
target can be a normal unit (either plain or a specific instance of a template unit). In case when the
source unit is a template, the target can also be a template, in which case the instance will be
"propagated" to the target unit to form a valid unit instance. The target of symlinks in
`.wants/` or `.requires/` must thus point to a valid unit file
location, i.e. the symlink target name must satisfy the described requirements, and the destination path
must be in one of the unit search paths, see UNIT FILE LOAD PATH section below for more details. Note
that the target file might not exist, i.e. the symlink may be dangling.

Along with a unit file `foo.service`, a "drop-in" directory
`foo.service.d/` may exist. All files with the suffix
" `.conf`" from this directory will be merged in the alphanumeric order and parsed
after the main unit file itself has been parsed. This is useful to alter or add configuration
settings for a unit, without having to modify unit files. Each drop-in file must contain appropriate
section headers. For instantiated units, this logic will first look for the instance
" `.d/`" subdirectory (e.g. " `foo@bar.service.d/`") and read its
" `.conf`" files, followed by the template " `.d/`" subdirectory (e.g.
" `foo@.service.d/`") and the " `.conf`" files there. Moreover, for unit
names containing dashes (" `-`"), the set of directories generated by repeatedly
truncating the unit name after all dashes is searched too. Specifically, for a unit name
`foo-bar-baz.service` not only the regular drop-in directory
`foo-bar-baz.service.d/` is searched but also both `foo-bar-.service.d/` and
`foo-.service.d/`. This is useful for defining common drop-ins for a set of related units, whose
names begin with a common prefix. This scheme is particularly useful for mount, automount and slice units, whose
systematic naming structure is built around dashes as component separators. Note that equally named drop-in files
further down the prefix hierarchy override those further up,
i.e. `foo-bar-.service.d/10-override.conf` overrides
`foo-.service.d/10-override.conf`.

In cases of unit aliases (described above), dropins for the aliased name and all aliases are
loaded. In the example of `default.target` aliasing
`graphical.target`, `default.target.d/`,
`default.target.wants/`, `default.target.requires/`,
`graphical.target.d/`, `graphical.target.wants/`,
`graphical.target.requires/` would all be read. For templates, dropins for the
template, any template aliases, the template instance, and all alias instances are read. When just a
specific template instance is aliased, then the dropins for the target template, the target template
instance, and the alias template instance are read.

In addition to `/etc/systemd/system`, the drop-in " `.d/`"
directories for system services can be placed in `/usr/lib/systemd/system` or
`/run/systemd/system` directories. Drop-in files in `/etc/`
take precedence over those in `/run/` which in turn take precedence over those
in `/usr/lib/`. Drop-in files under any of these directories take precedence
over unit files wherever located. Multiple drop-in files with different names are applied in
lexicographic order, regardless of which of the directories they reside in.

Units also support a top-level drop-in with `type.d/`,
where _`type`_ may be e.g. " `service`" or " `socket`",
that allows altering or adding to the settings of all corresponding unit files on the system.
The formatting and precedence of applying drop-in configurations follow what is defined above.
Files in `type.d/` have lower precedence compared
to files in name-specific override directories. The usual rules apply: multiple drop-in files
with different names are applied in lexicographic order, regardless of which of the directories
they reside in, so a file in `type.d/` applies
to a unit only if there are no drop-ins or masks with that name in directories with higher
precedence. See Examples.

Note that while systemd offers a flexible dependency system
between units it is recommended to use this functionality only
sparingly and instead rely on techniques such as bus-based or
socket-based activation which make dependencies implicit,
resulting in a both simpler and more flexible system.

As mentioned above, a unit may be instantiated from a template file. This allows creation
of multiple units from a single configuration file. If systemd looks for a unit configuration
file, it will first search for the literal unit name in the file system. If that yields no
success and the unit name contains an " `@`" character, systemd will look for a
unit template that shares the same name but with the instance string (i.e. the part between the
" `@`" character and the suffix) removed. Example: if a service
`getty@tty3.service` is requested and no file by that name is found, systemd
will look for `getty@.service` and instantiate a service from that
configuration file if it is found.

To refer to the instance string from within the
configuration file you may use the special " `%i`"
specifier in many of the configuration options. See below for
details.

If a unit file is empty (i.e. has the file size 0) or is
symlinked to `/dev/null`, its configuration
will not be loaded and it appears with a load state of
" `masked`", and cannot be activated. Use this as an
effective way to fully disable a unit, making it impossible to
start it even manually.

Files (including directories) with names that match certain patterns are
generally ignored. This includes names that start with a " `.`" or
end with a " `.ignore`".

The unit file format is covered by the
[Interface\
Portability and Stability Promise](https://systemd.io/PORTABILITY_AND_STABILITY/).

*Based on [systemd.unit(5)](https://www.freedesktop.org/software/systemd/man/systemd.unit.html) official documentation.*

### Description=

A brief, meaningful, human-readable text identifying the unit. This may be used by
**systemd** (and suitable UIs) as a user-visible label for the unit, so this string
should identify the unit rather than just describe it, despite the name. This string also should not
just repeat the unit name. " `Apache HTTP Server`" or " `Postfix Mail
        Server`" are good examples. Bad examples are " `high-performance lightweight HTTP
        server`" (too generic) or " `Apache`" (meaningless for people who do not know
the Apache HTTP server project, duplicates the unit name). **systemd** may use this
string as a noun in status messages (" `Starting
        Description...`", " `Started
        Description.`", " `Reached target
        Description.`", " `Failed to start
        Description.`"), so it should be capitalized, and should not be a
full sentence, or a phrase with a verb conjugated in the present continuous, or end in a full
stop. Bad examples include " `exiting the container`", " `updating the database
        once per day.`", or " `OpenSSH server second instance daemon`".

Added in version 201.

### Documentation=

A space-separated list of URIs referencing
documentation for this unit or its configuration. Accepted are
only URIs of the types " `http://`",
" `https://`", " `file:`",
" `info:`", " `man:`". For more
information about the syntax of these URIs, see [uri(7)](https://man7.org/linux/man-pages/man7/uri.7.html).
The URIs should be listed in order of relevance, starting with
the most relevant. It is a good idea to first reference
documentation that explains what the unit's purpose is,
followed by how it is configured, followed by any other
related documentation. This option may be specified more than
once, in which case the specified list of URIs is merged. If
the empty string is assigned to this option, the list is reset
and all prior assignments will have no
effect.

Added in version 201.

### Wants=

Configures (weak) requirement dependencies on other units. This option may be
specified more than once or multiple space-separated units may be specified in one option in which
case dependencies for all listed names will be created. Dependencies of this type may also be
configured outside of the unit configuration file by adding a symlink to a
`.wants/` directory accompanying the unit file. For details, see above.

Units listed in this option will be started if the configuring unit is. However, if the listed
units fail to start or cannot be added to the transaction, this has no impact on the validity of the
transaction as a whole, and this unit will still be started. This is the recommended way to hook
the start-up of one unit to the start-up of another unit.

Note that requirement dependencies do not influence the order in which services are started or
stopped. This has to be configured independently with the `After=` or
`Before=` options. If unit `foo.service` pulls in unit
`bar.service` as configured with `Wants=` and no ordering is
configured with `After=` or `Before=`, then both units will be
started simultaneously and without any delay between them if `foo.service` is
activated.

Added in version 201.

### Requires=

Similar to `Wants=`, but declares a stronger requirement
dependency. Dependencies of this type may also be configured by adding a symlink to a
`.requires/` directory accompanying the unit file.

If this unit gets activated, the units listed will be activated as well. If one of
the other units fails to activate, and an ordering dependency `After=` on the
failing unit is set, this unit will not be started. Besides, with or without specifying
`After=`, this unit will be stopped (or restarted) if one of the other units is
explicitly stopped (or restarted).

Often, it is a better choice to use `Wants=` instead of
`Requires=` in order to achieve a system that is more robust when dealing with
failing services.

Note that this dependency type does not imply that the other unit always has to be in active state when
this unit is running. Specifically: failing condition checks (such as `ConditionPathExists=`,
`ConditionPathIsSymbolicLink=`, … — see below) do not cause the start job of a unit with a
`Requires=` dependency on it to fail. Also, some unit types may deactivate on their own (for
example, a service process may decide to exit cleanly, or a device may be unplugged by the user), which is not
propagated to units having a `Requires=` dependency. Use the `BindsTo=`
dependency type together with `After=` to ensure that a unit may never be in active state
without a specific other unit also in active state (see below).

Added in version 201.

### Requisite=

Similar to `Requires=`. However, if the units listed here
are not started already, they will not be started and the starting of this unit will fail
immediately. `Requisite=` does not imply an ordering dependency, even if
both units are started in the same transaction. Hence this setting should usually be
combined with `After=`, to ensure this unit is not started before the other
unit.

When `Requisite=b.service` is used on
`a.service`, this dependency will show as
`RequisiteOf=a.service` in property listing of
`b.service`. `RequisiteOf=`
dependency cannot be specified directly.

Added in version 201.

### BindsTo=

Configures requirement dependencies, very similar in style to
`Requires=`. However, this dependency type is stronger: in addition to the effects of
`Requires=`, which already stops (or restarts) the configuring unit when a listed unit is
explicitly stopped (or restarted), it also does so when a listed unit stops unexpectedly (which includes when it
fails).
Units can suddenly, unexpectedly enter inactive state for different reasons: the main process of a service unit
might terminate on its own choice, the backing device of a device unit might be unplugged or the mount point of
a mount unit might be unmounted without involvement of the system and service manager.

When used in conjunction with `After=` on the same unit the behaviour of
`BindsTo=` is even stronger. In this case, the unit bound to strictly has to be in active
state for this unit to also be in active state. This not only means a unit bound to another unit that suddenly
enters inactive state, but also one that is bound to another unit that gets skipped due to an unmet condition
check (such as `ConditionPathExists=`, `ConditionPathIsSymbolicLink=`, … —
see below) will be stopped, should it be running. Hence, in many cases it is best to combine
`BindsTo=` with `After=`.

When `BindsTo=b.service` is used on
`a.service`, this dependency will show as
`BoundBy=a.service` in property listing of
`b.service`. `BoundBy=`
dependency cannot be specified directly.

Added in version 201.

### PartOf=

Configures dependencies similar to
`Requires=`, but limited to stopping and
restarting of units. When systemd stops or restarts the units
listed here, the action is propagated to this unit. Note that
this is a one-way dependency — changes to this unit do not
affect the listed units.

When `PartOf=b.service` is used on
`a.service`, this dependency will show as
`ConsistsOf=a.service` in property listing of
`b.service`. `ConsistsOf=`
dependency cannot be specified directly.

Added in version 201.

### Upholds=

Configures dependencies similar to `Wants=`, but as long as this unit
is up, all units listed in `Upholds=` are started whenever found to be inactive or
failed, and no job is queued for them. While a `Wants=` dependency on another unit
has a one-time effect when this units started, a `Upholds=` dependency on it has a
continuous effect, constantly restarting the unit if necessary. This is an alternative to the
`Restart=` setting of service units, to ensure they are kept running whatever
happens. The restart happens without delay, and usual per-unit rate-limit applies.

When `Upholds=b.service` is used on `a.service`, this
dependency will show as `UpheldBy=a.service` in the property listing of
`b.service`.

Added in version 249.

### Conflicts=

A space-separated list of unit names. Configures negative requirement
dependencies. If a unit has a `Conflicts=` requirement on a set of other units,
then starting it will stop all of them and starting any of them will stop it.

Note that this setting does not imply an ordering dependency, similarly to the
`Wants=` and `Requires=` dependencies described above. This means
that to ensure that the conflicting unit is stopped before the other unit is started, an
`After=` or `Before=` dependency must be declared. It does not
matter which of the two ordering dependencies is used, because stop jobs are always ordered before
start jobs, see the discussion in `Before=`/ `After=` below.

If unit A that conflicts with unit B is scheduled to
be started at the same time as B, the transaction will either
fail (in case both are required parts of the transaction) or be
modified to be fixed (in case one or both jobs are not a
required part of the transaction). In the latter case, the job
that is not required will be removed, or in case both are
not required, the unit that conflicts will be started and the
unit that is conflicted is stopped.

Added in version 201.

### Before=

These two settings expect a space-separated list of unit names. They may be specified
more than once, in which case dependencies for all listed names are created.

Those two settings configure ordering dependencies between units. If unit
`foo.service` contains the setting `Before=bar.service` and both
units are being started, `bar.service`'s start-up is delayed until
`foo.service` has finished starting up. `After=` is the inverse
of `Before=`, i.e. while `Before=` ensures that the configured unit
is started before the listed unit begins starting up, `After=` ensures the opposite,
that the listed unit is fully started up before the configured unit is started.

When two units with an ordering dependency between them are shut down, the inverse of the
start-up order is applied. I.e. if a unit is configured with `After=` on another
unit, the former is stopped before the latter if both are shut down. Given two units with any
ordering dependency between them, if one unit is shut down and the other is started up, the shutdown
is ordered before the start-up. It does not matter if the ordering dependency is
`After=` or `Before=`, in this case. It also does not matter which
of the two is shut down, as long as one is shut down and the other is started up; the shutdown is
ordered before the start-up in all cases. If two units have no ordering dependencies between them,
they are shut down or started up simultaneously, and no ordering takes place. It depends on the unit
type when precisely a unit has finished starting up. Most importantly, for service units start-up is
considered completed for the purpose of `Before=`/ `After=` when all
its configured start-up commands have been invoked and they either failed or reported start-up
success. Note that this includes `ExecStartPost=` (or
`ExecStopPost=` for the shutdown case).

Note that those settings are independent of and orthogonal to the requirement dependencies as
configured by `Requires=`, `Wants=`, `Requisite=`,
or `BindsTo=`. It is a common pattern to include a unit name in both the
`After=` and `Wants=` options, in which case the unit listed will
be started before the unit that is configured with these options.

Note that `Before=` dependencies on device units have no effect and are not
supported. Devices generally become available as a result of an external hotplug event, and systemd
creates the corresponding device unit without delay.

Added in version 201.

### After=

These two settings expect a space-separated list of unit names. They may be specified
more than once, in which case dependencies for all listed names are created.

Those two settings configure ordering dependencies between units. If unit
`foo.service` contains the setting `Before=bar.service` and both
units are being started, `bar.service`'s start-up is delayed until
`foo.service` has finished starting up. `After=` is the inverse
of `Before=`, i.e. while `Before=` ensures that the configured unit
is started before the listed unit begins starting up, `After=` ensures the opposite,
that the listed unit is fully started up before the configured unit is started.

When two units with an ordering dependency between them are shut down, the inverse of the
start-up order is applied. I.e. if a unit is configured with `After=` on another
unit, the former is stopped before the latter if both are shut down. Given two units with any
ordering dependency between them, if one unit is shut down and the other is started up, the shutdown
is ordered before the start-up. It does not matter if the ordering dependency is
`After=` or `Before=`, in this case. It also does not matter which
of the two is shut down, as long as one is shut down and the other is started up; the shutdown is
ordered before the start-up in all cases. If two units have no ordering dependencies between them,
they are shut down or started up simultaneously, and no ordering takes place. It depends on the unit
type when precisely a unit has finished starting up. Most importantly, for service units start-up is
considered completed for the purpose of `Before=`/ `After=` when all
its configured start-up commands have been invoked and they either failed or reported start-up
success. Note that this includes `ExecStartPost=` (or
`ExecStopPost=` for the shutdown case).

Note that those settings are independent of and orthogonal to the requirement dependencies as
configured by `Requires=`, `Wants=`, `Requisite=`,
or `BindsTo=`. It is a common pattern to include a unit name in both the
`After=` and `Wants=` options, in which case the unit listed will
be started before the unit that is configured with these options.

Note that `Before=` dependencies on device units have no effect and are not
supported. Devices generally become available as a result of an external hotplug event, and systemd
creates the corresponding device unit without delay.

Added in version 201.

### OnFailure=

A space-separated list of one or more units that are activated when this unit enters
the " `failed`" state.

Added in version 201.

### OnSuccess=

A space-separated list of one or more units that are activated when this unit enters
the " `inactive`" state.

Added in version 249.

### PropagatesReloadTo=

A space-separated list of one or more units to which reload requests from this unit
shall be propagated to, or units from which reload requests shall be propagated to this unit,
respectively. Issuing a reload request on a unit will automatically also enqueue reload requests on
all units that are linked to it using these two settings.

Added in version 201.

### ReloadPropagatedFrom=

A space-separated list of one or more units to which reload requests from this unit
shall be propagated to, or units from which reload requests shall be propagated to this unit,
respectively. Issuing a reload request on a unit will automatically also enqueue reload requests on
all units that are linked to it using these two settings.

Added in version 201.

### PropagatesStopTo=

A space-separated list of one or more units to which stop requests from this unit
shall be propagated to, or units from which stop requests shall be propagated to this unit,
respectively. Issuing a stop request on a unit will automatically also enqueue stop requests on all
units that are linked to it using these two settings.

Added in version 249.

### StopPropagatedFrom=

A space-separated list of one or more units to which stop requests from this unit
shall be propagated to, or units from which stop requests shall be propagated to this unit,
respectively. Issuing a stop request on a unit will automatically also enqueue stop requests on all
units that are linked to it using these two settings.

Added in version 249.

### JoinsNamespaceOf=

For units that start processes (such as service units), lists one or more other units
whose network and/or temporary file namespace to join. If this is specified on a unit (say,
`a.service` has `JoinsNamespaceOf=b.service`), then the inverse
dependency ( `JoinsNamespaceOf=a.service` for b.service) is implied. This only
applies to unit types which support the `PrivateNetwork=`,
`NetworkNamespacePath=`, `PrivateIPC=`,
`IPCNamespacePath=`, and `PrivateTmp=` directives (see
[systemd.exec(5)](systemd.exec.html#) for
details). If a unit that has this setting set is started, its processes will see the same
`/tmp/`, `/var/tmp/`, IPC namespace and network namespace as
one listed unit that is started. If multiple listed units are already started and these do not share
their namespace, then it is not defined which namespace is joined. Note that this setting only has an
effect if `PrivateNetwork=`/ `NetworkNamespacePath=`,
`PrivateIPC=`/ `IPCNamespacePath=` and/or
`PrivateTmp=` is enabled for both the unit that joins the namespace and the unit
whose namespace is joined.

Added in version 209.

### RequiresMountsFor=

Takes a space-separated list of absolute
paths. Automatically adds dependencies of type
`Requires=` and `After=` for
all mount units required to access the specified path.

Mount points marked with `noauto` are not
mounted automatically through `local-fs.target`,
but are still honored for the purposes of this option, i.e. they
will be pulled in by this unit.

Added in version 201.

### WantsMountsFor=

Same as `RequiresMountsFor=`,
but adds dependencies of type `Wants=` instead
of `Requires=`.

Added in version 256.

### OnSuccessJobMode=

Takes a value of
" `fail`",
" `replace`",
" `replace-irreversibly`",
" `isolate`",
" `flush`",
" `ignore-dependencies`" or
" `ignore-requirements`". Defaults to
" `replace`". Specifies how the units listed in
`OnSuccess=`/ `OnFailure=` will be enqueued. See
[systemctl(1)](systemctl.html#)'s
`--job-mode=` option for details on the
possible values. If this is set to " `isolate`",
only a single unit may be listed in
`OnSuccess=`/ `OnFailure=`.

Added in version 209.

### OnFailureJobMode=

Takes a value of
" `fail`",
" `replace`",
" `replace-irreversibly`",
" `isolate`",
" `flush`",
" `ignore-dependencies`" or
" `ignore-requirements`". Defaults to
" `replace`". Specifies how the units listed in
`OnSuccess=`/ `OnFailure=` will be enqueued. See
[systemctl(1)](systemctl.html#)'s
`--job-mode=` option for details on the
possible values. If this is set to " `isolate`",
only a single unit may be listed in
`OnSuccess=`/ `OnFailure=`.

Added in version 209.

### IgnoreOnIsolate=

Takes a boolean argument. If `true`, this unit will not be stopped
when isolating another unit. Defaults to `false` for service, target, socket, timer,
and path units, and `true` for slice, scope, device, swap, mount, and automount
units.

Added in version 201.

### StopWhenUnneeded=

Takes a boolean argument. If
`true`, this unit will be stopped when it is no
longer used. Note that, in order to minimize the work to be
executed, systemd will not stop units by default unless they
are conflicting with other units, or the user explicitly
requested their shut down. If this option is set, a unit will
be automatically cleaned up if no other active unit requires
it. Defaults to `false`.

Added in version 201.

### RefuseManualStart=

Takes a boolean argument. If
`true`, this unit can only be activated or
deactivated indirectly. In this case, explicit start-up or
termination requested by the user is denied, however if it is
started or stopped as a dependency of another unit, start-up
or termination will succeed. This is mostly a safety feature
to ensure that the user does not accidentally activate units
that are not intended to be activated explicitly, and not
accidentally deactivate units that are not intended to be
deactivated. These options default to
`false`.

Added in version 201.

### RefuseManualStop=

Takes a boolean argument. If
`true`, this unit can only be activated or
deactivated indirectly. In this case, explicit start-up or
termination requested by the user is denied, however if it is
started or stopped as a dependency of another unit, start-up
or termination will succeed. This is mostly a safety feature
to ensure that the user does not accidentally activate units
that are not intended to be activated explicitly, and not
accidentally deactivate units that are not intended to be
deactivated. These options default to
`false`.

Added in version 201.

### AllowIsolate=

Takes a boolean argument. If
`true`, this unit may be used with the
**systemctl isolate** command. Otherwise, this
will be refused. It probably is a good idea to leave this
disabled except for target units that shall be used similar to
runlevels in SysV init systems, just as a precaution to avoid
unusable system states. This option defaults to
`false`.

Added in version 201.

### DefaultDependencies=

Takes a boolean argument. If
`yes`, (the default), a few default
dependencies will implicitly be created for the unit. The
actual dependencies created depend on the unit type. For
example, for service units, these dependencies ensure that the
service is started only after basic system initialization is
completed and is properly terminated on system shutdown. See
the respective man pages for details. Generally, only services
involved with early boot or late shutdown should set this
option to `no`. It is highly recommended to
leave this option enabled for the majority of common units. If
set to `no`, this option does not disable
all implicit dependencies, just non-essential
ones.

Added in version 201.

### SurviveFinalKillSignal=

Takes a boolean argument. Defaults to `no`. If `yes`,
processes belonging to this unit will not be sent the final " `SIGTERM`" and
" `SIGKILL`" signals during the final phase of the system shutdown process.
This functionality replaces the older mechanism that allowed a program to set
" `argv[0][0] = '@'`" as described at
[systemd and Storage Daemons for the Root File\
System](https://systemd.io/ROOT_STORAGE_DAEMONS), which however continues to be supported.

Added in version 255.

### CollectMode=

Tweaks the "garbage collection" algorithm for this unit. Takes one of `inactive`
or `inactive-or-failed`. If set to `inactive` the unit will be unloaded if it is
in the `inactive` state and is not referenced by clients, jobs or other units — however it
is not unloaded if it is in the `failed` state. In `failed` mode, failed
units are not unloaded until the user invoked **systemctl reset-failed** on them to reset the
`failed` state, or an equivalent command. This behaviour is altered if this option is set to
`inactive-or-failed`: in this case, the unit is unloaded even if the unit is in a
`failed` state, and thus an explicitly resetting of the `failed` state is
not necessary. Note that if this mode is used unit results (such as exit codes, exit signals, consumed
resources, …) are flushed out immediately after the unit completed, except for what is stored in the logging
subsystem. Defaults to `inactive`.

Added in version 236.

### FailureAction=

Configure the action to take when the unit stops and enters a failed state or
inactive state. Takes one of `none`, `reboot`,
`reboot-force`, `reboot-immediate`, `poweroff`,
`poweroff-force`, `poweroff-immediate`, `exit`,
`exit-force`, `soft-reboot`, `soft-reboot-force`,
`kexec`, `kexec-force`, `halt`,
`halt-force` and `halt-immediate`. In system mode, all options are
allowed. In user mode, only `none`, `exit`, and
`exit-force` are allowed. Both options default to `none`.

If `none` is set, no action will be triggered. `reboot` causes a
reboot following the normal shutdown procedure (i.e. equivalent to **systemctl**
**reboot**). `reboot-force` causes a forced reboot which will terminate all
processes forcibly but should cause no dirty file systems on reboot (i.e. equivalent to
**systemctl reboot -f**) and `reboot-immediate` causes immediate
execution of the
[reboot(2)](https://man7.org/linux/man-pages/man2/reboot.2.html) system
call, which might result in data loss (i.e. equivalent to **systemctl reboot -ff**).
Similarly, `poweroff`, `poweroff-force`,
`poweroff-immediate`, `kexec`, `kexec-force`,
`halt`, `halt-force` and `halt-immediate` have the
effect of powering down the system, executing kexec, and halting the system respectively with similar
semantics. `exit` causes the manager to exit following the normal shutdown procedure,
and `exit-force` causes it terminate without shutting down services. When
`exit` or `exit-force` is used by default the exit status of the main
process of the unit (if this applies) is returned from the service manager. However, this may be
overridden with
`FailureActionExitStatus=`/ `SuccessActionExitStatus=`, see below.
`soft-reboot` will trigger a userspace reboot operation.
`soft-reboot-force` does that too, but does not go through the shutdown transaction
beforehand.

Added in version 236.

### SuccessAction=

Configure the action to take when the unit stops and enters a failed state or
inactive state. Takes one of `none`, `reboot`,
`reboot-force`, `reboot-immediate`, `poweroff`,
`poweroff-force`, `poweroff-immediate`, `exit`,
`exit-force`, `soft-reboot`, `soft-reboot-force`,
`kexec`, `kexec-force`, `halt`,
`halt-force` and `halt-immediate`. In system mode, all options are
allowed. In user mode, only `none`, `exit`, and
`exit-force` are allowed. Both options default to `none`.

If `none` is set, no action will be triggered. `reboot` causes a
reboot following the normal shutdown procedure (i.e. equivalent to **systemctl**
**reboot**). `reboot-force` causes a forced reboot which will terminate all
processes forcibly but should cause no dirty file systems on reboot (i.e. equivalent to
**systemctl reboot -f**) and `reboot-immediate` causes immediate
execution of the
[reboot(2)](https://man7.org/linux/man-pages/man2/reboot.2.html) system
call, which might result in data loss (i.e. equivalent to **systemctl reboot -ff**).
Similarly, `poweroff`, `poweroff-force`,
`poweroff-immediate`, `kexec`, `kexec-force`,
`halt`, `halt-force` and `halt-immediate` have the
effect of powering down the system, executing kexec, and halting the system respectively with similar
semantics. `exit` causes the manager to exit following the normal shutdown procedure,
and `exit-force` causes it terminate without shutting down services. When
`exit` or `exit-force` is used by default the exit status of the main
process of the unit (if this applies) is returned from the service manager. However, this may be
overridden with
`FailureActionExitStatus=`/ `SuccessActionExitStatus=`, see below.
`soft-reboot` will trigger a userspace reboot operation.
`soft-reboot-force` does that too, but does not go through the shutdown transaction
beforehand.

Added in version 236.

### FailureActionExitStatus=

Controls the exit status to propagate back to an invoking container manager (in case of a
system service) or service manager (in case of a user manager) when the
`FailureAction=`/ `SuccessAction=` are set to `exit` or
`exit-force` and the action is triggered. By default, the exit status of the main process of the
triggering unit (if this applies) is propagated. Takes a value in the range 0…255 or the empty string to
request default behaviour.

Added in version 240.

### SuccessActionExitStatus=

Controls the exit status to propagate back to an invoking container manager (in case of a
system service) or service manager (in case of a user manager) when the
`FailureAction=`/ `SuccessAction=` are set to `exit` or
`exit-force` and the action is triggered. By default, the exit status of the main process of the
triggering unit (if this applies) is propagated. Takes a value in the range 0…255 or the empty string to
request default behaviour.

Added in version 240.

### JobTimeoutSec=

`JobTimeoutSec=` specifies a timeout for the whole job that starts
running when the job is queued. `JobRunningTimeoutSec=` specifies a timeout that
starts running when the queued job is actually started. If either limit is reached, the job will be
cancelled, the unit however will not change state or even enter the " `failed`" mode.


Both settings take a time span with the default unit of seconds, but other units may be
specified, see
[systemd.time(7)](systemd.time.html#).
The default is " `infinity`" (job timeouts disabled), except for device units where
`JobRunningTimeoutSec=` defaults to `DefaultDeviceTimeoutSec=`.


Note: these timeouts are independent from any unit-specific timeouts (for example, the timeout
set with `TimeoutStartSec=` in service units). The job timeout has no effect on the
unit itself. Or in other words: unit-specific timeouts are useful to abort unit state changes, and
revert them. The job timeout set with this option however is useful to abort only the job waiting for
the unit state to change.

Added in version 201.

### JobRunningTimeoutSec=

`JobTimeoutSec=` specifies a timeout for the whole job that starts
running when the job is queued. `JobRunningTimeoutSec=` specifies a timeout that
starts running when the queued job is actually started. If either limit is reached, the job will be
cancelled, the unit however will not change state or even enter the " `failed`" mode.


Both settings take a time span with the default unit of seconds, but other units may be
specified, see
[systemd.time(7)](systemd.time.html#).
The default is " `infinity`" (job timeouts disabled), except for device units where
`JobRunningTimeoutSec=` defaults to `DefaultDeviceTimeoutSec=`.


Note: these timeouts are independent from any unit-specific timeouts (for example, the timeout
set with `TimeoutStartSec=` in service units). The job timeout has no effect on the
unit itself. Or in other words: unit-specific timeouts are useful to abort unit state changes, and
revert them. The job timeout set with this option however is useful to abort only the job waiting for
the unit state to change.

Added in version 201.

### JobTimeoutAction=

`JobTimeoutAction=` optionally configures an additional action to
take when the timeout is hit, see description of `JobTimeoutSec=` and
`JobRunningTimeoutSec=` above. It takes the same values as
`FailureAction=`/ `SuccessAction=`. Defaults to
`none`.

`JobTimeoutRebootArgument=` configures an optional reboot string to pass to
the [reboot(2)](https://man7.org/linux/man-pages/man2/reboot.2.html) system
call.

Added in version 240.

### JobTimeoutRebootArgument=

`JobTimeoutAction=` optionally configures an additional action to
take when the timeout is hit, see description of `JobTimeoutSec=` and
`JobRunningTimeoutSec=` above. It takes the same values as
`FailureAction=`/ `SuccessAction=`. Defaults to
`none`.

`JobTimeoutRebootArgument=` configures an optional reboot string to pass to
the [reboot(2)](https://man7.org/linux/man-pages/man2/reboot.2.html) system
call.

Added in version 240.

### StartLimitIntervalSec=

Configure unit start rate limiting. Units which are started more than
_`burst`_ times within an _`interval`_ time span are
not permitted to start any more. Use `StartLimitIntervalSec=` to configure the
checking interval and `StartLimitBurst=` to configure how many starts per interval
are allowed.

_`interval`_ is a time span with the default unit of seconds, but other
units may be specified, see
[systemd.time(7)](systemd.time.html#).
The special value " `infinity`" can be used to limit the total number of start
attempts, even if they happen at large time intervals.
Defaults to `DefaultStartLimitIntervalSec=` in manager configuration file, and may
be set to 0 to disable any kind of rate limiting. _`burst`_ is a number and
defaults to `DefaultStartLimitBurst=` in manager configuration file.

These configuration options are particularly useful in conjunction with the service setting
`Restart=` (see
[systemd.service(5)](systemd.service.html#));
however, they apply to all kinds of starts (including manual), not just those triggered by the
`Restart=` logic.

Note that units which are configured for `Restart=`, and which reach the start
limit are not attempted to be restarted anymore; however, they may still be restarted manually or
from a timer or socket at a later point, after the _`interval`_ has passed.
From that point on, the restart logic is activated again. **systemctl reset-failed**
will cause the restart rate counter for a service to be flushed, which is useful if the administrator
wants to manually start a unit and the start limit interferes with that. Rate-limiting is enforced
after any unit condition checks are executed, and hence unit activations with failing conditions do
not count towards the rate limit.

When a unit is unloaded due to the garbage collection logic (see above) its rate limit counters
are flushed out too. This means that configuring start rate limiting for a unit that is not
referenced continuously has no effect.

This setting does not apply to slice, target, device, and scope units, since they are unit
types whose activation may either never fail, or may succeed only a single time.

Added in version 229.

### StartLimitBurst=

Configure unit start rate limiting. Units which are started more than
_`burst`_ times within an _`interval`_ time span are
not permitted to start any more. Use `StartLimitIntervalSec=` to configure the
checking interval and `StartLimitBurst=` to configure how many starts per interval
are allowed.

_`interval`_ is a time span with the default unit of seconds, but other
units may be specified, see
[systemd.time(7)](systemd.time.html#).
The special value " `infinity`" can be used to limit the total number of start
attempts, even if they happen at large time intervals.
Defaults to `DefaultStartLimitIntervalSec=` in manager configuration file, and may
be set to 0 to disable any kind of rate limiting. _`burst`_ is a number and
defaults to `DefaultStartLimitBurst=` in manager configuration file.

These configuration options are particularly useful in conjunction with the service setting
`Restart=` (see
[systemd.service(5)](systemd.service.html#));
however, they apply to all kinds of starts (including manual), not just those triggered by the
`Restart=` logic.

Note that units which are configured for `Restart=`, and which reach the start
limit are not attempted to be restarted anymore; however, they may still be restarted manually or
from a timer or socket at a later point, after the _`interval`_ has passed.
From that point on, the restart logic is activated again. **systemctl reset-failed**
will cause the restart rate counter for a service to be flushed, which is useful if the administrator
wants to manually start a unit and the start limit interferes with that. Rate-limiting is enforced
after any unit condition checks are executed, and hence unit activations with failing conditions do
not count towards the rate limit.

When a unit is unloaded due to the garbage collection logic (see above) its rate limit counters
are flushed out too. This means that configuring start rate limiting for a unit that is not
referenced continuously has no effect.

This setting does not apply to slice, target, device, and scope units, since they are unit
types whose activation may either never fail, or may succeed only a single time.

Added in version 229.

### StartLimitAction=

Configure an additional action to take if the rate limit configured with
`StartLimitIntervalSec=` and `StartLimitBurst=` is hit. Takes the same
values as the `FailureAction=`/ `SuccessAction=` settings. If
`none` is set, hitting the rate limit will trigger no action except that
the start will not be permitted. Defaults to `none`.

Added in version 229.

### RebootArgument=

Configure the optional argument for the
[reboot(2)](https://man7.org/linux/man-pages/man2/reboot.2.html) system call if
`StartLimitAction=` or `FailureAction=` is a reboot action. This
works just like the optional argument to **systemctl reboot** command.

Added in version 229.

### SourcePath=

A path to a configuration file this unit has
been generated from. This is primarily useful for
implementation of generator tools that convert configuration
from an external configuration file format into native unit
files. This functionality should not be used in normal
units.

Added in version 201.

### ConditionArchitecture=

Check whether the system is running on a specific architecture. Takes one of
" `x86`",
" `x86-64`",
" `ppc`",
" `ppc-le`",
" `ppc64`",
" `ppc64-le`",
" `ia64`",
" `parisc`",
" `parisc64`",
" `s390`",
" `s390x`",
" `sparc`",
" `sparc64`",
" `mips`",
" `mips-le`",
" `mips64`",
" `mips64-le`",
" `alpha`",
" `arm`",
" `arm-be`",
" `arm64`",
" `arm64-be`",
" `sh`",
" `sh64`",
" `m68k`",
" `tilegx`",
" `cris`",
" `arc`",
" `arc-be`", or
" `native`".

Use
[systemd-analyze(1)](systemd-analyze.html#)
for the complete list of known architectures.

The architecture is determined from the information returned by
[uname(2)](https://man7.org/linux/man-pages/man2/uname.2.html)
and is thus subject to
[personality(2)](https://man7.org/linux/man-pages/man2/personality.2.html).
Note that a `Personality=` setting in the same unit file has no effect on this
condition. A special architecture name " `native`" is mapped to the architecture the
system manager itself is compiled for. The test may be negated by prepending an exclamation
mark.

Added in version 201.

### ConditionFirmware=

Check whether the system's firmware is of a certain type. The following values are
possible:

- " `uefi`" matches systems with EFI.

- " `device-tree`" matches systems with a device tree.


- " `device-tree-compatible(value)`"
matches systems with a device tree that are compatible with " `value`".


- " `smbios-field(field
              operator value)`" matches systems
with a SMBIOS field containing a certain value. _`field`_ is the name of
the SMBIOS field exposed as " `sysfs`" attribute file below
`/sys/class/dmi/id/`. _`operator`_ is one of
" `<`", " `<=`", " `>=`",
" `>`", " `==`", " `<>`" for version
comparisons, " `=`" and " `!=`" for literal string comparisons, or
" `$=`", " `!$=`" for shell-style glob comparisons.
_`value`_ is the expected value of the SMBIOS field value (possibly
containing shell style globs in case " `$=`"/" `!$=`" is used).



Added in version 249.

### ConditionVirtualization=

Check whether the system is executed in a virtualized environment and optionally
test whether it is a specific implementation. Takes either boolean value to check if being executed
in any virtualized environment, or one of
" `vm`" and
" `container`" to test against a generic type of virtualization solution, or one of
" `qemu`",
" `kvm`",
" `amazon`",
" `zvm`",
" `vmware`",
" `microsoft`",
" `oracle`",
" `powervm`",
" `xen`",
" `bochs`",
" `uml`",
" `bhyve`",
" `qnx`",
" `apple`",
" `sre`",
" `openvz`",
" `lxc`",
" `lxc-libvirt`",
" `systemd-nspawn`",
" `docker`",
" `podman`",
" `rkt`",
" `wsl`",
" `proot`",
" `pouch`",
" `acrn`" to test
against a specific implementation, or
" `private-users`" to check whether we are running in a user namespace. See
[systemd-detect-virt(1)](systemd-detect-virt.html#)
for a full list of known virtualization technologies and their identifiers. If multiple
virtualization technologies are nested, only the innermost is considered. The test may be negated
by prepending an exclamation mark.

Added in version 244.

### ConditionHost=

`ConditionHost=` may be used to match against the hostname,
machine ID, boot ID or product UUID of the host. This either takes a hostname string (optionally
with shell style globs) which is tested against the locally set hostname as returned by
[gethostname(2)](https://man7.org/linux/man-pages/man2/gethostname.2.html), or
a 128bit ID or UUID, formatted as string. The latter is compared against machine ID, boot ID and the
firmware product UUID if there is any. See
[machine-id(5)](machine-id.html#) for
details about the machine ID. The test may be negated by prepending an exclamation mark.

Added in version 244.

### ConditionKernelCommandLine=

`ConditionKernelCommandLine=` may be used to check whether a
specific kernel command line option is set (or if prefixed with the exclamation mark — unset). The
argument must either be a single word, or an assignment (i.e. two words, separated by
" `=`"). In the former case the kernel command line is searched for the word
appearing as is, or as left hand side of an assignment. In the latter case, the exact assignment is
looked for with right and left hand side matching. This operates on the kernel command line
communicated to userspace via `/proc/cmdline`, except when the service manager
is invoked as payload of a container manager, in which case the command line of `PID
          1` is used instead (i.e. `/proc/1/cmdline`).

Added in version 244.

### ConditionKernelVersion=

`ConditionKernelVersion=` may be used to check whether the kernel
version (as reported by **uname -r**) matches a certain expression, or if prefixed
with the exclamation mark, does not match. The argument must be a list of (potentially quoted)
expressions. Each expression starts with one of " `=`" or " `!=`" for
string comparisons, " `<`", " `<=`", " `==`",
" `<>`", " `>=`", " `>`" for version
comparisons, or " `$=`", " `!$=`" for a shell-style glob match. If no
operator is specified, " `$=`" is implied.

Note that using the kernel version string is an unreliable way to determine which features
are supported by a kernel, because of the widespread practice of backporting drivers, features, and
fixes from newer upstream kernels into older versions provided by distributions. Hence, this check
is inherently unportable and should not be used for units which may be used on different
distributions.

Added in version 244.

### ConditionVersion=

`ConditionVersion=` may be used to check whether a software
version matches a certain expression, or if prefixed with the exclamation mark, does not match.
The first argument is the software whose version has to be checked. Currently
" `kernel`", " `systemd`" and " `glibc`" are supported.
If this argument is omitted, " `kernel`" is implied. The second argument must be a
list of (potentially quoted) expressions. Each expression starts with one of " `=`"
or " `!=`" for string comparisons, " `<`", " `<=`",
" `==`", " `<>`", " `>=`",
" `>`" for version comparisons, or " `$=`", " `!$=`"
for a shell-style glob match. If no operator is specified, " `$=`" is implied.

Added in version 258.

### ConditionCredential=

`ConditionCredential=` may be used to check whether a credential
by the specified name was passed into the service manager. See [System and Service Credentials](https://systemd.io/CREDENTIALS) for details about
credentials. If used in services for the system service manager this may be used to conditionalize
services based on system credentials passed in. If used in services for the per-user service
manager this may be used to conditionalize services based on credentials passed into the
`unit@.service` service instance belonging to the user. The argument must be a
valid credential name.

Added in version 252.

### ConditionEnvironment=

`ConditionEnvironment=` may be used to check whether a specific
environment variable is set (or if prefixed with the exclamation mark — unset) in the service
manager's environment block.

The argument may be a single word, to check if the variable with this name is defined in the
environment block, or an assignment
(" `name=value`"), to check if
the variable with this exact value is defined. Note that the environment block of the service
manager itself is checked, i.e. not any variables defined with `Environment=` or
`EnvironmentFile=`, as described above. This is particularly useful when the
service manager runs inside a containerized environment or as per-user service manager, in order to
check for variables passed in by the enclosing container manager or PAM.

Added in version 246.

### ConditionSecurity=

`ConditionSecurity=` may be used to check whether the given
security technology is enabled on the system. Currently, the following values are recognized:

**Table 3. Recognized security technologies**

ValueDescriptionselinuxSELinux MACapparmorAppArmor MACtomoyoTomoyo MACsmackSMACK MACimaIntegrity Measurement Architecture (IMA)auditLinux Audit Frameworkuefi-securebootUEFI SecureBoottpm2Trusted Platform Module 2.0 (TPM2) (with full UEFI support, including the TCG PC Client
Platform Firmware Profile)cvmConfidential virtual machine (SEV/TDX)measured-ukiUnified Kernel Image with PCR 11 Measurements, as per [systemd-stub(7)](systemd-stub.html#).

Added in version 255.

The test may be negated by prepending an exclamation mark.

Added in version 244.

### ConditionCapability=

Check whether the given capability exists in the capability bounding set of the
service manager (i.e. this does not check whether capability is actually available in the permitted
or effective sets, see
[capabilities(7)](https://man7.org/linux/man-pages/man7/capabilities.7.html)
for details). Pass a capability name such as " `CAP_MKNOD`", possibly prefixed with
an exclamation mark to negate the check.

Added in version 244.

### ConditionACPower=

Check whether the system has AC power, or is exclusively battery powered at the
time of activation of the unit. This takes a boolean argument. If set to " `true`",
the condition will hold only if at least one AC connector of the system is connected to a power
source, or if no AC connectors are known. Conversely, if set to " `false`", the
condition will hold only if there is at least one AC connector known and all AC connectors are
disconnected from a power source.

Added in version 244.

### ConditionNeedsUpdate=

Takes one of `/var/` or `/etc/` as argument,
possibly prefixed with a " `!`" (to invert the condition). This condition may be
used to conditionalize units on whether the specified directory requires an update because
`/usr/`'s modification time is newer than the stamp file
`.updated` in the specified directory. This is useful to implement offline
updates of the vendor operating system resources in `/usr/` that require updating
of `/etc/` or `/var/` on the next following boot. Units making
use of this condition should order themselves before
[systemd-update-done.service(8)](systemd-update-done.service.html#),
to make sure they run before the stamp file's modification time gets reset indicating a completed
update.

If the `systemd.condition_needs_update=` option is specified on the kernel
command line (taking a boolean), it will override the result of this condition check, taking
precedence over any file modification time checks. If the kernel command line option is used,
`systemd-update-done.service` will not have immediate effect on any following
`ConditionNeedsUpdate=` checks, until the system is rebooted where the kernel
command line option is not specified anymore.

Note that to make this scheme effective, the timestamp of `/usr/` should
be explicitly updated after its contents are modified. The kernel will automatically update
modification timestamp on a directory only when immediate children of a directory are modified; an
modification of nested files will not automatically result in mtime of `/usr/`
being updated.

Also note that if the update method includes a call to execute appropriate post-update steps
itself, it should not touch the timestamp of `/usr/`. In a typical distribution
packaging scheme, packages will do any required update steps as part of the installation or
upgrade, to make package contents immediately usable. `ConditionNeedsUpdate=`
should be used with other update mechanisms where such an immediate update does not
happen.

Added in version 244.

### ConditionFirstBoot=

Takes a boolean argument. This condition may be used to conditionalize units on
whether the system is booting up for the first time. This roughly means that `/etc/`
was unpopulated when the system started booting (for details, see "First Boot Semantics" in
[machine-id(5)](machine-id.html#)).
First Boot is considered finished (this condition will evaluate as false) after the manager
has finished the startup phase.

This condition may be used to populate `/etc/` on the first boot after
factory reset, or when a new system instance boots up for the first time.

Note that the service manager itself will perform setup steps during First Boot: it will
initialize
[machine-id(5)](machine-id.html#) and
preset all units, enabling or disabling them according to the
[systemd.preset(5)](systemd.preset.html#)
settings. Additional setup may be performed via units with
`ConditionFirstBoot=yes`.

For robustness, units with `ConditionFirstBoot=yes` should order themselves
before `first-boot-complete.target` and pull in this passive target with
`Wants=`. This ensures that in a case of an aborted first boot, these units will
be re-run during the next system startup.

If the `systemd.condition_first_boot=` option is specified on the kernel
command line (taking a boolean), it will override the result of this condition check, taking
precedence over `/etc/machine-id` existence checks.

Added in version 244.

### ConditionPathExists=

Check for the existence of a file. If the specified absolute path name does not exist,
the condition will fail. If the absolute path name passed to
`ConditionPathExists=` is prefixed with an exclamation mark
(" `!`"), the test is negated, and the unit is only started if the path does not
exist.

Added in version 244.

### ConditionPathExistsGlob=

`ConditionPathExistsGlob=` is similar to
`ConditionPathExists=`, but checks for the existence of at least one file or
directory matching the specified globbing pattern.

Added in version 244.

### ConditionPathIsDirectory=

`ConditionPathIsDirectory=` is similar to
`ConditionPathExists=` but verifies that a certain path exists and is a
directory.

Added in version 244.

### ConditionPathIsSymbolicLink=

`ConditionPathIsSymbolicLink=` is similar to
`ConditionPathExists=` but verifies that a certain path exists and is a symbolic
link.

Added in version 244.

### ConditionPathIsMountPoint=

`ConditionPathIsMountPoint=` is similar to
`ConditionPathExists=` but verifies that a certain path exists and is a mount
point.

Added in version 244.

### ConditionPathIsReadWrite=

`ConditionPathIsReadWrite=` is similar to
`ConditionPathExists=` but verifies that the underlying file system is readable
and writable (i.e. not mounted read-only).

Added in version 244.

### ConditionPathIsEncrypted=

`ConditionPathIsEncrypted=` is similar to
`ConditionPathExists=` but verifies that the underlying file system's backing
block device is encrypted using dm-crypt/LUKS. Note that this check does not cover ext4
per-directory encryption, and only detects block level encryption. Moreover, if the specified path
resides on a file system on top of a loopback block device, only encryption above the loopback device is
detected. It is not detected whether the file system backing the loopback block device is encrypted.

Added in version 246.

### ConditionDirectoryNotEmpty=

`ConditionDirectoryNotEmpty=` is similar to
`ConditionPathExists=` but verifies that a certain path exists and is a non-empty
directory.

Added in version 244.

### ConditionFileNotEmpty=

`ConditionFileNotEmpty=` is similar to
`ConditionPathExists=` but verifies that a certain path exists and refers to a
regular file with a non-zero size.

Added in version 244.

### ConditionFileIsExecutable=

`ConditionFileIsExecutable=` is similar to
`ConditionPathExists=` but verifies that a certain path exists, is a regular file,
and marked executable.

Added in version 244.

### ConditionUser=

`ConditionUser=` takes a numeric " `UID`", a UNIX
user name, or the special value " `@system`". This condition may be used to check
whether the service manager is running as the given user. The special value
" `@system`" can be used to check if the user id is within the system user
range. This option is not useful for system services, as the system manager exclusively runs as the
root user, and thus the test result is constant.

Added in version 244.

### ConditionGroup=

`ConditionGroup=` is similar to `ConditionUser=`
but verifies that the service manager's real or effective group, or any of its auxiliary groups,
match the specified group or GID. This setting does not support the special value
" `@system`".

Added in version 244.

### ConditionControlGroupController=

Check whether given cgroup controllers (e.g. " `cpu`") are available
for use on the system or whether the legacy v1 cgroup or the modern v2 cgroup hierarchy is used.


Multiple controllers may be passed with a space separating them; in this case, the condition
will only pass if all listed controllers are available for use. Controllers unknown to systemd are
ignored. Valid controllers are " `cpu`", " `io`",
" `memory`", and " `pids`". Even if available in the kernel, a
particular controller may not be available if it was disabled on the kernel command line with
`cgroup_disable=controller`.

Alternatively, two special strings " `v1`" and " `v2`" may be
specified (without any controller names). " `v2`" will pass if the unified v2 cgroup
hierarchy is used, and " `v1`" will pass if the legacy v1 hierarchy or the hybrid
hierarchy are used. Note that legacy or hybrid hierarchies have been deprecated. See
[systemd(1)](systemd.html#) for
more information.

Added in version 244.

### ConditionMemory=

Verify that the specified amount of system memory is available to the current
system. Takes a memory size in bytes as argument, optionally prefixed with a comparison operator
" `<`", " `<=`", " `=`" (or " `==`"),
" `!=`" (or " `<>`"), " `>=`",
" `>`". On bare-metal systems compares the amount of physical memory in the system
with the specified size, adhering to the specified comparison operator. In containers compares the
amount of memory assigned to the container instead.

Added in version 244.

### ConditionCPUs=

Verify that the specified number of CPUs is available to the current system. Takes
a number of CPUs as argument, optionally prefixed with a comparison operator
" `<`", " `<=`", " `=`" (or " `==`"),
" `!=`" (or " `<>`"), " `>=`",
" `>`". Compares the number of CPUs in the CPU affinity mask configured of the
service manager itself with the specified number, adhering to the specified comparison operator. On
physical systems the number of CPUs in the affinity mask of the service manager usually matches the
number of physical CPUs, but in special and virtual environments might differ. In particular, in
containers the affinity mask usually matches the number of CPUs assigned to the container and not
the physically available ones.

Added in version 244.

### ConditionCPUFeature=

Verify that a given CPU feature is available via the " `CPUID`"
instruction. This condition only does something on i386 and x86-64 processors. On other
processors it is assumed that the CPU does not support the given feature. It checks the leaves
" `1`", " `7`", " `0x80000001`", and
" `0x80000007`". Valid values are:
" `fpu`",
" `vme`",
" `de`",
" `pse`",
" `tsc`",
" `msr`",
" `pae`",
" `mce`",
" `cx8`",
" `apic`",
" `sep`",
" `mtrr`",
" `pge`",
" `mca`",
" `cmov`",
" `pat`",
" `pse36`",
" `clflush`",
" `mmx`",
" `fxsr`",
" `sse`",
" `sse2`",
" `ht`",
" `pni`",
" `pclmul`",
" `monitor`",
" `ssse3`",
" `fma3`",
" `cx16`",
" `sse4_1`",
" `sse4_2`",
" `movbe`",
" `popcnt`",
" `aes`",
" `xsave`",
" `osxsave`",
" `avx`",
" `f16c`",
" `rdrand`",
" `bmi1`",
" `avx2`",
" `bmi2`",
" `rdseed`",
" `adx`",
" `sha_ni`",
" `syscall`",
" `rdtscp`",
" `lm`",
" `lahf_lm`",
" `abm`",
" `constant_tsc`".

Added in version 248.

### ConditionOSRelease=

Verify that a specific " `key=value`" pair is set in the host's
[os-release(5)](os-release.html#).

Other than exact string matching (with " `=`" and " `!=`"),
relative comparisons are supported for versioned parameters (e.g. " `VERSION_ID`";
with " `<`", " `<=`", " `==`",
" `<>`", " `>=`", " `>`"), and shell-style
wildcard comparisons (" `*`", " `?`", " `[]`") are
supported with the " `$=`" (match) and " `!$=`" (non-match).

If the given key is not found in the file, the match is done against an empty value.

Added in version 249.

### ConditionMemoryPressure=

Verify that the overall system (memory, CPU or IO) pressure is below or equal to a threshold.
This setting takes a threshold value as argument. It can be specified as a simple percentage value,
suffixed with " `%`", in which case the pressure will be measured as an average over the last
five minutes before the attempt to start the unit is performed.
Alternatively, the average timespan can also be specified using " `/`" as a separator, for
example: " `10%/1min`". The supported timespans match what the kernel provides, and are
limited to " `10sec`", " `1min`" and " `5min`". The
" `full`" PSI will be checked first, and if not found " `some`" will be
checked. For more details, see the documentation on [PSI (Pressure Stall Information)](https://docs.kernel.org/accounting/psi.html).

Optionally, the threshold value can be prefixed with the slice unit under which the pressure will be checked,
followed by a " `:`". If the slice unit is not specified, the overall system pressure will be measured,
instead of a particular cgroup's.

Added in version 250.

### ConditionCPUPressure=

Verify that the overall system (memory, CPU or IO) pressure is below or equal to a threshold.
This setting takes a threshold value as argument. It can be specified as a simple percentage value,
suffixed with " `%`", in which case the pressure will be measured as an average over the last
five minutes before the attempt to start the unit is performed.
Alternatively, the average timespan can also be specified using " `/`" as a separator, for
example: " `10%/1min`". The supported timespans match what the kernel provides, and are
limited to " `10sec`", " `1min`" and " `5min`". The
" `full`" PSI will be checked first, and if not found " `some`" will be
checked. For more details, see the documentation on [PSI (Pressure Stall Information)](https://docs.kernel.org/accounting/psi.html).

Optionally, the threshold value can be prefixed with the slice unit under which the pressure will be checked,
followed by a " `:`". If the slice unit is not specified, the overall system pressure will be measured,
instead of a particular cgroup's.

Added in version 250.

### ConditionIOPressure=

Verify that the overall system (memory, CPU or IO) pressure is below or equal to a threshold.
This setting takes a threshold value as argument. It can be specified as a simple percentage value,
suffixed with " `%`", in which case the pressure will be measured as an average over the last
five minutes before the attempt to start the unit is performed.
Alternatively, the average timespan can also be specified using " `/`" as a separator, for
example: " `10%/1min`". The supported timespans match what the kernel provides, and are
limited to " `10sec`", " `1min`" and " `5min`". The
" `full`" PSI will be checked first, and if not found " `some`" will be
checked. For more details, see the documentation on [PSI (Pressure Stall Information)](https://docs.kernel.org/accounting/psi.html).

Optionally, the threshold value can be prefixed with the slice unit under which the pressure will be checked,
followed by a " `:`". If the slice unit is not specified, the overall system pressure will be measured,
instead of a particular cgroup's.

Added in version 250.

### ConditionKernelModuleLoaded=

Test whether the specified kernel module has been loaded and is already fully
initialized.

Added in version 258.

### AssertArchitecture=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertVirtualization=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertHost=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertKernelCommandLine=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertKernelVersion=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertVersion=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertCredential=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertEnvironment=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertSecurity=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertCapability=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertACPower=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertNeedsUpdate=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertFirstBoot=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertPathExists=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertPathExistsGlob=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertPathIsDirectory=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertPathIsSymbolicLink=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertPathIsMountPoint=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertPathIsReadWrite=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertPathIsEncrypted=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertDirectoryNotEmpty=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertFileNotEmpty=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertFileIsExecutable=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertUser=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertGroup=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertControlGroupController=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertMemory=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertCPUs=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertCPUFeature=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertOSRelease=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertMemoryPressure=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertCPUPressure=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertIOPressure=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

### AssertKernelModuleLoaded=

Similar to the `ConditionArchitecture=`,
`ConditionVirtualization=`, …, condition settings described above, these settings
add assertion checks to the start-up of the unit. However, unlike the conditions settings, any
assertion setting that is not met results in failure of the start job (which means this is logged
loudly). Note that hitting a configured assertion does not cause the unit to enter the
" `failed`" state (or in fact result in any state change of the unit), it affects
only the job queued for it. Use assertion expressions for units that cannot operate when specific
requirements are not met, and when this is something the administrator or user should look
into.

Added in version 218.

