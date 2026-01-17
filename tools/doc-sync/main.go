package main

import (
	"crypto/sha256"
	"encoding/hex"
	"flag"
	"fmt"
	"os"
	"sort"

	md "github.com/JohannesKaufmann/html-to-markdown"
)

var (
	systemdBackend = &SystemdBackend{}
	podmanBackend  = &PodmanBackend{}
	config         = NewConfig(systemdBackend, podmanBackend)
)

const (
	red   = "\033[31m"
	green = "\033[32m"
	reset = "\033[0m"
)

func main() {
	generate := flag.Bool("generate", false, "Generate and write documentation files (default: check only)")
	flag.Parse()

	mode := "CHECK"
	if *generate {
		mode = "GENERATE"
	}
	fmt.Printf("Starting doc-sync (%s MODE) for %d pages\n", mode, len(config.Pages))

	success, fail, mismatch := 0, 0, 0

	for i, page := range config.Pages {
		fmt.Printf("[%d/%d] Processing: %s\n", i+1, len(config.Pages), page.Name)

		htmlContent, err := fetchHTML(page.URL)
		if err != nil {
			fmt.Printf("%sError fetching %s: %v%s\n", red, page.Name, err, reset)
			fail++
			continue
		}

		directives, err := page.Backend.ParseDirectives(htmlContent, page.SectionFilter)
		if err != nil {
			fmt.Printf("%sError parsing directives for %s: %v%s\n", red, page.Name, err, reset)
			fail++
			continue
		}

		// Generate markdown from base directives only (NO merging for markdown)
		converter := md.NewConverter("", true, nil)
		desc := page.Backend.ExtractDescription(htmlContent, page.Name, converter)
		markdown := generateMarkdownDoc(page.Name, directives, page.Backend, page.URL, desc)

		// For directive name list (.txt files), merge shared directives for validation
		// This ensures LSP knows which directives are valid in each section
		directivesForValidation := append([]Directive{}, directives...)
		if includes, ok := config.SharedIncludes[page.Name]; ok {
			for _, sharedPage := range includes {
				if url, ok := config.SharedPages[sharedPage]; ok {
					if html, err := fetchHTML(url); err == nil {
						if shared, err := systemdBackend.ParseDirectives(html, ""); err == nil {
							directivesForValidation = mergeDirectives(directivesForValidation, shared)
							fmt.Printf("  + Merged %d directive names from %s (for validation)\n", len(shared), sharedPage)
						}
					}
				}
			}
		}

		names := directivesToNames(directivesForValidation)
		sort.Strings(names)

		dirHash := hash(names)
		mdHash := hash(markdown)

		if *generate {
			if err := writeDirectivesList(config.OutputDir, page.Name, names); err != nil {
				fmt.Printf("%sError writing directives list: %v%s\n", red, err, reset)
				fail++
				continue
			}
			if err := writeMarkdownDoc(config.OutputDir, page.Filename, markdown); err != nil {
				fmt.Printf("%sError writing markdown: %v%s\n", red, err, reset)
				fail++
				continue
			}
			fmt.Printf("%s  Generated (directives: %s, markdown: %s)%s\n", green, dirHash, mdHash, reset)
			success++
		} else {
			existingDir, err1 := loadExistingDirectives(config.OutputDir, page.Name)
			existingMd, err2 := loadExistingMarkdown(config.OutputDir, page.Filename)

			if err1 != nil || err2 != nil {
				fmt.Printf("%s  Missing baseline files%s\n", red, reset)
				mismatch++
				continue
			}

			sort.Strings(existingDir)
			hasMismatch := false

			if dirHash != hash(existingDir) {
				fmt.Printf("%s  DIRECTIVES MISMATCH%s\n", red, reset)
				fmt.Printf("    Expected: %s (%d directives)\n", hash(existingDir), len(existingDir))
				fmt.Printf("    Got:      %s (%d directives)\n", dirHash, len(names))
				hasMismatch = true
			}

			if mdHash != hash(existingMd) {
				fmt.Printf("%s  MARKDOWN MISMATCH%s\n", red, reset)
				fmt.Printf("    Expected: %s\n", hash(existingMd))
				fmt.Printf("    Got:      %s\n", mdHash)
				hasMismatch = true
			}

			if hasMismatch {
				mismatch++
			} else {
				fmt.Printf("%s  OK (directives: %s, markdown: %s)%s\n", green, dirHash, mdHash, reset)
				success++
			}
		}
	}

	fmt.Printf("\n=== Summary ===\n")
	if *generate {
		fmt.Printf("%sGenerated: %d%s\n", green, success, reset)
		if fail > 0 {
			fmt.Printf("%sFailed: %d%s\n", red, fail, reset)
		}
	} else {
		fmt.Printf("%sPassed: %d%s\n", green, success, reset)
		if mismatch > 0 {
			fmt.Printf("%sMismatches: %d%s\n", red, mismatch, reset)
		}
		if fail > 0 {
			fmt.Printf("%sFailed: %d%s\n", red, fail, reset)
		}
	}

	if fail > 0 || (!*generate && mismatch > 0) {
		fmt.Printf("\n%sFailed%s\n", red, reset)
		os.Exit(1)
	}
}

func hash(v interface{}) string {
	var data []byte
	switch v := v.(type) {
	case []string:
		for _, s := range v {
			data = append(data, []byte(s+"\n")...)
		}
	case string:
		data = []byte(v)
	}
	h := sha256.Sum256(data)
	return hex.EncodeToString(h[:])[:16]
}
