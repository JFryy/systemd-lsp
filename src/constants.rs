use std::collections::HashMap;

pub struct SystemdConstants;

impl SystemdConstants {
    pub fn valid_sections() -> Vec<&'static str> {
        include_str!("../docs/sections.txt").lines().collect()
    }

    pub fn section_directives() -> HashMap<&'static str, Vec<&'static str>> {
        let mut map = HashMap::new();

        map.insert(
            "Unit",
            include_str!("../docs/directives/unit.txt")
                .lines()
                .collect(),
        );
        map.insert(
            "Service",
            include_str!("../docs/directives/service.txt")
                .lines()
                .collect(),
        );
        map.insert(
            "Install",
            include_str!("../docs/directives/install.txt")
                .lines()
                .collect(),
        );
        map.insert(
            "Timer",
            include_str!("../docs/directives/timer.txt")
                .lines()
                .collect(),
        );
        map.insert(
            "Socket",
            include_str!("../docs/directives/socket.txt")
                .lines()
                .collect(),
        );
        map.insert(
            "Mount",
            include_str!("../docs/directives/mount.txt")
                .lines()
                .collect(),
        );
        map.insert(
            "Path",
            include_str!("../docs/directives/path.txt")
                .lines()
                .collect(),
        );
        map.insert(
            "Swap",
            include_str!("../docs/directives/swap.txt")
                .lines()
                .collect(),
        );
        map.insert(
            "Target",
            include_str!("../docs/directives/target.txt")
                .lines()
                .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
                .collect(),
        );

        map
    }

    pub fn directive_descriptions() -> HashMap<(&'static str, &'static str), &'static str> {
        let mut map = HashMap::new();

        // Unit directives
        map.insert(
            ("Unit", "Description"),
            include_str!("../docs/directives/unit/description.txt"),
        );
        map.insert(
            ("Unit", "Wants"),
            include_str!("../docs/directives/unit/wants.txt"),
        );

        // Service directives
        map.insert(
            ("Service", "Type"),
            include_str!("../docs/directives/service/type.txt"),
        );
        map.insert(
            ("Service", "ExecStart"),
            include_str!("../docs/directives/service/execstart.txt"),
        );

        // Install directives
        map.insert(
            ("Install", "WantedBy"),
            include_str!("../docs/directives/install/wantedby.txt"),
        );

        map
    }

    pub fn valid_values() -> HashMap<&'static str, &'static [&'static str]> {
        let mut map = HashMap::new();

        map.insert(
            "Type",
            &[
                "simple", "exec", "forking", "oneshot", "dbus", "notify", "idle",
            ] as &[&str],
        );
        map.insert(
            "Restart",
            &[
                "no",
                "on-success",
                "on-failure",
                "on-abnormal",
                "on-watchdog",
                "on-abort",
                "always",
            ] as &[&str],
        );
        map.insert(
            "ProtectSystem",
            &["true", "false", "strict", "full", "yes", "no"] as &[&str],
        );
        map.insert(
            "ProtectHome",
            &["true", "false", "read-only", "tmpfs", "yes", "no"] as &[&str],
        );

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

        let standard_io_values = &[
            "inherit",
            "null",
            "tty",
            "journal",
            "kmsg",
            "journal+console",
            "kmsg+console",
            "file:",
            "append:",
            "truncate:",
            "socket",
        ] as &[&str];
        map.insert("StandardOutput", standard_io_values);
        map.insert("StandardError", standard_io_values);

        map
    }

    pub fn section_documentation() -> HashMap<&'static str, &'static str> {
        let mut map = HashMap::new();

        map.insert("Unit", include_str!("../docs/sections/unit.txt"));
        map.insert("Service", include_str!("../docs/sections/service.txt"));
        map.insert("Install", include_str!("../docs/sections/install.txt"));
        map.insert("Timer", include_str!("../docs/sections/timer.txt"));
        map.insert("Socket", include_str!("../docs/sections/socket.txt"));
        map.insert("Mount", include_str!("../docs/sections/mount.txt"));
        map.insert("Path", include_str!("../docs/sections/path.txt"));
        map.insert("Swap", include_str!("../docs/sections/swap.txt"));

        map
    }

    pub const APP_NAME: &'static str = "systemdls";
}

