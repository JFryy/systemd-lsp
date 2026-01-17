package main

import "strings"

type Config struct {
	OutputDir      string
	Pages          []DocPage
	SharedPages    map[string]string
	SharedIncludes map[string][]string
}

func NewConfig(systemdBackend, podmanBackend Backend) *Config {
	base := "https://www.freedesktop.org/software/systemd/man/latest/systemd."
	sd := func(name, filter string) DocPage {
		lower := strings.ToLower(name)
		return DocPage{name, base + lower + ".html", lower + ".md", systemdBackend, filter}
	}
	pm := func(name string) DocPage {
		return DocPage{name, "https://docs.podman.io/en/latest/markdown/podman-systemd.unit.5.html", strings.ToLower(name) + ".md", podmanBackend, name}
	}

	return &Config{
		OutputDir: "../../docs",
		Pages: []DocPage{
			// Main systemd sections
			sd("Unit", "Unit"), sd("Service", ""), sd("Socket", ""), sd("Timer", ""),
			sd("Mount", ""), sd("Path", ""), sd("Swap", ""),
			DocPage{"Install", base + "unit.html", "install.md", systemdBackend, "Install"},
			sd("Automount", ""), sd("Slice", ""), sd("Scope", ""),

			// Shared directive pages - generated separately for LSP to merge at runtime
			DocPage{"Exec", base + "exec.html", "exec.md", systemdBackend, ""},
			DocPage{"Kill", base + "kill.html", "kill.md", systemdBackend, ""},
			DocPage{"ResourceControl", base + "resource-control.html", "resource-control.md", systemdBackend, ""},

			// Podman sections
			pm("Container"), pm("Volume"), pm("Network"), pm("Kube"),
			pm("Pod"), pm("Build"), pm("Image"),
		},
		SharedPages: map[string]string{
			"exec": "https://www.freedesktop.org/software/systemd/man/latest/systemd.exec.html",
			"kill": "https://www.freedesktop.org/software/systemd/man/latest/systemd.kill.html",
			"resource-control": "https://www.freedesktop.org/software/systemd/man/latest/systemd.resource-control.html",
		},
		SharedIncludes: map[string][]string{
			"Service": {"exec", "kill", "resource-control"},
			"Socket":  {"exec", "kill", "resource-control"},
			"Mount":   {"exec", "kill", "resource-control"},
			"Swap":    {"exec", "kill", "resource-control"},
			"Scope":   {"kill", "resource-control"},
			"Slice":   {"resource-control"},
		},
	}
}
