# [Install] Section

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

### Alias=

A space-separated list of additional names this unit shall be installed under. The names listed
here must have the same suffix (i.e. type) as the unit filename. This option may be specified more than once,
in which case all listed names are used. At installation time, **systemctl enable** will create
symlinks from these names to the unit filename. Note that not all unit types support such alias names, and this
setting is not supported for them. Specifically, mount, slice, swap, and automount units do not support
aliasing.

Added in version 201.

### WantedBy=

This option may be used more than once, or a space-separated list of unit names may
be given. A symbolic link is created in the `.wants/`, `.requires/`,
or `.upholds/` directory of each of the listed units when this unit is installed
by **systemctl enable**. This has the effect of a dependency of type
`Wants=`, `Requires=`, or `Upholds=` being added
from the listed unit to the current unit. See the description of the mentioned dependency types
in the \[Unit\] section for details.

In case of template units listing non template units, the listing unit must have
`DefaultInstance=` set, or **systemctl enable** must be called with
an instance name. The instance (default or specified) will be added to the
`.wants/`, `.requires/`, or `.upholds/`
list of the listed unit. For example, **WantedBy=getty.target** in a service
`getty@.service` will result in **systemctl enable getty@tty2.service**
creating a `getty.target.wants/getty@tty2.service` link to
`getty@.service`. This also applies to listing specific instances of templated
units: this specific instance will gain the dependency. A template unit may also list a template
unit, in which case a generic dependency will be added where each instance of the listing unit will
have a dependency on an instance of the listed template with the same instance value. For example,
**WantedBy=container@.target** in a service `monitor@.service` will
result in **systemctl enable monitor@.service** creating a
`container@.target.wants/monitor@.service` link to
`monitor@.service`, which applies to all instances of
`container@.target`.

Added in version 201.

### RequiredBy=

This option may be used more than once, or a space-separated list of unit names may
be given. A symbolic link is created in the `.wants/`, `.requires/`,
or `.upholds/` directory of each of the listed units when this unit is installed
by **systemctl enable**. This has the effect of a dependency of type
`Wants=`, `Requires=`, or `Upholds=` being added
from the listed unit to the current unit. See the description of the mentioned dependency types
in the \[Unit\] section for details.

In case of template units listing non template units, the listing unit must have
`DefaultInstance=` set, or **systemctl enable** must be called with
an instance name. The instance (default or specified) will be added to the
`.wants/`, `.requires/`, or `.upholds/`
list of the listed unit. For example, **WantedBy=getty.target** in a service
`getty@.service` will result in **systemctl enable getty@tty2.service**
creating a `getty.target.wants/getty@tty2.service` link to
`getty@.service`. This also applies to listing specific instances of templated
units: this specific instance will gain the dependency. A template unit may also list a template
unit, in which case a generic dependency will be added where each instance of the listing unit will
have a dependency on an instance of the listed template with the same instance value. For example,
**WantedBy=container@.target** in a service `monitor@.service` will
result in **systemctl enable monitor@.service** creating a
`container@.target.wants/monitor@.service` link to
`monitor@.service`, which applies to all instances of
`container@.target`.

Added in version 201.

### UpheldBy=

This option may be used more than once, or a space-separated list of unit names may
be given. A symbolic link is created in the `.wants/`, `.requires/`,
or `.upholds/` directory of each of the listed units when this unit is installed
by **systemctl enable**. This has the effect of a dependency of type
`Wants=`, `Requires=`, or `Upholds=` being added
from the listed unit to the current unit. See the description of the mentioned dependency types
in the \[Unit\] section for details.

In case of template units listing non template units, the listing unit must have
`DefaultInstance=` set, or **systemctl enable** must be called with
an instance name. The instance (default or specified) will be added to the
`.wants/`, `.requires/`, or `.upholds/`
list of the listed unit. For example, **WantedBy=getty.target** in a service
`getty@.service` will result in **systemctl enable getty@tty2.service**
creating a `getty.target.wants/getty@tty2.service` link to
`getty@.service`. This also applies to listing specific instances of templated
units: this specific instance will gain the dependency. A template unit may also list a template
unit, in which case a generic dependency will be added where each instance of the listing unit will
have a dependency on an instance of the listed template with the same instance value. For example,
**WantedBy=container@.target** in a service `monitor@.service` will
result in **systemctl enable monitor@.service** creating a
`container@.target.wants/monitor@.service` link to
`monitor@.service`, which applies to all instances of
`container@.target`.

Added in version 201.

### Also=

Additional units to install/deinstall when
this unit is installed/deinstalled. If the user requests
installation/deinstallation of a unit with this option
configured, **systemctl enable** and
**systemctl disable** will automatically
install/uninstall units listed in this option as well.

This option may be used more than once, or a
space-separated list of unit names may be
given.

Added in version 201.

### DefaultInstance=

In template unit files, this specifies for
which instance the unit shall be enabled if the template is
enabled without any explicitly set instance. This option has
no effect in non-template unit files. The specified string
must be usable as instance identifier.

Added in version 215.

