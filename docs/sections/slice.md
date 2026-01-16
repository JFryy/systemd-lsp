# Slice Section

The `[Slice]` section defines configuration for slice units, which are used for hierarchical resource management through Linux Control Groups (cgroups).

## Overview

Slice units manage system resources by organizing processes into a hierarchical tree structure. They provide a way to apply resource limits and policies to groups of processes, enabling fine-grained control over CPU, memory, I/O, and other system resources.

## Hierarchical Organization

Slices are organized in a tree structure:
- **Root Slice**: `-.slice` (the root of all slices)
- **System Slices**: `system.slice` (system services), `user.slice` (user sessions)
- **Custom Slices**: `foo.slice`, `foo-bar.slice` (custom hierarchies)

Slice names encode their position in the tree using dash-separated components:
- `foo-bar.slice` is located within `foo.slice`
- `foo.slice` is located within the root slice `-.slice`

## Resource Control Directives

The `[Slice]` section supports comprehensive resource control through systemd.resource-control(5) directives:

### Memory Management
- **MemoryAccounting=**: Enable/disable memory accounting (yes/no)
- **MemoryLimit=**: Set memory usage limit (bytes, K, M, G, T)
- **MemoryHigh=**: Set high memory usage threshold for throttling
- **MemoryMax=**: Set maximum memory usage (hard limit)
- **MemorySwapMax=**: Set maximum swap usage

### CPU Management
- **CPUAccounting=**: Enable/disable CPU accounting (yes/no)
- **CPUQuota=**: Set CPU time quota as percentage (e.g., 200% for 2 CPUs)
- **CPUShares=**: Set relative CPU scheduling weight (1-262144)
- **CPUWeight=**: Set CPU scheduling weight (1-10000, modern replacement for CPUShares)

### Task Management
- **TasksAccounting=**: Enable/disable task accounting (yes/no)
- **TasksMax=**: Set maximum number of tasks (processes/threads)

### I/O Management
- **IOAccounting=**: Enable/disable I/O accounting (yes/no)
- **IOWeight=**: Set I/O scheduling weight (1-10000)
- **IODeviceWeight=**: Set per-device I/O weight
- **IOReadBandwidthMax=**: Set maximum read bandwidth per device
- **IOWriteBandwidthMax=**: Set maximum write bandwidth per device
- **IOReadIOPSMax=**: Set maximum read IOPS per device
- **IOWriteIOPSMax=**: Set maximum write IOPS per device

### Block I/O Management (Legacy)
- **BlockIOAccounting=**: Enable/disable block I/O accounting (yes/no)
- **BlockIOWeight=**: Set block I/O scheduling weight (10-1000)
- **BlockIODeviceWeight=**: Set per-device block I/O weight
- **BlockIOReadBandwidth=**: Set per-device read bandwidth limit
- **BlockIOWriteBandwidth=**: Set per-device write bandwidth limit

## Usage Patterns

### Resource Isolation
```ini
[Unit]
Description=Database Services Slice

[Slice]
MemoryAccounting=yes
MemoryMax=4G
CPUAccounting=yes
CPUWeight=800
TasksMax=500
```

### Service Organization
```ini
[Unit]
Description=Web Services Slice

[Slice]
MemoryAccounting=yes
MemoryHigh=2G
CPUAccounting=yes
CPUQuota=150%
IOAccounting=yes
IOWeight=100
```

## Default Slices

systemd creates several default slices:

- **-.slice**: Root slice containing all other slices
- **system.slice**: Contains all system services
- **user.slice**: Contains all user sessions
- **machine.slice**: Contains all virtual machines and containers

## Assignment to Slices

Services can be assigned to slices using the `Slice=` directive in their `[Service]` section:

```ini
[Service]
Slice=database.slice
ExecStart=/usr/bin/mysqld
```

## Monitoring and Control

Slice resource usage can be monitored using:
- `systemctl status slice-name.slice`
- `systemd-cgtop` for real-time resource usage
- `systemd-cgls` for cgroup hierarchy visualization

## Best Practices

1. **Enable Accounting**: Always enable accounting for resources you want to control
2. **Hierarchical Design**: Use meaningful slice hierarchies (e.g., `web.slice`, `web-frontend.slice`)
3. **Resource Limits**: Set appropriate limits to prevent resource exhaustion
4. **Monitoring**: Regularly monitor slice resource usage
5. **Testing**: Test resource limits under load conditions

## Related

- systemd.slice(5) - Full manual page
- systemd.resource-control(5) - Resource control directives
- systemd.service(5) - Service unit configuration
- systemd-cgtop(1) - Resource usage monitoring
- systemd-cgls(1) - Control group hierarchy viewer