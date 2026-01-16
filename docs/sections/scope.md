# Scope Section

The `[Scope]` section defines configuration for scope units, which are used to group and manage externally created processes for resource control and organization.

## Overview

Scope units manage sets of system processes that were not started by systemd itself. Unlike service units, scope units do not fork processes on their own but rather manage externally created processes. They are primarily used for organizing and controlling resources of processes started by other means.

## Key Characteristics

- **External Process Management**: Manages processes created outside of systemd
- **Resource Control**: Applies resource limits and policies to grouped processes  
- **Programmatic Creation**: Created programmatically via D-Bus, not through unit files
- **Transient Units**: Typically created as transient units during runtime
- **Process Grouping**: Groups related processes for unified management

## Scope-Specific Directives

### RuntimeMaxSec=
- **Type**: Time span or "infinity"
- **Default**: infinity (no limit)
- **Description**: Configures a maximum runtime for the scope. If the scope has been active longer than specified, it is terminated and put into a failure state.

### RuntimeRandomizedExtraSec=
- **Type**: Time span in seconds
- **Default**: 0 (disabled)
- **Description**: Adds a randomized extra time to RuntimeMaxSec= to prevent thundering herd problems. The actual runtime will be between RuntimeMaxSec= and RuntimeMaxSec= + RuntimeRandomizedExtraSec=.

## Resource Control Directives

Scope units inherit all resource control directives from systemd.resource-control(5):

### Memory Management
- **MemoryAccounting=**: Enable/disable memory accounting
- **MemoryLimit=**: Set memory usage limit (deprecated, use MemoryMax=)
- **MemoryHigh=**: Set high memory usage threshold for throttling
- **MemoryMax=**: Set maximum memory usage (hard limit)

### CPU Management
- **CPUAccounting=**: Enable/disable CPU accounting
- **CPUQuota=**: Set CPU time quota as percentage
- **CPUWeight=**: Set CPU scheduling weight (1-10000)

### Task Management
- **TasksAccounting=**: Enable/disable task accounting
- **TasksMax=**: Set maximum number of tasks (processes/threads)

### I/O Management
- **IOAccounting=**: Enable/disable I/O accounting
- **IOWeight=**: Set I/O scheduling weight (1-10000)

### Block I/O Management (Legacy)
- **BlockIOAccounting=**: Enable/disable block I/O accounting
- **BlockIOWeight=**: Set block I/O scheduling weight (10-1000)

## Process Control Directives

Scope units inherit process control directives from systemd.kill(5):

### Kill Management
- **KillMode=**: How to kill processes (control-group, mixed, process, none)
- **KillSignal=**: Signal to send to main process (default: SIGTERM)
- **RestartKillSignal=**: Signal to send during restart (default: SIGTERM)
- **SendSIGKILL=**: Whether to send SIGKILL after timeout (yes/no)
- **SendSIGHUP=**: Whether to send SIGHUP to remaining processes (yes/no)
- **WatchdogSignal=**: Signal to send for watchdog timeout (default: SIGABRT)

## Common Use Cases

### User Session Management
```ini
[Unit]
Description=User Session Scope

[Scope]
RuntimeMaxSec=8h
MemoryAccounting=yes
MemoryMax=2G
CPUAccounting=yes
CPUWeight=100
```

### Application Process Groups
```ini
[Unit]
Description=Database Process Group

[Scope]
MemoryAccounting=yes
MemoryMax=4G
CPUAccounting=yes
CPUQuota=200%
TasksMax=1000
KillMode=mixed
```

## Creating Scope Units

Scope units are typically created programmatically through D-Bus:

```bash
# Create a transient scope unit
systemd-run --scope --unit=my-scope.scope \
    --property=MemoryMax=1G \
    --property=CPUWeight=500 \
    my-command
```

## Relationship with Other Units

### vs Service Units
- **Service**: systemd starts and manages the processes
- **Scope**: External processes are assigned to the scope for management

### vs Slice Units
- **Slice**: Hierarchical resource management container
- **Scope**: Manages specific process groups, can be assigned to slices

### Integration with Slices
```ini
[Scope]
Slice=user-applications.slice
MemoryMax=1G
```

## Process Assignment

Processes can be assigned to scope units:
- During scope creation via D-Bus
- By moving PIDs into the scope's cgroup
- Through systemd-run with existing processes

## Monitoring and Control

Monitor scope units using:
- `systemctl status scope-name.scope`
- `systemd-cgtop` for resource usage
- `systemd-cgls` for process hierarchy

## Best Practices

1. **Resource Limits**: Always set appropriate resource limits
2. **Accounting**: Enable accounting for resources you want to monitor
3. **Kill Modes**: Choose appropriate kill modes for clean shutdown
4. **Naming**: Use descriptive names for scope units
5. **Monitoring**: Regularly monitor scope resource usage

## Related

- systemd.scope(5) - Full manual page
- systemd.resource-control(5) - Resource control directives
- systemd.kill(5) - Process killing directives
- systemd.slice(5) - Slice unit configuration
- systemd-run(1) - Creating transient units