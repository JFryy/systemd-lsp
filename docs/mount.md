# [Mount] Section

The `[Mount]` section contains mount-specific settings. Mount units are used to control mount points in the system. Mount units must be named after the mount point directories they control. For example, the mount point /home/lennart must be configured in the unit file home-lennart.mount.

## Required Directives

### What=
Takes an absolute path of a device node, file or other resource to mount. This corresponds to the device/source field of the mount(8) command or the first field of /etc/fstab.

### Where=
Takes an absolute path of the mount point. This must match the unit name. This corresponds to the mount point field of the mount(8) command or the second field of /etc/fstab.

## Optional Directives

### Type=
Takes a file system type as used by mount(8). This corresponds to the file system type field of /etc/fstab or the -t parameter of mount(8).

### Options=
Mount options to use when mounting. This corresponds to the options field in /etc/fstab or the -o parameter of mount(8).

### SloppyOptions=
Takes a boolean argument. If true, parsing of the mount options is relaxed, and unknown mount options are tolerated.

### LazyUnmount=
Takes a boolean argument. If true, detach the filesystem from the mount tree before attempting to unmount it (MNT_DETACH).

### ReadWriteOnly=
Takes a boolean argument. If true, remount the mount point read-write after mounting.

### ForceUnmount=
Takes a boolean argument. If true, force an unmount (MNT_FORCE).

### DirectoryMode=
Directories of mount points (and any parent directories) are automatically created if needed. This option specifies the access mode of these directories.

### TimeoutSec=
Configures the time to wait for the mount/unmount command to finish.

## File System Types

### Common File Systems
- `ext4` - Fourth extended filesystem
- `ext3` - Third extended filesystem  
- `ext2` - Second extended filesystem
- `xfs` - XFS filesystem
- `btrfs` - B-tree filesystem
- `vfat` - VFAT filesystem (FAT32)
- `ntfs` - NTFS filesystem
- `exfat` - exFAT filesystem
- `iso9660` - ISO 9660 filesystem (CD-ROM)
- `tmpfs` - Temporary filesystem in RAM
- `proc` - Process information pseudo-filesystem
- `sysfs` - System information pseudo-filesystem
- `devpts` - Device pseudo-terminal filesystem

### Network File Systems
- `nfs` - Network File System
- `nfs4` - Network File System version 4
- `cifs` - Common Internet File System (SMB/CIFS)
- `sshfs` - SSH Filesystem

### Special File Systems
- `bind` - Bind mount (not a real filesystem)
- `overlay` - Overlay filesystem
- `squashfs` - Compressed read-only filesystem
- `fuse` - Filesystem in Userspace

## Common Mount Options

### General Options
- `defaults` - Use default options (rw,suid,dev,exec,auto,nouser,async)
- `rw` - Mount read-write (default)
- `ro` - Mount read-only
- `noatime` - Do not update access times
- `relatime` - Update access times relative to modify/change time
- `sync` - Synchronous I/O
- `async` - Asynchronous I/O (default)
- `auto` - Can be mounted with -a option
- `noauto` - Cannot be mounted with -a option
- `user` - Allow ordinary users to mount
- `nouser` - Only root can mount (default)
- `owner` - Allow device owner to mount
- `users` - Allow any user to mount and unmount
- `group` - Allow members of device group to mount

### Security Options
- `suid` - Allow set-user-identifier (default)
- `nosuid` - Ignore set-user-identifier bits
- `dev` - Interpret character/block special devices (default)
- `nodev` - Don't interpret character/block special devices
- `exec` - Allow execution of binaries (default)
- `noexec` - Don't allow execution of binaries

### Performance Options
- `cache=strict` - Cache mode for network filesystems
- `vers=3` - NFS version
- `proto=tcp` - Protocol to use
- `rsize=32768` - Read buffer size
- `wsize=32768` - Write buffer size
- `timeo=14` - Timeout value
- `retrans=3` - Number of retransmissions

## Examples

### Basic Mount
```ini
[Mount]
What=/dev/sdb1
Where=/mnt/data
Type=ext4
Options=defaults

[Install]
WantedBy=multi-user.target
```

### Temporary Filesystem
```ini
[Mount]
What=tmpfs
Where=/tmp/scratch
Type=tmpfs
Options=size=1G,uid=1000,gid=1000,mode=0755

[Install]
WantedBy=multi-user.target
```

### Network Mount (NFS)
```ini
[Mount]
What=server.example.com:/export/home
Where=/mnt/nfs-home
Type=nfs4
Options=vers=4,proto=tcp,rsize=32768,wsize=32768

[Install]
WantedBy=remote-fs.target
```

### Bind Mount
```ini
[Mount]
What=/var/lib/docker
Where=/docker-data
Type=none
Options=bind,rw

[Install]
WantedBy=multi-user.target
```

### Read-Only Mount
```ini
[Mount]
What=/dev/cdrom
Where=/mnt/cdrom
Type=iso9660
Options=ro,noauto

[Install]
WantedBy=multi-user.target
```

### CIFS/SMB Mount
```ini
[Mount]
What=//server.example.com/share
Where=/mnt/samba
Type=cifs
Options=credentials=/etc/samba/credentials,uid=1000,gid=1000,iocharset=utf8

[Install]
WantedBy=multi-user.target
```

### Encrypted Mount
```ini
[Mount]
What=/dev/mapper/encrypted-data
Where=/mnt/encrypted
Type=ext4
Options=defaults,noatime

[Install]
WantedBy=multi-user.target
RequiredBy=data-service.service
```

## Common Patterns

### Data Directory
For application data directories:
```ini
[Mount]
What=/dev/disk/by-label/app-data
Where=/var/lib/myapp
Type=ext4
Options=defaults,noatime
DirectoryMode=0755
```

### Backup Mount
For backup storage:
```ini
[Mount]
What=backup-server:/backups
Where=/mnt/backups
Type=nfs4
Options=ro,vers=4,proto=tcp
```

### Container Storage
For container runtime storage:
```ini
[Mount]
What=/dev/disk/by-label/containers
Where=/var/lib/containers
Type=xfs
Options=defaults,prjquota
```

## Dependencies

### Unit Dependencies
Mount units automatically gain dependencies:
- `Requires=` and `After=` on units for parent mount points
- `Before=` umount.target
- `Conflicts=` and `Before=` umount.target

### Automatic Dependencies
systemd automatically creates dependencies for:
- Parent directory mount points
- Device units for block devices
- Network targets for network mounts

## Management Commands

### List Mount Units
```bash
systemctl list-units --type=mount
```

### Show Mount Status
```bash
systemctl status mnt-data.mount
```

### Mount/Unmount
```bash
systemctl start mnt-data.mount
systemctl stop mnt-data.mount
```

### Enable at Boot
```bash
systemctl enable mnt-data.mount
```

### Show Mount Points
```bash
findmnt
mount | grep systemd
```

## Troubleshooting

### Common Issues
- Unit name must match mount point (/ becomes -, escape special chars)
- Device must exist before mounting
- Mount point directory is created automatically
- Network mounts require network to be available

### Debugging
```bash
# Check mount unit status
systemctl status unit.mount

# View mount logs
journalctl -u unit.mount

# Test mount manually
mount -t type what where
```

## See Also

- systemd.mount(5)
- mount(8)
- fstab(5)
- systemctl(1)
- systemd(1)
- findmnt(8)