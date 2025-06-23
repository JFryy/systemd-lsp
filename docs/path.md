# [Path] Section

The `[Path]` section contains path-specific settings. Path units are used to activate other units when file system objects change or are accessed. This provides a more efficient alternative to cron jobs for file system monitoring tasks.

## Path Monitoring Types

### PathExists=
If the specified absolute path exists, the configured unit is activated. Takes an absolute file system path as argument.

### PathExistsGlob=
Works like PathExists= but checks for the existence of at least one file matching the glob pattern. Takes a glob pattern as argument.

### PathChanged=
If the specified absolute path is modified, the configured unit is activated. This includes changes to the file content, metadata, or the path being created/deleted.

### PathModified=
Similar to PathChanged= but only watches for content changes (writes), not metadata changes like permission or ownership changes.

### DirectoryNotEmpty=
If the specified directory exists and contains at least one file, the configured unit is activated.

## Path Configuration

### Unit=
The unit to activate when the path condition is met. If not specified, defaults to a service unit with the same name as the path unit (except for the suffix).

### MakeDirectory=
Takes a boolean argument. If true, the directories to watch are created before watching. This only applies to PathExists= and PathChanged= options.

### DirectoryMode=
If MakeDirectory= is enabled, use this mode when creating the directory. Takes a numeric mode value in octal notation.

## Examples

### Watch Configuration File
```ini
[Path]
PathChanged=/etc/myapp/config.conf
Unit=myapp-reload.service

[Install]
WantedBy=multi-user.target
```

### Monitor Log Directory
```ini
[Path]
DirectoryNotEmpty=/var/log/myapp
Unit=log-processor.service

[Install]
WantedBy=multi-user.target
```

### Watch for New Files
```ini
[Path]
PathExistsGlob=/incoming/*.txt
Unit=file-processor.service
MakeDirectory=true
DirectoryMode=0755

[Install]
WantedBy=multi-user.target
```

### USB Device Detection
```ini
[Path]
PathExists=/dev/disk/by-label/BACKUP-USB
Unit=backup-to-usb.service

[Install]
WantedBy=multi-user.target
```

### Configuration Reload
```ini
[Path]
PathModified=/etc/nginx/nginx.conf
PathModified=/etc/nginx/sites-enabled
Unit=nginx-reload.service

[Install]
WantedBy=multi-user.target
```

### Temporary File Cleanup
```ini
[Path]
DirectoryNotEmpty=/tmp/uploads
Unit=cleanup-uploads.service

[Install]
WantedBy=multi-user.target
```

### Database Backup Trigger
```ini
[Path]
PathExists=/var/lib/mysql/backup-requested
Unit=mysql-backup.service
MakeDirectory=false

[Install]
WantedBy=multi-user.target
```

## Common Patterns

### Configuration Reloading
Automatically reload services when configuration changes:
```ini
[Path]
PathChanged=/etc/myservice/
Unit=myservice-reload.service
```

### File Processing
Process files as they appear:
```ini
[Path]
PathExistsGlob=/queue/*.job
Unit=job-processor.service
```

### System Monitoring
Monitor system state changes:
```ini
[Path]
PathExists=/var/run/system-ready
Unit=post-boot-tasks.service
```

### Batch Processing
Trigger batch jobs when work accumulates:
```ini
[Path]
DirectoryNotEmpty=/var/spool/batch
Unit=batch-processor.service
```

## Comparison with Alternatives

### vs. inotify
- Path units use inotify internally but provide systemd integration
- Automatic service management and logging
- Better resource management and limits
- Integration with systemd's dependency system

### vs. cron with find
- More efficient than periodic polling
- Immediate response to changes
- Better error handling and logging
- Integrated with service lifecycle

### vs. File watchers in applications
- Centralized monitoring configuration
- Service can be stopped/started independently of monitoring
- Consistent logging and error handling
- No need to implement file watching in each application

## Advanced Configuration

### Multiple Paths
```ini
[Path]
PathChanged=/etc/app1/config
PathChanged=/etc/app2/config  
PathModified=/var/log/app/error.log
Unit=config-sync.service
```

### Conditional Activation
```ini
[Path]
PathExists=/tmp/maintenance-mode
Unit=maintenance.service

[Install]
WantedBy=multi-user.target
```

### Integration with Timers
```ini
# path-monitor.path
[Path]
DirectoryNotEmpty=/var/spool/reports
Unit=report-processor.timer

# report-processor.timer  
[Timer]
OnActiveSec=5min
Unit=report-processor.service
```

## Service Integration

### Path-Activated Service
The service activated by the path unit should typically be designed to:

1. Process all relevant files/changes
2. Complete quickly or run as oneshot
3. Handle the case where multiple changes occurred
4. Clean up after processing

Example service:
```ini
[Service]
Type=oneshot
ExecStart=/usr/local/bin/process-changes
User=processor
Group=processor
```

### Multi-Shot vs One-Shot
- Use `Type=oneshot` for processing that should complete
- Use `Type=simple` for long-running monitoring
- Consider `RemainAfterExit=yes` for oneshot services

## Debugging

### List Path Units
```bash
systemctl list-units --type=path
```

### Show Path Status
```bash
systemctl status my-monitor.path
```

### View Path Logs
```bash
journalctl -u my-monitor.path -f
```

### Test Path Manually
```bash
# Trigger the path condition
touch /watched/file

# Check if service was activated
systemctl status triggered-service.service
```

## Limitations

### File System Support
- Requires inotify support (most modern Linux filesystems)
- May not work reliably on network filesystems
- Some filesystems have inotify limitations

### Performance Considerations
- Each path unit consumes inotify watches
- System limits on number of watches (/proc/sys/fs/inotify/max_user_watches)
- Avoid watching very large directories

### Race Conditions
- Files might be processed multiple times
- Consider atomic operations (write to temp, then move)
- Handle partial writes (use locking or temporary files)

## Best Practices

### Path Selection
- Watch specific files rather than large directories when possible
- Use PathModified= for content changes, PathChanged= for any changes
- Consider PathExistsGlob= for pattern matching

### Service Design
- Make services idempotent (safe to run multiple times)
- Process all matching files/changes, not just the triggering one
- Include error handling for missing or changed files
- Use appropriate file locking for concurrent access

### Resource Management
- Monitor inotify watch usage
- Consider batch processing for high-frequency changes
- Use appropriate cleanup in services

## See Also

- systemd.path(5)
- inotify(7)
- systemctl(1)
- systemd(1)
- glob(7)