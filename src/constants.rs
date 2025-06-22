use std::collections::HashMap;

pub struct SystemdConstants;

impl SystemdConstants {
    pub fn valid_sections() -> &'static [&'static str] {
        &[
            "Unit", "Service", "Socket", "Timer", "Path", "Mount",
            "Automount", "Swap", "Target", "Device", "Slice", "Scope", "Install"
        ]
    }

    pub fn section_directives() -> HashMap<&'static str, &'static [&'static str]> {
        let mut map = HashMap::new();
        
        map.insert("Unit", &[
            "Description", "Documentation", "Requires", "Wants", "After", "Before",
            "Conflicts", "BindsTo", "PartOf", "Upholds", "OnFailure", "OnSuccess",
            "PropagatesReloadTo", "ReloadPropagatedFrom", "JoinsNamespaceOf",
            "RequiresMountsFor", "OnFailureJobMode", "IgnoreOnIsolate",
            "StopWhenUnneeded", "RefuseManualStart", "RefuseManualStop",
            "AllowIsolate", "DefaultDependencies", "CollectMode",
            "FailureAction", "SuccessAction", "FailureActionExitStatus",
            "SuccessActionExitStatus", "JobTimeoutSec", "JobRunningTimeoutSec",
            "JobTimeoutAction", "JobTimeoutRebootArgument", "StartLimitIntervalSec",
            "StartLimitBurst", "StartLimitAction", "RebootArgument", "SourcePath",
            "Condition", "Assert", "ConditionPathExists", "ConditionPathExistsGlob",
            "ConditionPathIsDirectory", "ConditionPathIsSymbolicLink",
            "ConditionPathIsMountPoint", "ConditionPathIsReadWrite",
            "ConditionDirectoryNotEmpty", "ConditionFileNotEmpty",
            "ConditionFileIsExecutable", "ConditionNeedsUpdate",
            "ConditionFirstBoot", "ConditionKernelCommandLine",
            "ConditionKernelVersion", "ConditionArchitecture",
            "ConditionVirtualization", "ConditionSecurity", "ConditionCapability",
            "ConditionHost", "ConditionACPower", "ConditionMemory", "ConditionCPUs",
            "ConditionEnvironment", "ConditionUser", "ConditionGroup",
            "ConditionControlGroupController", "AssertPathExists",
            "AssertPathExistsGlob", "AssertPathIsDirectory",
            "AssertPathIsSymbolicLink", "AssertPathIsMountPoint",
            "AssertPathIsReadWrite", "AssertDirectoryNotEmpty",
            "AssertFileNotEmpty", "AssertFileIsExecutable", "AssertNeedsUpdate",
            "AssertFirstBoot", "AssertKernelCommandLine", "AssertKernelVersion",
            "AssertArchitecture", "AssertVirtualization", "AssertSecurity",
            "AssertCapability", "AssertHost", "AssertACPower", "AssertMemory",
            "AssertCPUs", "AssertEnvironment", "AssertUser", "AssertGroup",
            "AssertControlGroupController"
        ] as &[&str]);

        map.insert("Service", &[
            "Type", "ExecStart", "ExecStop", "ExecReload", "ExecStartPre",
            "ExecStartPost", "ExecStopPre", "ExecStopPost", "Restart", "RestartSec",
            "RestartPreventExitStatus", "RestartForceExitStatus", "User", "Group",
            "WorkingDirectory", "Environment", "EnvironmentFile", "StandardOutput",
            "StandardError", "StandardInput", "TimeoutStartSec", "TimeoutStopSec",
            "TimeoutAbortSec", "TimeoutSec", "RuntimeMaxSec", "WatchdogSec",
            "PIDFile", "BusName", "Slice", "Delegate", "TasksMax", "MemoryMax",
            "CPUQuota", "IOWeight", "NoNewPrivileges", "ProtectSystem",
            "ProtectHome", "ProtectKernelTunables", "ProtectKernelModules",
            "ProtectKernelLogs", "ProtectControlGroups", "PrivateTmp",
            "PrivateDevices", "PrivateNetwork", "PrivateUsers", "PrivateMounts",
            "ReadWritePaths", "ReadOnlyPaths", "InaccessiblePaths", "BindPaths",
            "BindReadOnlyPaths", "MountAPIVFS", "SystemCallFilter",
            "SystemCallErrorNumber", "SystemCallArchitectures",
            "RestrictAddressFamilies", "RestrictNamespaces", "RestrictRealtime",
            "RestrictSUIDSGID", "RemoveIPC", "DynamicUser", "SupplementaryGroups",
            "PAMName", "CapabilityBoundingSet", "AmbientCapabilities",
            "SecureBits", "UMask", "KeyringMode", "OOMScoreAdjust",
            "TimerSlackNSec", "Personality", "IgnoreSIGPIPE", "TTYPath",
            "TTYReset", "TTYVHangup", "TTYVTDisallocate", "SyslogIdentifier",
            "SyslogFacility", "SyslogLevel", "SyslogLevelPrefix", "LogLevelMax",
            "LogExtraFields", "LogRateLimitIntervalSec", "LogRateLimitBurst"
        ] as &[&str]);

        map.insert("Install", &[
            "WantedBy", "RequiredBy", "Alias", "Also", "DefaultInstance"
        ] as &[&str]);

        map
    }

    pub fn directive_descriptions() -> HashMap<(&'static str, &'static str), &'static str> {
        let mut map = HashMap::new();
        
        // Unit directives
        map.insert(("Unit", "Description"), "A human readable name for the unit. This is used by systemd and other UIs as a user-visible label for the unit");
        map.insert(("Unit", "Documentation"), "A space-separated list of URIs referencing documentation for this unit or its configuration");
        map.insert(("Unit", "Requires"), "Units this unit depends on. If any of the listed units fail to start, this unit will fail too");
        map.insert(("Unit", "Wants"), "Units this unit wants to activate. Unlike Requires, if listed units fail, this unit continues");
        map.insert(("Unit", "After"), "Units that should be started before this unit. Only affects ordering, not dependencies");
        map.insert(("Unit", "Before"), "Units that should be started after this unit. Only affects ordering, not dependencies");

        // Service directives
        map.insert(("Service", "Type"), "Configures the process start-up type. One of: simple, exec, forking, oneshot, dbus, notify, idle");
        map.insert(("Service", "ExecStart"), "Commands with arguments to execute when this service is started. Required for all service types except oneshot");
        map.insert(("Service", "User"), "User name or UID to run the service process under. Affects process privileges");
        map.insert(("Service", "Group"), "Group name or GID to run the service process under. Supplements User setting");
        map.insert(("Service", "WorkingDirectory"), "Working directory for executed processes. Can use ~ for user home");
        map.insert(("Service", "Environment"), "Sets environment variables for executed processes. Format: VAR=value");
        map.insert(("Service", "NoNewPrivileges"), "Disable privilege escalation via execve(). Prevents setuid/setgid and capabilities");
        map.insert(("Service", "ProtectSystem"), "Make file system hierarchy read-only. Options: true, strict, full, false");
        map.insert(("Service", "ProtectHome"), "Make home directories inaccessible. Options: true, false, read-only, tmpfs");
        map.insert(("Service", "PrivateTmp"), "Use private /tmp and /var/tmp directories. Isolates temporary files");

        // Install directives
        map.insert(("Install", "WantedBy"), "Units that should pull this unit in when enabled. Most commonly multi-user.target or graphical.target");
        map.insert(("Install", "RequiredBy"), "Units that strongly depend on this unit when enabled. Stricter than WantedBy");

        map
    }

    pub fn valid_values() -> HashMap<&'static str, &'static [&'static str]> {
        let mut map = HashMap::new();
        
        map.insert("Type", &["simple", "exec", "forking", "oneshot", "dbus", "notify", "idle"] as &[&str]);
        map.insert("Restart", &["no", "on-success", "on-failure", "on-abnormal", "on-watchdog", "on-abort", "always"] as &[&str]);
        map.insert("ProtectSystem", &["true", "false", "strict", "full", "yes", "no"] as &[&str]);
        map.insert("ProtectHome", &["true", "false", "read-only", "tmpfs", "yes", "no"] as &[&str]);
        
        // Boolean values for security directives
        let boolean_values = &["true", "false", "yes", "no", "1", "0"] as &[&str];
        map.insert("NoNewPrivileges", boolean_values);
        map.insert("PrivateTmp", boolean_values);
        map.insert("PrivateDevices", boolean_values);
        map.insert("PrivateNetwork", boolean_values);
        map.insert("PrivateUsers", boolean_values);
        map.insert("PrivateMounts", boolean_values);
        map.insert("ProtectKernelTunables", boolean_values);
        map.insert("ProtectKernelModules", boolean_values);
        map.insert("ProtectKernelLogs", boolean_values);
        map.insert("ProtectControlGroups", boolean_values);
        map.insert("RestrictRealtime", boolean_values);
        map.insert("RestrictSUIDSGID", boolean_values);
        map.insert("RemoveIPC", boolean_values);
        map.insert("DynamicUser", boolean_values);
        map.insert("MountAPIVFS", boolean_values);

        let standard_io_values = &["inherit", "null", "tty", "journal", "kmsg", "journal+console", "kmsg+console", "file:", "append:", "truncate:", "socket"] as &[&str];
        map.insert("StandardOutput", standard_io_values);
        map.insert("StandardError", standard_io_values);

        map
    }

    pub fn section_documentation() -> HashMap<&'static str, &'static str> {
        let mut map = HashMap::new();
        
        map.insert("Unit", "The [Unit] section contains generic information about the unit that is not dependent on the type of unit. Options in this section are shared by all unit types. This section is used to define metadata about the unit (such as dependencies), conditions and assertions that must be met before the unit is started, and various other configuration settings.");
        
        map.insert("Service", "The [Service] section contains information about the service and the process it supervises. This section is used for units of Type=service. A number of options that may be used in this section are shared with other unit types. These options are documented in systemd.exec(5), systemd.kill(5) and systemd.resource-control(5).");
        
        map.insert("Socket", "The [Socket] section contains information about a socket or a FIFO controlled and supervised by systemd, for socket-based activation. This section is used for units of Type=socket.");
        
        map.insert("Timer", "The [Timer] section contains information about a timer controlled and supervised by systemd, for timer-based activation. This section is used for units of Type=timer.");
        
        map.insert("Path", "The [Path] section contains information about a path or set of paths monitored by systemd, for path-based activation. This section is used for units of Type=path. Path units automatically gain dependencies of type Requires= and After= on the service they are intended to activate.");
        
        map.insert("Mount", "The [Mount] section contains information about a file system mount point controlled and supervised by systemd. This section is used for units of Type=mount. Mount units automatically gain dependencies of type Requires= and After= on the device units of the block devices they reference.");
        
        map.insert("Automount", "The [Automount] section contains information about a file system automount point controlled and supervised by systemd. This section is used for units of Type=automount. Automount units must be named after the automount directories they control.");
        
        map.insert("Swap", "The [Swap] section contains information about a swap device or file controlled and supervised by systemd. This section is used for units of Type=swap. Swap units must be named after the devices or files they control.");
        
        map.insert("Target", "The [Target] section contains information about a target unit of systemd, which is used for grouping units and as well-known synchronization points during start-up. This section is used for units of Type=target. This unit type has no specific options.");
        
        map.insert("Device", "The [Device] section contains information about a device unit as exposed in the sysfs/udev device tree. This section is used for units of Type=device. Device units are automatically created for all devices that are tagged as such in the udev database.");
        
        map.insert("Slice", "The [Slice] section contains information about a process resource slice. This section is used for units of Type=slice. Slice units manage resources for groups of processes and implement a tree of resource control groups.");
        
        map.insert("Scope", "The [Scope] section contains information about a service scope unit. This section is used for units of Type=scope. Unlike service units, scope units manage externally created processes, and do not fork off processes on their own.");
        
        map.insert("Install", "The [Install] section contains installation information for the unit. This section is not interpreted by systemd(1) during runtime; it is used by the enable and disable commands of the systemctl(1) tool during installation of a unit.");

        map
    }

    pub const APP_NAME: &'static str = "systemdls";
}