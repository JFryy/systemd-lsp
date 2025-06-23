# [Service] Section

The `[Service]` section encodes information about a process controlled and supervised by systemd. This section is used for units of type `service`. Many options that may be used in this section are shared with other unit types and are documented in [systemd.exec(5)](https://www.freedesktop.org/software/systemd/man/systemd.exec.html), [systemd.kill(5)](https://www.freedesktop.org/software/systemd/man/systemd.kill.html), and [systemd.resource-control(5)](https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html).

*Based on [systemd.service(5)](https://www.freedesktop.org/software/systemd/man/systemd.service.html) official documentation.*

## Service Types

### Type=
Configures the unit type. The available options are:

- **simple** (default): The main process of the service is specified in the command line. systemd will consider the service started immediately after the main service process has been forked off.
- **exec**: Similar to simple, but systemd will consider the service started up only after the main service process has been executed successfully. This is the recommended type for long-running services.
- **forking**: The service forks a child process, exiting the parent process almost immediately. This is the traditional daemon behavior. The original process is expected to exit, while the child continues to run as the main service process.
- **oneshot**: This is useful for scripts that do a single job and then exit. The service is considered active even after the main process exits. You may want to set `RemainAfterExit=yes` as well.
- **dbus**: The service is considered ready when the specified `BusName=` appears on the D-Bus system message bus.
- **notify**: The service will issue a notification when it has finished starting up using [sd_notify(3)](https://www.freedesktop.org/software/systemd/man/sd_notify.html) or an equivalent call.
- **notify-reload**: Similar to notify, but supports reloading via `RELOADING=1` and `READY=1` notifications.
- **idle**: The service will not be run until all active jobs are dispatched. This avoids interleaving of shell output of services with the status output on the console.

## Execution Directives

### ExecStart=
Commands with their arguments that are executed when this service is started. The value is split into zero or more command lines according to the rules described below.

### ExecStartPre=
Additional commands that are executed before the command in ExecStart=. Syntax is the same as for ExecStart=, except that multiple command lines are allowed and the commands are executed one after the other, serially.

### ExecStartPost=
Additional commands that are executed after the command in ExecStart=. Syntax is the same as for ExecStart=.

### ExecStop=
Commands to execute to stop the service started via ExecStart=. If not specified, the process will be terminated by sending SIGTERM.

### ExecStopPost=
Additional commands that are executed after the service is stopped. Syntax is the same as for ExecStart=.

### ExecReload=
Commands to execute to trigger a configuration reload in the service.

### ExecCondition=
Commands executed before ExecStart= to check if the service should be started. If any command fails, the service is not started, but this is not considered a failure.

### ExecStopPre=
Additional commands that are executed before the service is stopped. These commands are executed before ExecStop=.

### RestartSec=
Configures the time to sleep before restarting a service (as configured with Restart=). Takes a unit-less value in seconds, or a time span value such as "5min 20s".

### RestartSteps=
Configure the number of steps to take to increase the restart delay between RestartSec= and RestartMaxDelaySec=.

### RestartMaxDelaySec=
Configure the longest time to sleep before restarting a service as the number of restart steps increases.

### ExitType=
Specifies how to determine the exit status of the service. Takes one of main, cgroup.

### RemainAfterExit=
Takes a boolean value that specifies whether the service shall be considered active even when all its processes exited.

### GuessMainPID=
Takes a boolean value that specifies whether systemd should try to guess the main PID of a service if it cannot be determined reliably.

### PIDFile=
Takes a path referring to the PID file of the service. Usage of this option is recommended for services where Type=forking is used.

### BusName=
Takes a D-Bus bus name that this service is reachable as. This option is required for services where Type=dbus is used.

## Process Control

### Restart=
Configures whether the service shall be restarted when the service process exits, is killed, or a timeout is reached:

- **no**: The service will not be restarted (default)
- **always**: The service will be restarted regardless of whether it exited cleanly or not
- **on-success**: The service will be restarted only if the service process exits cleanly
- **on-failure**: The service will be restarted if the service process exits with a non-zero exit code, is terminated by a signal, when an operation times out, or when the configured watchdog timeout is triggered
- **on-abnormal**: The service will be restarted if the process exits with a signal or when a timeout occurs
- **on-abort**: The service will be restarted only if the service process exits due to an uncaught signal not specified as a clean exit status
- **on-watchdog**: The service will be restarted only if the watchdog timeout for the service expires

### User=
Set the Unix user or UID that the processes are executed as. Takes a single user name or a numeric user ID as argument.

### Group=
Set the Unix group or GID that the processes are executed as. Takes a single group name or a numeric group ID as argument.

### WorkingDirectory=
Set the working directory for executed processes. Takes a directory path as argument.

### Environment=
Set environment variables for executed processes. Takes a space-separated list of variable assignments.

### EnvironmentFile=
Similar to Environment= but reads the environment variables from a text file.

## Security Options

### PrivateTmp=
Takes a boolean argument. If true, sets up a new file system namespace for the executed processes and mounts private /tmp and /var/tmp directories inside it.

### NoNewPrivileges=
Takes a boolean argument. If true, ensures that the service process and all its children can never gain new privileges.

### ProtectSystem=
Takes a boolean argument or "strict" or "full". If true, mounts the /usr and /boot directories read-only for processes invoked by this unit.

### ProtectHome=
Takes a boolean argument or "read-only". If true, the directories /home, /root and /run/user are made inaccessible and empty for processes invoked by this unit.

## Example

```ini
[Service]
Type=simple
ExecStart=/usr/bin/my-service --config /etc/my-service.conf
ExecStop=/bin/kill -TERM $MAINPID
Restart=always
RestartSec=5
User=my-service
Group=my-service
WorkingDirectory=/var/lib/my-service
Environment=NODE_ENV=production
PrivateTmp=yes
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
```

## See Also

- [systemd.service(5)](https://www.freedesktop.org/software/systemd/man/systemd.service.html) - Service unit configuration
- [systemd.exec(5)](https://www.freedesktop.org/software/systemd/man/systemd.exec.html) - Execution environment configuration
- [systemd.kill(5)](https://www.freedesktop.org/software/systemd/man/systemd.kill.html) - Process killing configuration
- [systemd.resource-control(5)](https://www.freedesktop.org/software/systemd/man/systemd.resource-control.html) - Resource control configuration
- [systemd(1)](https://www.freedesktop.org/software/systemd/man/systemd.html) - systemd system and service manager
- [systemctl(1)](https://www.freedesktop.org/software/systemd/man/systemctl.html) - Control systemd services and units