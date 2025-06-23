# [Install] Section

The `[Install]` section contains installation information for the unit. This section is not interpreted by systemd(1) during runtime; it is used by the enable and disable commands of the systemctl(1) tool during installation of a unit.

## Installation Directives

### WantedBy=
A space-separated list of unit names. When this unit is enabled, a symlink is created in the .wants/ directory of the specified units. This is used to specify when this unit should be started.

The most common target is `multi-user.target` for services that should be started in multi-user mode (equivalent to runlevel 3 in SysV init).

### RequiredBy=
Similar to WantedBy=, but creates symlinks in .requires/ directories instead. Units listed here will be required by the specified units when this unit is enabled.

### Alias=
A space-separated list of additional names this unit shall be installed under. The names listed here must have the same suffix (i.e., type) as the unit filename.

### Also=
A space-separated list of unit names. When this unit is enabled/disabled, the units listed here will also be enabled/disabled.

### DefaultInstance=
In template unit files, this specifies for which instance the unit shall be enabled if the template is enabled without any explicitly set instance.

## Common Targets

### multi-user.target
This target is reached when a multi-user system is set up with all essential services running. Most services should be wanted by this target.

### graphical.target
This target is reached when a graphical session is available. GUI applications and display managers should be wanted by this target.

### default.target
This is the default target unit systemd starts at boot-up. Usually an alias for either multi-user.target or graphical.target.

### sysinit.target
This target is reached when basic system initialization is complete.

### basic.target
This target is reached when basic system startup is finished.

### shutdown.target
This target is reached when the system is being shut down.

### reboot.target
This target is reached when the system is being rebooted.

### poweroff.target
This target is reached when the system is being powered off.

## Example

```ini
[Install]
WantedBy=multi-user.target
Alias=my-service.service
Also=my-service-helper.service
```

## Common Patterns

### System Services
Most system services should be wanted by `multi-user.target`:
```ini
[Install]
WantedBy=multi-user.target
```

### User Services
Services that should run in a user session typically use `default.target`:
```ini
[Install]
WantedBy=default.target
```

### Socket-Activated Services
Services that are socket-activated often don't need an [Install] section, as they are started by their corresponding socket unit:
```ini
# Usually no [Install] section needed for socket-activated services
```

### Timer-Activated Services
Services that are run by timers typically only need to be enabled if they should run immediately:
```ini
[Install]
WantedBy=timers.target
```

## Installation Commands

### Enable a Unit
```bash
systemctl enable my-service.service
```
This creates the necessary symlinks according to the [Install] section.

### Disable a Unit
```bash
systemctl disable my-service.service
```
This removes the symlinks created by enable.

### Check if Unit is Enabled
```bash
systemctl is-enabled my-service.service
```

### Enable and Start
```bash
systemctl enable --now my-service.service
```

## See Also

- systemd.unit(5)
- systemctl(1)
- systemd(1)
- systemd.target(5)