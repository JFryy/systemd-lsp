# [Swap] Section

The `[Swap]` section contains swap-specific settings. Swap units are used to control swap files and swap partitions. Similar to mount units, swap units must be named after the swap file/device they control, with slashes replaced by dashes.

## Required Directives

### What=
Takes an absolute path of a device node or swap file to activate. This corresponds to the device field in /etc/fstab or the device parameter of swapon(8).

## Optional Directives

### Priority=
Swap priority, a value between -1 and 32767. Higher numbers indicate higher priority. If not specified, the kernel will assign a priority automatically.

### Options=
May contain an option string for the swap file/device. This corresponds to the options field in /etc/fstab.

### TimeoutSec=
Configures the time to wait for the swapon/swapoff command to finish.

## Swap Priority

### Priority Values
- Range: -1 to 32767
- Higher numbers = higher priority  
- Default: kernel assigns automatically (usually -1 to -3)
- Equal priority: kernel uses round-robin

### Priority Strategy
- **High priority** (32767): For fastest storage (NVMe, SSD)
- **Medium priority** (100-1000): For regular SSDs
- **Low priority** (1-99): For HDDs or network storage
- **Negative priority**: Let kernel decide automatically

## Common Swap Options

### General Options
- `defaults` - Use default options (equivalent to no options)
- `pri=N` - Set priority (alternative to Priority= directive)
- `discard` - Enable TRIM support for SSDs (discard unused blocks)
- `nofail` - Don't fail boot if swap cannot be enabled

### Performance Options
- `discard=once` - Discard all blocks at swapon time
- `discard=pages` - Discard individual pages when freed  
- `discard` - Same as discard=pages

## Examples

### Basic Swap Partition
```ini
[Swap]
What=/dev/sda2

[Install]
WantedBy=swap.target
```

### High Priority SSD Swap
```ini
[Swap]
What=/dev/nvme0n1p3
Priority=1000
Options=discard

[Install]
WantedBy=swap.target
```

### Swap File
```ini
[Swap]
What=/swapfile
Priority=100

[Install]
WantedBy=swap.target
```

### Multiple Swap Devices
```ini
# fast-swap.swap
[Swap]  
What=/dev/ssd/swap
Priority=1000
Options=discard

[Install]
WantedBy=swap.target

# slow-swap.swap
[Swap]
What=/dev/hdd/swap  
Priority=100

[Install]
WantedBy=swap.target
```

### Network Swap (NBD)
```ini
[Swap]
What=/dev/nbd0
Priority=10
Options=nofail
TimeoutSec=30

[Install]
WantedBy=swap.target
```

### Encrypted Swap
```ini
[Swap]
What=/dev/mapper/swap-crypt
Priority=500

[Install]
WantedBy=swap.target
```

## Swap File Creation

### Create Swap File
```bash
# Create 2GB swap file
sudo fallocate -l 2G /swapfile
# OR
sudo dd if=/dev/zero of=/swapfile bs=1M count=2048

# Set permissions
sudo chmod 600 /swapfile

# Make it a swap file
sudo mkswap /swapfile

# Verify
sudo swapon --show
```

### Unit File for Swap File
```ini
[Swap]
What=/swapfile
Priority=100

[Install]  
WantedBy=swap.target
```

## Swap Partition Setup

### Create Swap Partition
```bash
# Create partition with fdisk/parted
sudo fdisk /dev/sda

# Make it a swap partition
sudo mkswap /dev/sda2

# Get UUID for stable naming
blkid /dev/sda2
```

### Unit File with UUID
```ini
[Swap]
What=/dev/disk/by-uuid/12345678-1234-1234-1234-123456789abc
Priority=1000
Options=discard

[Install]
WantedBy=swap.target  
```

## Swap Strategies

### Single Fast Swap
For systems with one fast storage device:
```ini
[Swap]
What=/dev/nvme0n1p2
Priority=1000
Options=discard
```

