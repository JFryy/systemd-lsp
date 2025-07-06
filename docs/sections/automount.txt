# Automount Section

The `[Automount]` section defines configuration for automount units, which are used to automatically mount filesystems when accessed.

## Overview

Automount units supervise file system automount points. They are used to automatically mount filesystems when they are accessed, providing on-demand mounting capabilities.

## Key Directives

### Where=
- **Type**: Absolute path
- **Required**: Yes
- **Description**: Specifies the absolute path of the directory where the automount point should be created. If the directory does not exist, it will be created when the automount point is installed.

### ExtraOptions=
- **Type**: Comma-separated list
- **Required**: No
- **Description**: Additional mount options to use when creating the autofs mountpoint. These options are passed directly to the autofs system.

### DirectoryMode=
- **Type**: Octal notation
- **Default**: 0755
- **Description**: Specifies the file system access mode used when creating the automount directory. The value must be in octal notation (e.g., 0755, 0644).

### TimeoutIdleSec=
- **Type**: Time span
- **Default**: infinity (disabled)
- **Description**: Configures an idle timeout for the automount. Once the mount has been idle for the specified time, systemd will attempt to unmount it. Can be specified as a unitless value in seconds or as a time span (e.g., "5min 20s").

## Usage Notes

- For each automount unit file, a matching mount unit file must exist
- The mount unit is activated when the automount path is accessed
- Automount units must be named after the automount directories they control
- Automount units are useful for network filesystems, removable media, and infrequently used filesystems

## Example

```ini
[Unit]
Description=Automount for /mnt/data

[Automount]
Where=/mnt/data
TimeoutIdleSec=60

[Install]
WantedBy=multi-user.target
```

## Related

- systemd.automount(5) - Full manual page
- systemd.mount(5) - Mount unit configuration
- systemd.unit(5) - General unit configuration