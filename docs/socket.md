# [Socket] Section

The `[Socket]` section contains socket-specific settings. Socket units are used to implement on-demand starting of services, socket-based activation, and systemd's socket-based activation and socket passing features.

## Socket Types

### ListenStream=
Specifies an address for a stream socket (TCP). Takes an address specification in the format described below.

### ListenDatagram=
Specifies an address for a datagram socket (UDP). Takes an address specification.

### ListenSequentialPacket=
Specifies an address for a sequential packet socket. Takes an address specification.

### ListenFIFO=
Specifies a file system FIFO (named pipe) to listen on. Takes an absolute file system path as argument.

### ListenSpecial=
Specifies a special file in the file system to listen on. Takes an absolute file system path as argument.

### ListenNetlink=
Specifies a Netlink family to create a socket for to listen on. Takes a Netlink family name as argument.

### ListenMessageQueue=
Specifies a POSIX message queue to listen on. Takes a message queue name as argument.

### ListenUSBFunction=
Specifies a USB FunctionFS to listen on. Takes a path to the FunctionFS mount point as argument.

## Address Specifications

### Network Addresses
- `*:80` - Listen on all interfaces, port 80
- `127.0.0.1:8080` - Listen on localhost, port 8080
- `[::]:443` - Listen on all IPv6 interfaces, port 443
- `192.168.1.100:22` - Listen on specific IP, port 22

### Unix Domain Sockets
- `/run/my-service.sock` - Unix domain socket
- `@abstract-socket` - Abstract Unix domain socket

### Named Ports
- `http` - Port 80 (HTTP)
- `https` - Port 443 (HTTPS)
- `ssh` - Port 22 (SSH)
- `ftp` - Port 21 (FTP)

## Socket Options

### SocketMode=
If listening on a file system socket or FIFO, this option specifies the file system access mode used when creating the file node. Takes a numeric mode value.

### SocketUser=, SocketGroup=
If listening on a file system socket or FIFO, the user and group to make the socket accessible to. Takes a user/group name or numeric ID.

### Service=
Specifies the service unit name to activate when this socket receives a connection. Defaults to the service unit with the same name as the socket unit.

### Accept=
Takes a boolean argument. If yes, a service instance is spawned for each incoming connection and only the connection socket is passed to it. If no (default), all listening sockets themselves are passed to the started service unit.

### MaxConnections=
The maximum number of connections to accept simultaneously. Takes a positive integer argument. Defaults to 64.

### MaxConnectionsPerSource=
The maximum number of connections for a service per source IP address. Takes a positive integer argument.

### KeepAlive=
Takes a boolean argument. If true, the TCP/IP stack will send a keep alive message after 2 hours of inactivity. This controls the SO_KEEPALIVE socket option.

### KeepAliveTimeSec=
Configures how long to wait before starting to send TCP keep-alive probes.

### KeepAliveIntervalSec=
Configures the interval between individual TCP keep-alive probes.

### KeepAliveProbes=
Configures how many TCP keep-alive probes to send before giving up and killing the connection.

### DeferAccept=
Takes a boolean or time span argument. If true, the TCP_DEFER_ACCEPT socket option is enabled. If set to a time span, TCP_DEFER_ACCEPT is set to the specified time in seconds.

### NoDelay=
Takes a boolean argument. If true, the TCP_NODELAY option is set. This disables Nagle's algorithm.

### Priority=
Takes an integer argument and configures the priority for all traffic sent from this socket.

### ReceiveBuffer=, SendBuffer=
Configures the receive/send buffer size in bytes. Takes an integer value.

### IPTOS=
Takes an integer value and configures the IP_TOS socket option.

### IPTTL=
Takes an integer value and configures the IP_TTL socket option.

### Mark=
Takes an integer value and configures the SO_MARK socket option.

### ReusePort=
Takes a boolean value and configures the SO_REUSEPORT socket option.

### SmackLabel=, SmackLabelIPIn=, SmackLabelIPOut=
Configures SMAC labels. Takes a string value.