### Tiered Swap
For systems with multiple storage tiers:
```ini
# nvme-swap.swap (highest priority)
[Swap]
What=/dev/nvme0n1p3
Priority=2000
Options=discard

# ssd-swap.swap (medium priority)  
[Swap]
What=/dev/sda3
Priority=1000
Options=discard

# hdd-swap.swap (lowest priority)
[Swap]  
What=/dev/sdb2
Priority=100
```

### Temporary Swap
For emergency situations:
```ini
[Swap]
What=/tmp/emergency.swap
Priority=1
Options=nofail
```

## Performance Considerations

### SSD Optimization
- Use `discard` option for TRIM support
- Set higher priority for faster SSDs
- Consider wear leveling impact

### HDD Optimization  
- Place swap on outer tracks (lower partition numbers)
- Consider separate disk for swap to reduce seek times
- Use lower priority

### Network Swap
- Use `nofail` option
- Set longer TimeoutSec=
- Use very low priority
- Consider reliability implications

## Monitoring

### Check Swap Usage
```bash
# Show swap usage
swapon --show
free -h
cat /proc/swaps

# Show swap activity
sar -S 1 10
iostat -x 1 10
```

### Swap Statistics
```bash
# Swap in/out rates
vmstat 1 10

# Per-process swap usage
for pid in $(ps -eo pid --no-headers); do
    if [ -r /proc/$pid/smaps ]; then
        swap=$(awk '/^Swap:/ { sum += $2 } END { print sum }' /proc/$pid/smaps 2>/dev/null)
        if [ "$swap" -gt 0 ] 2>/dev/null; then
            echo "PID $pid: ${swap}kB"
        fi
    fi
done
```

## Dependencies

### Automatic Dependencies
Swap units automatically gain:
- `Requires=` and `After=` on device units for block devices
- `Before=` umount.target  
- `Conflicts=` and `Before=` umount.target

### Manual Dependencies
```ini
[Unit]
RequiredBy=hibernate.service
After=dev-disk-by\x2dlabel-swap.device

[Swap]
What=/dev/disk/by-label/swap
```

## Management Commands

### List Swap Units
```bash
systemctl list-units --type=swap
systemctl list-unit-files --type=swap
```

### Control Swap
```bash
# Start swap unit
systemctl start dev-sda2.swap

# Stop swap unit  
systemctl stop dev-sda2.swap

# Enable at boot
systemctl enable dev-sda2.swap

# Status
systemctl status dev-sda2.swap
```

### Manual Swap Control
```bash
# Enable swap manually
sudo swapon /dev/sda2
sudo swapon /swapfile

# Disable swap manually
sudo swapoff /dev/sda2  
sudo swapoff -a

# Show active swap
swapon --show
```

## Troubleshooting

### Common Issues
- Unit name must match device path (/ becomes -, escape special chars)
- Device must exist and be formatted as swap
- Permissions must be correct for swap files (600)
- Sufficient disk space for swap files

### Debugging
```bash
# Check swap unit status
systemctl status dev-sda2.swap

# View swap logs
journalctl -u dev-sda2.swap

# Test swap manually
sudo swapon /dev/sda2
sudo swapoff /dev/sda2
```

### Performance Issues
- Monitor swap usage with `sar -S`
- Check for excessive swapping (thrashing)
- Consider increasing RAM or optimizing applications
- Verify swap device performance with `iostat`

## Security Considerations

### Swap File Security
- Set restrictive permissions (600)
- Consider encrypted swap for sensitive data
- Secure deletion of swap files when no longer needed

### Encrypted Swap
```bash
# Create encrypted swap
sudo cryptsetup luksFormat /dev/sda2
sudo cryptsetup luksOpen /dev/sda2 swap-crypt
sudo mkswap /dev/mapper/swap-crypt
```

## See Also

- systemd.swap(5)
- swapon(8)
- swapoff(8)
- mkswap(8)
- fstab(5)
- systemctl(1)
- systemd(1)