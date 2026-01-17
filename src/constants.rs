use std::collections::HashMap;

pub struct SystemdConstants;

impl SystemdConstants {
    pub fn valid_sections() -> Vec<&'static str> {
        include_str!("../docs/sections.txt").lines().collect()
    }

    /*
    // Lists containing valid directives for each section
    // this is stored in docs and _should_ be generated from
    // from the parent documentation to keep it up to date
    // and not prone to human error.
    */
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
            "Automount",
            include_str!("../docs/directives/automount.txt")
                .lines()
                .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
                .collect(),
        );
        map.insert(
            "Slice",
            include_str!("../docs/directives/slice.txt")
                .lines()
                .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
                .collect(),
        );
        map.insert(
            "Scope",
            include_str!("../docs/directives/scope.txt")
                .lines()
                .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
                .collect(),
        );
        map.insert(
            "Container",
            include_str!("../docs/directives/container.txt")
                .lines()
                .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
                .collect(),
        );
        map.insert(
            "Pod",
            include_str!("../docs/directives/pod.txt")
                .lines()
                .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
                .collect(),
        );
        map.insert(
            "Volume",
            include_str!("../docs/directives/volume.txt")
                .lines()
                .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
                .collect(),
        );
        map.insert(
            "Network",
            include_str!("../docs/directives/network.txt")
                .lines()
                .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
                .collect(),
        );
        map.insert(
            "Kube",
            include_str!("../docs/directives/kube.txt")
                .lines()
                .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
                .collect(),
        );
        map.insert(
            "Build",
            include_str!("../docs/directives/build.txt")
                .lines()
                .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
                .collect(),
        );
        map.insert(
            "Image",
            include_str!("../docs/directives/image.txt")
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

        // Note: Type= is now handled context-sensitively in valid_values_for_section()
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
        map.insert("LockPersonality", boolean_values);
        map.insert("MemoryDenyWriteExecute", boolean_values);
        map.insert("ProtectHostname", boolean_values);
        map.insert("ProtectClock", boolean_values);
        map.insert("PrivatePIDs", boolean_values);

        // PrivateTmp with additional 'disconnected' value
        map.insert(
            "PrivateTmp",
            &["true", "false", "yes", "no", "1", "0", "disconnected"] as &[&str],
        );

        // DevicePolicy values
        map.insert("DevicePolicy", &["auto", "closed", "strict"] as &[&str]);

        // ProtectProc values
        map.insert(
            "ProtectProc",
            &["noaccess", "invisible", "ptraceable", "default"] as &[&str],
        );

        // NotifyAccess values
        map.insert("NotifyAccess", &["none", "main", "exec", "all"] as &[&str]);

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

    pub fn valid_values_for_section(
        section: &str,
        directive: &str,
    ) -> Option<&'static [&'static str]> {
        match (section, directive) {
            ("Service", "Type") => Some(&[
                "simple",
                "exec",
                "forking",
                "oneshot",
                "dbus",
                "notify",
                "notify-reload",
                "idle",
            ]),
            ("Mount", "Type") => Some(&[
                "ext4", "ext3", "ext2", "xfs", "btrfs", "vfat", "ntfs", "exfat", "iso9660",
                "tmpfs", "proc", "sysfs", "devpts", "nfs", "nfs4", "cifs", "sshfs", "bind",
                "overlay", "squashfs", "fuse", "none", "auto",
            ]),
            _ => {
                // Fall back to global valid_values for other directives
                let global_values = Self::valid_values();
                global_values.get(directive).copied()
            }
        }
    }

    pub fn section_documentation() -> HashMap<&'static str, &'static str> {
        let mut map = HashMap::new();

        // Use comprehensive markdown files for detailed documentation
        map.insert("Unit", include_str!("../docs/unit.md"));
        map.insert("Service", include_str!("../docs/service.md"));
        map.insert("Install", include_str!("../docs/install.md"));
        map.insert("Timer", include_str!("../docs/timer.md"));
        map.insert("Socket", include_str!("../docs/socket.md"));
        map.insert("Mount", include_str!("../docs/mount.md"));
        map.insert("Path", include_str!("../docs/path.md"));
        map.insert("Swap", include_str!("../docs/swap.md"));
        map.insert("Container", include_str!("../docs/container.md"));
        map.insert("Pod", include_str!("../docs/pod.md"));
        map.insert("Volume", include_str!("../docs/volume.md"));
        map.insert("Network", include_str!("../docs/network.md"));
        map.insert("Kube", include_str!("../docs/kube.md"));
        map.insert("Build", include_str!("../docs/build.md"));
        map.insert("Image", include_str!("../docs/image.md"));

        map.insert("Automount", include_str!("../docs/automount.md"));
        map.insert("Slice", include_str!("../docs/slice.md"));
        map.insert("Scope", include_str!("../docs/scope.md"));

        map
    }

    /// Get shared documentation files (exec, kill, resource-control)
    /// These contain directives that are shared across multiple section types
    pub fn shared_documentation() -> HashMap<&'static str, &'static str> {
        let mut map = HashMap::new();
        map.insert("exec", include_str!("../docs/exec.md"));
        map.insert("kill", include_str!("../docs/kill.md"));
        map.insert("resource-control", include_str!("../docs/resource-control.md"));
        map
    }

    /// Returns the list of shared documentation keys that apply to a given section
    /// This mapping determines which additional directive sets are available in each section
    pub fn section_shared_docs(section: &str) -> Vec<&'static str> {
        match section.to_lowercase().as_str() {
            "service" => vec!["exec", "kill", "resource-control"],
            "socket" => vec!["exec", "kill", "resource-control"],
            "mount" => vec!["exec", "kill", "resource-control"],
            "swap" => vec!["exec", "kill", "resource-control"],
            "scope" => vec!["kill", "resource-control"],
            "slice" => vec!["resource-control"],
            _ => vec![],
        }
    }

    pub const APP_NAME: &'static str = "systemdls";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_sections_not_empty() {
        let sections = SystemdConstants::valid_sections();
        assert!(!sections.is_empty());

        // Check that common sections are present
        assert!(sections.contains(&"Unit"));
        assert!(sections.contains(&"Service"));
        assert!(sections.contains(&"Install"));
    }

    #[test]
    fn test_section_directives_contains_expected_sections() {
        let directives = SystemdConstants::section_directives();

        // Check that main sections are present
        assert!(directives.contains_key("Unit"));
        assert!(directives.contains_key("Service"));
        assert!(directives.contains_key("Install"));
        assert!(directives.contains_key("Timer"));
        assert!(directives.contains_key("Socket"));
        assert!(directives.contains_key("Mount"));
        assert!(directives.contains_key("Path"));
        assert!(directives.contains_key("Swap"));
        assert!(directives.contains_key("Automount"));
        assert!(directives.contains_key("Slice"));
        assert!(directives.contains_key("Scope"));
    }

    #[test]
    fn test_section_directives_contain_valid_directives() {
        let directives = SystemdConstants::section_directives();

        // Test Unit section directives
        let unit_directives = directives.get("Unit").unwrap();
        assert!(!unit_directives.is_empty());
        assert!(unit_directives.contains(&"Description"));

        // Test Service section directives
        let service_directives = directives.get("Service").unwrap();
        assert!(!service_directives.is_empty());
        assert!(service_directives.contains(&"Type"));
        assert!(service_directives.contains(&"ExecStart"));

        // Test Install section directives
        let install_directives = directives.get("Install").unwrap();
        assert!(!install_directives.is_empty());
        assert!(install_directives.contains(&"WantedBy"));
    }

    #[test]
    fn test_directive_descriptions_contain_expected_entries() {
        let descriptions = SystemdConstants::directive_descriptions();

        // Test that key directive descriptions exist
        assert!(descriptions.contains_key(&("Unit", "Description")));
        assert!(descriptions.contains_key(&("Unit", "Wants")));
        assert!(descriptions.contains_key(&("Service", "Type")));
        assert!(descriptions.contains_key(&("Service", "ExecStart")));
        assert!(descriptions.contains_key(&("Install", "WantedBy")));

        // Test that descriptions are not empty
        assert!(!descriptions[&("Unit", "Description")].is_empty());
        assert!(!descriptions[&("Service", "Type")].is_empty());
    }

    #[test]
    fn test_valid_values_for_type_directive() {
        // Test Service Type directive
        let service_type_values =
            SystemdConstants::valid_values_for_section("Service", "Type").unwrap();
        assert!(service_type_values.contains(&"simple"));
        assert!(service_type_values.contains(&"exec"));
        assert!(service_type_values.contains(&"forking"));
        assert!(service_type_values.contains(&"oneshot"));
        assert!(service_type_values.contains(&"dbus"));
        assert!(service_type_values.contains(&"notify"));
        assert!(service_type_values.contains(&"idle"));

        // Test Mount Type directive
        let mount_type_values =
            SystemdConstants::valid_values_for_section("Mount", "Type").unwrap();
        assert!(mount_type_values.contains(&"ext4"));
        assert!(mount_type_values.contains(&"exfat"));
        assert!(mount_type_values.contains(&"ntfs"));
        assert!(mount_type_values.contains(&"tmpfs"));
    }

    #[test]
    fn test_valid_values_for_restart_directive() {
        let valid_values = SystemdConstants::valid_values();

        let restart_values = valid_values.get("Restart").unwrap();
        assert!(restart_values.contains(&"no"));
        assert!(restart_values.contains(&"on-success"));
        assert!(restart_values.contains(&"on-failure"));
        assert!(restart_values.contains(&"always"));
    }

    #[test]
    fn test_valid_values_for_boolean_directives() {
        let valid_values = SystemdConstants::valid_values();

        let boolean_directives = [
            "NoNewPrivileges",
            "PrivateTmp",
            "PrivateDevices",
            "PrivateNetwork",
            "DynamicUser",
        ];

        for directive in &boolean_directives {
            let values = valid_values.get(directive).unwrap();
            assert!(values.contains(&"true"));
            assert!(values.contains(&"false"));
            assert!(values.contains(&"yes"));
            assert!(values.contains(&"no"));
            assert!(values.contains(&"1"));
            assert!(values.contains(&"0"));
        }
    }

    #[test]
    fn test_valid_values_for_standard_io() {
        let valid_values = SystemdConstants::valid_values();

        let standard_output = valid_values.get("StandardOutput").unwrap();
        assert!(standard_output.contains(&"inherit"));
        assert!(standard_output.contains(&"null"));
        assert!(standard_output.contains(&"journal"));
        assert!(standard_output.contains(&"file:"));

        let standard_error = valid_values.get("StandardError").unwrap();
        assert!(standard_error.contains(&"inherit"));
        assert!(standard_error.contains(&"null"));
        assert!(standard_error.contains(&"journal"));
    }

    #[test]
    fn test_section_documentation_not_empty() {
        let docs = SystemdConstants::section_documentation();

        // Check that main sections have documentation
        assert!(docs.contains_key("Unit"));
        assert!(docs.contains_key("Service"));
        assert!(docs.contains_key("Install"));
        assert!(docs.contains_key("Automount"));
        assert!(docs.contains_key("Slice"));
        assert!(docs.contains_key("Scope"));

        // Check that documentation is not empty
        assert!(!docs["Unit"].is_empty());
        assert!(!docs["Service"].is_empty());
        assert!(!docs["Install"].is_empty());
        assert!(!docs["Automount"].is_empty());
        assert!(!docs["Slice"].is_empty());
        assert!(!docs["Scope"].is_empty());
    }

    #[test]
    fn test_app_name_constant() {
        assert_eq!(SystemdConstants::APP_NAME, "systemdls");
    }

    #[test]
    fn test_protect_system_values() {
        let valid_values = SystemdConstants::valid_values();
        let protect_system = valid_values.get("ProtectSystem").unwrap();

        assert!(protect_system.contains(&"true"));
        assert!(protect_system.contains(&"false"));
        assert!(protect_system.contains(&"strict"));
        assert!(protect_system.contains(&"full"));
        assert!(protect_system.contains(&"yes"));
        assert!(protect_system.contains(&"no"));
    }

    #[test]
    fn test_protect_home_values() {
        let valid_values = SystemdConstants::valid_values();
        let protect_home = valid_values.get("ProtectHome").unwrap();

        assert!(protect_home.contains(&"true"));
        assert!(protect_home.contains(&"false"));
        assert!(protect_home.contains(&"read-only"));
        assert!(protect_home.contains(&"tmpfs"));
        assert!(protect_home.contains(&"yes"));
        assert!(protect_home.contains(&"no"));
    }
}
