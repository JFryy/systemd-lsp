# Device Section

The `[Device]` section is used for device units, which represent hardware devices and are automatically created by systemd based on udev information.

## Overview

Device units encapsulate kernel devices as exposed in the sysfs/udev device tree. They are automatically created by systemd for all kernel devices that are tagged with "systemd" in udev (by default all block and network devices).

## Key Characteristics

- **No Device-Specific Configuration**: Device units have no device-specific directives
- **Automatic Creation**: Device units are created automatically by systemd, not manually configured
- **Dynamic Management**: They are dynamically created and removed based on hardware presence
- **Dependency Integration**: Other units can depend on device units for hardware availability

## Configuration Sections

Device units only use the generic configuration sections:

### [Unit] Section
- Contains standard unit configuration options
- Used for dependencies, descriptions, and conditions
- See systemd.unit(5) for available directives

### [Install] Section  
- Contains installation information for the unit
- Rarely used with device units since they are automatically managed
- See systemd.unit(5) for available directives

## Device Unit Naming

Device units are named after the `/sys/` and `/dev/` paths they control:
- `/dev/sda1` becomes `dev-sda1.device`
- `/dev/ttyS0` becomes `dev-ttyS0.device`
- Network interfaces like `eth0` become `sys-subsystem-net-devices-eth0.device`

## udev Integration

Device units can be configured through udev properties:

### SYSTEMD_WANTS=
- Adds dependencies of type `Wants=` from the device unit to specified units
- Used to start services when specific hardware is detected

### SYSTEMD_USER_WANTS=
- Similar to SYSTEMD_WANTS= but for user service manager instances

## Usage in Dependencies

Other units commonly depend on device units:

```ini
[Unit]
Description=My Service
After=dev-sda1.device
Requires=dev-sda1.device

[Service]
ExecStart=/usr/bin/my-service
```

## Common Device Unit Examples

- **Block Devices**: `dev-sda1.device`, `dev-nvme0n1p1.device`
- **Network Devices**: `sys-subsystem-net-devices-eth0.device`
- **Serial Devices**: `dev-ttyS0.device`, `dev-ttyUSB0.device`

## Important Notes

- Device units cannot be started or stopped manually
- They are automatically activated when the hardware is present
- They are automatically deactivated when the hardware is removed
- No device unit files should be created manually
- If systemd-udevd.service is not running, no device units will be available

## Related

- systemd.device(5) - Full manual page
- systemd.unit(5) - General unit configuration  
- udev(7) - Device manager for the Linux kernel
- systemd-udevd.service(8) - Device event managing daemon