### SELinuxContextFromNet=
Takes a boolean argument. If true, systemd will attempt to figure out the SELinux label for the instantiated service from the information provided by the peer over the network.

### PipeSize=
Takes a size in bytes and configures the pipe buffer size for FIFOs.

### MessageQueueMaxMessages=, MessageQueueMessageSize=
Configures the maximum number of messages and message size for POSIX message queues.

### FreeBind=
Takes a boolean argument. If true, the IP_FREEBIND socket option is enabled.

### Transparent=
Takes a boolean argument. If true, the IP_TRANSPARENT socket option is enabled.

### Broadcast=
Takes a boolean argument and controls the SO_BROADCAST socket option.

### PassCredentials=
Takes a boolean argument and controls the SO_PASSCRED socket option.

### PassSecurity=
Takes a boolean argument and controls the SO_PASSSEC socket option.

### TCPCongestion=
Takes a string value and controls the TCP congestion algorithm used by the socket.

### ExecStartPre=, ExecStartPost=, ExecStopPre=, ExecStopPost=
Commands to execute before/after starting/stopping the socket.

### TimeoutSec=
Configures the time to wait for the commands specified in ExecStartPre=, ExecStartPost=, ExecStopPre= and ExecStopPost= to finish.

### FileDescriptorName=
Assigns a name to all file descriptors this socket unit encapsulates.

### TriggerLimitIntervalSec=, TriggerLimitBurst=
Configures a limit on how often this socket unit may be activated within a specific time interval.

### Symlinks=
Takes a list of file system paths. The specified paths will be created as symlinks to the AF_UNIX socket path or FIFO path of this socket unit.

### DirectoryMode=
If listening on a file system socket or FIFO, the parent directories are created if needed. This option specifies the file system access mode used when creating these directories.

## Examples

### HTTP Server Socket
```ini
[Socket]
ListenStream=80
Service=web-server.service

[Install]
WantedBy=sockets.target
```

### SSH-like Service
```ini
[Socket]
ListenStream=2222
Accept=yes
MaxConnections=20
Service=my-ssh.service

[Install]
WantedBy=sockets.target
```

### Unix Domain Socket
```ini
[Socket]
ListenStream=/run/my-service.sock
SocketMode=0660
SocketUser=my-service
SocketGroup=my-service
Service=my-service.service

[Install]
WantedBy=sockets.target
```

### UDP Service
```ini
[Socket]
ListenDatagram=*:1234
Service=udp-handler.service

[Install]
WantedBy=sockets.target
```

### Multiple Addresses
```ini
[Socket]
ListenStream=80
ListenStream=443
ListenStream=/run/web.sock
KeepAlive=true
NoDelay=true
Service=web-server.service

[Install]
WantedBy=sockets.target
```

### High-Performance Socket
```ini
[Socket]
ListenStream=*:8080
Accept=no
MaxConnections=1000
ReceiveBuffer=2M
SendBuffer=2M
NoDelay=true
ReusePort=true
Service=high-perf.service

[Install]
WantedBy=sockets.target
```

## Common Patterns

### Web Server
For HTTP/HTTPS services:
```ini
[Socket]
ListenStream=80
ListenStream=443
NoDelay=true
KeepAlive=true
```

### Database Services
For database-like services:
```ini
[Socket]
ListenStream=5432
Accept=no
MaxConnections=100
KeepAlive=true
```

### Secure Services
For services requiring access control:
```ini
[Socket]
ListenStream=/run/secure.sock
SocketMode=0600
SocketUser=daemon
SocketGroup=daemon
```

## Management Commands

### List Socket Units
```bash
systemctl list-units --type=socket
```

### Show Socket Status
```bash
systemctl status my-service.socket
```

### Start Socket
```bash
systemctl start my-service.socket
```

### Test Socket Connection
```bash
# For stream sockets
nc localhost 8080

# For Unix domain sockets  
nc -U /run/my-service.sock
```

## See Also

- systemd.socket(5)
- systemd.exec(5)
- systemctl(1)
- systemd(1)
- socket(7)
- unix(7)
- tcp(7)
- udp(7)