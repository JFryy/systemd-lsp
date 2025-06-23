# [Unit] Section

The `[Unit]` section contains generic information about the unit that is not dependent on the type of unit. Options in this section are shared by all unit types. This section is used to define metadata about the unit (such as dependencies), conditions and assertions that must be met before the unit is started, and various other configuration settings.

## Common Directives

### Description=
A human readable name for the unit. This is used by systemd (and other UIs) as the label for the unit, so this string should identify the unit rather than describe its functionality. "Apache2 Web Server" is a good example. Bad examples are "high-performance light-weight HTTP server" (too generic) or "Apache2" (too specific and doesn't describe the unit type).

### Documentation=
A space-separated list of URIs referencing documentation for this unit or its configuration. Accepted are only URIs of the types "http://", "https://", "file:", "info:", "man:". For more information about the syntax of these URIs, see uri(7).

### Requires=
Configures requirement dependencies on other units. If this unit gets activated, the units listed here will be activated as well. If one of the other units fails to activate, and an ordering dependency After= on the failing unit is set, this unit will not be started. If one of the other units is explicitly stopped, this unit will be stopped as well.

### Wants=
A weaker form of Requires=. Units listed in this option will be started if the configuring unit is. However, if the listed units fail to start or cannot be added to the transaction, this has no impact on the validity of the transaction as a whole. This is the recommended way to hook start-up of one unit to the start-up of another unit.

### Before=
A space-separated list of unit names. Configures ordering dependencies between units. If a unit foo.service contains a setting Before=bar.service and both units are being started, bar.service's start-up is delayed until foo.service has finished starting up.

### After=
The opposite of Before=. If a unit foo.service contains a setting After=bar.service and both units are being started, foo.service's start-up is delayed until bar.service has finished starting up.

### Conflicts=
A space-separated list of unit names. Configures negative requirement dependencies. If a unit has a Conflicts= setting on another unit, starting the former will stop the latter and vice versa.

### Condition...=
Various condition directives that must be satisfied for the unit to be started. Examples include ConditionPathExists=, ConditionFileNotEmpty=, ConditionDirectoryNotEmpty=, etc.

### Assert...=
Various assertion directives that must be satisfied for the unit to be started. Similar to conditions but cause the unit to fail if not met.

## Example

```ini
[Unit]
Description=My Custom Service
Documentation=man:my-service(8)
Requires=network.target
After=network.target
Wants=multi-user.target
Before=multi-user.target
Conflicts=shutdown.target
```

## See Also

- systemd.unit(5)
- systemd.service(5)
- systemd(1)
- systemctl(1)