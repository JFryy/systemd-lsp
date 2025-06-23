# [Timer] Section

The `[Timer]` section encodes information about a timer controlled and supervised by systemd, for timer-based activation. Timer units are used to schedule activation of other units at specified times or intervals based on monotonic or realtime timers.

*Based on [systemd.timer(5)](https://www.freedesktop.org/software/systemd/man/systemd.timer.html) official documentation.*

## Timer Types

### OnActiveSec=
Defines a timer relative to the moment the timer unit itself is activated. Takes a time span as argument.

### OnBootSec=
Defines a timer relative to when the machine was booted up. Takes a time span as argument.

### OnStartupSec=
Defines a timer relative to when systemd was first started. Takes a time span as argument.

### OnUnitActiveSec=
Defines a timer relative to when the unit the timer unit is activating was last activated. Takes a time span as argument.

### OnUnitInactiveSec=
Defines a timer relative to when the unit the timer unit is activating was last deactivated. Takes a time span as argument.

### OnCalendar=
Defines realtime (i.e., wallclock) timers with calendar event expressions. Takes a calendar specification as argument.

## Time Specifications

### Time Spans
Time spans are specified in the following units:
- `us`, `μs` (microseconds)
- `ms` (milliseconds)  
- `s`, `sec`, `second`, `seconds`
- `m`, `min`, `minute`, `minutes`
- `h`, `hr`, `hour`, `hours`
- `d`, `day`, `days`
- `w`, `week`, `weeks`
- `month`, `months` (30.44 days)
- `y`, `year`, `years` (365.25 days)

Examples: `2h`, `30min`, `1d 12h`, `5s`

### Calendar Events
Calendar events may be specified in the following formats:

#### Weekday Patterns
- `Mon,Tue,Wed,Thu,Fri` - Weekdays
- `Sat,Sun` - Weekends
- `Mon..Fri` - Monday through Friday

#### Date and Time Patterns
- `yearly` or `annually` - January 1st, 00:00:00
- `monthly` - 1st day of month, 00:00:00
- `weekly` - Every Monday, 00:00:00
- `daily` or `midnight` - Every day, 00:00:00
- `hourly` - Every hour, minute 0
- `minutely` - Every minute
- `secondly` - Every second

#### Specific Dates and Times
- `2023-05-15 14:30:00` - Specific date and time
- `Mon 10:00` - Every Monday at 10:00
- `*-*-* 04:00:00` - Every day at 04:00
- `*:0/15` - Every 15 minutes
- `Mon..Fri 09:00` - Weekdays at 9 AM

## Timer Configuration

### Unit=
The unit to activate when this timer elapses. If not specified, defaults to a service unit with the same name as the timer unit (except for the suffix).

### Persistent=
Takes a boolean argument. If true, the time when the service unit was last triggered is stored on disk. When the timer is activated, this information is used to possibly trigger missed runs.

### WakeSystem=
Takes a boolean argument. If true, an elapsing timer will cause the system to resume from suspend/hibernate if it was suspended at the time the timer elapses.

### RemainAfterElapse=
Takes a boolean argument. If true, the timer will stay active after it elapses. Default is true for OnCalendar= timers, false for other timer types.

### AccuracySec=
Specify the accuracy the timer shall elapse with. Defaults to 1min. The timer will elapse within a time window starting with the time specified in OnCalendar=, OnActiveSec=, etc. and ending AccuracySec= later.

### RandomizedDelaySec=
Delay the timer by a randomly selected, evenly distributed amount of time. This can be used to avoid having all timers fire at the same time.

## Examples

### Daily Backup
```ini
[Timer]
OnCalendar=daily
Persistent=true
Unit=backup.service

[Install]
WantedBy=timers.target
```

### Every 5 Minutes
```ini
[Timer]
OnCalendar=*:0/5
Unit=monitor.service

[Install]
WantedBy=timers.target
```

### Weekday Morning
```ini
[Timer]
OnCalendar=Mon..Fri 08:00
Persistent=true
WakeSystem=false
Unit=workday-start.service

[Install]
WantedBy=timers.target
```

### Relative Timer
```ini
[Timer]
OnBootSec=10min
OnUnitActiveSec=15min
Unit=maintenance.service

[Install]
WantedBy=timers.target
```

### Monthly with Randomization
```ini
[Timer]
OnCalendar=monthly
RandomizedDelaySec=3600
Persistent=true
Unit=monthly-report.service

[Install]
WantedBy=timers.target
```

## Common Patterns

### Replace Cron Jobs
Instead of cron's `0 2 * * *` (daily at 2 AM):
```ini
[Timer]
OnCalendar=02:00
Persistent=true
```

### System Maintenance
For tasks that should run when system is idle:
```ini
[Timer]
OnCalendar=03:00
AccuracySec=30min
RandomizedDelaySec=1h
```

### Monitoring
For frequent checks:
```ini
[Timer]
OnCalendar=*:0/10
AccuracySec=1s
```

## Management Commands

### List Active Timers
```bash
systemctl list-timers
```

### Show Timer Status
```bash
systemctl status my-timer.timer
```

### Start Timer Immediately
```bash
systemctl start my-timer.timer
```

### Enable Timer
```bash
systemctl enable my-timer.timer
```

## See Also

- [systemd.timer(5)](https://www.freedesktop.org/software/systemd/man/systemd.timer.html) - Timer unit configuration
- [systemd.time(7)](https://www.freedesktop.org/software/systemd/man/systemd.time.html) - Time and date specifications
- [systemctl(1)](https://www.freedesktop.org/software/systemd/man/systemctl.html) - Control systemd services and units
- [systemd(1)](https://www.freedesktop.org/software/systemd/man/systemd.html) - systemd system and service manager
- [crontab(5)](https://man7.org/linux/man-pages/man5/crontab.5.html) - Tables for driving cron