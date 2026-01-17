package main

import (
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
)

func fetchHTML(url string) (string, error) {
	resp, err := http.Get(url)
	if err != nil {
		return "", err
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("HTTP %d: %s", resp.StatusCode, resp.Status)
	}

	content, err := io.ReadAll(resp.Body)
	if err != nil {
		return "", err
	}
	return string(content), nil
}

func directivesToNames(directives []Directive) []string {
	names := make([]string, len(directives))
	for i, d := range directives {
		names[i] = d.Name
	}
	return names
}

func writeDirectivesList(outputDir, section string, directives []string) error {
	path := filepath.Join(outputDir, "directives", strings.ToLower(section)+".txt")
	if err := os.MkdirAll(filepath.Dir(path), 0755); err != nil {
		return err
	}
	return os.WriteFile(path, []byte(strings.Join(directives, "\n")), 0644)
}

func generateMarkdownDoc(section string, directives []Directive, backend Backend, sourceURL, description string) string {
	var doc strings.Builder
	doc.WriteString(fmt.Sprintf("# [%s] Section\n\n", section))

	if description != "" {
		doc.WriteString(description)
		doc.WriteString("\n\n")
	}

	doc.WriteString(backend.GenerateAttribution(sourceURL))

	for _, d := range directives {
		doc.WriteString(fmt.Sprintf("### %s=\n\n", d.Name))
		if d.Description != "" {
			doc.WriteString(strings.TrimSpace(d.Description))
			doc.WriteString("\n\n")
		} else {
			doc.WriteString("*No description available*\n\n")
		}
	}
	return doc.String()
}

func writeMarkdownDoc(outputDir, filename, content string) error {
	return os.WriteFile(filepath.Join(outputDir, filename), []byte(content), 0644)
}

func mergeDirectives(base, add []Directive) []Directive {
	seen := make(map[string]bool)
	for _, d := range base {
		seen[d.Name] = true
	}
	result := append([]Directive{}, base...)
	for _, d := range add {
		if !seen[d.Name] {
			result = append(result, d)
			seen[d.Name] = true
		}
	}
	return result
}

func loadExistingDirectives(outputDir, pageName string) ([]string, error) {
	data, err := os.ReadFile(filepath.Join(outputDir, "directives", strings.ToLower(pageName)+".txt"))
	if err != nil {
		return nil, err
	}
	var result []string
	for _, line := range strings.Split(string(data), "\n") {
		if line = strings.TrimSpace(line); line != "" {
			result = append(result, line)
		}
	}
	return result, nil
}

func loadExistingMarkdown(outputDir, filename string) (string, error) {
	data, err := os.ReadFile(filepath.Join(outputDir, filename))
	return string(data), err
}
