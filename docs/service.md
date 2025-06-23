# [Service] Section

The `[Service]` section contains information about the service and the process it supervises. This section is used for units of Type=service. A number of options that may be used in this section are shared with other unit types. These options are documented in systemd.exec(5), systemd.kill(5) and systemd.resource-control(5).

## Service Types

### Type=
Configures the unit type. The available options are:

- **simple** (default): The main process of the service is specified in the command line. systemd will consider the service started immediately after the main service process has been forked off
- **exec**: Similar to simple, but systemd will consider the service started up only after the main service process has been executed successfully
- **forking**: The service forks a child process, exiting the parent process almost immediately. This tells systemd that the process is still running even though the parent exited
- **oneshot**: This is useful for scripts that do a single job and then exit. You may want to set RemainAfterExit=yes as well
- **dbus**: The service is considered ready when the specified BusName appears on DBus system message bus
- **notify**: The service will issue a notification when it has finished starting up using sd_notify(3) or equivalent call
- **idle**: The service will not be run until all active jobs are dispatched

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

### RestartSec=
Configures the time to sleep before restarting a service (as configured with Restart=). Takes a unit-less value in seconds, or a time span value such as "5min 20s".

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

- systemd.service(5)
- systemd.exec(5)
- systemd.kill(5)
- systemd.resource-control(5)
- systemd(1)
- systemctl(1)