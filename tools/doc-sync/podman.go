package main

import (
	"fmt"
	"strings"

	md "github.com/JohannesKaufmann/html-to-markdown"
	"golang.org/x/net/html"
)

type PodmanBackend struct{}

func (p *PodmanBackend) ParseDirectives(htmlContent, filter string) ([]Directive, error) {
	doc, err := parseHTML(htmlContent)
	if err != nil {
		return nil, err
	}

	converter := md.NewConverter("", true, nil)
	var directives []Directive
	seen := make(map[string]bool)
	section := ""
	inSection := filter == ""

	var walk func(*html.Node)
	walk = func(n *html.Node) {
		if n.Type == html.ElementNode {
			// Track sections from h1/h2 tags
			if n.Data == "h1" || n.Data == "h2" {
				trackSection(getText(n), filter, &section, &inSection)
			}

			// Extract directives from h2 > code tags
			if n.Data == "h2" && inSection {
				for c := n.FirstChild; c != nil; c = c.NextSibling {
					if c.Type == html.ElementNode && c.Data == "code" {
						if matches := directivePattern.FindStringSubmatch(getText(c)); len(matches) > 1 {
							name := matches[1]
							if seen[name] {
								break
							}
							seen[name] = true

							var parts []string
							for elem := nextElement(n); elem != nil; elem = nextElement(elem) {
								if elem.Data == "h2" || elem.Data == "h3" {
									break
								}
								if elem.Data == "p" || elem.Data == "ul" || elem.Data == "ol" ||
									elem.Data == "dl" || elem.Data == "pre" || elem.Data == "table" ||
									elem.Data == "div" || elem.Data == "blockquote" {
									if md := toMarkdown(elem, converter); md != "" {
										parts = append(parts, md)
									}
								}
							}

							directives = append(directives, Directive{
								Name:        name,
								Description: strings.Join(parts, "\n\n"),
							})
						}
						break
					}
				}
			}
		}

		for c := n.FirstChild; c != nil; c = c.NextSibling {
			walk(c)
		}
	}

	walk(doc)
	return directives, nil
}

func (p *PodmanBackend) ExtractDescription(htmlContent, sectionName string, converter *md.Converter) string {
	doc, err := parseHTML(htmlContent)
	if err != nil {
		return ""
	}

	var parts []string
	var walk func(*html.Node) bool
	walk = func(n *html.Node) bool {
		if n.Type == html.ElementNode && (n.Data == "h1" || n.Data == "h2" || n.Data == "h3") {
			if strings.Contains(getText(n), "["+sectionName+"]") {
				for elem := nextElement(n); elem != nil; elem = nextElement(elem) {
					// Stop at directive headings
					if elem.Data == "h2" {
						for c := elem.FirstChild; c != nil; c = c.NextSibling {
							if c.Type == html.ElementNode && c.Data == "code" && strings.Contains(getText(c), "=") {
								return true
							}
						}
					}
					// Stop at "Valid options for" text
					if elem.Data == "p" && strings.Contains(getText(elem), "Valid options for") {
						parts = append(parts, toMarkdown(elem, converter))
						return true
					}
					// Collect intro content
					if elem.Data == "p" || elem.Data == "ul" || elem.Data == "ol" ||
						elem.Data == "pre" || elem.Data == "dl" || elem.Data == "blockquote" {
						parts = append(parts, toMarkdown(elem, converter))
					}
				}
				return true
			}
		}
		for c := n.FirstChild; c != nil; c = c.NextSibling {
			if walk(c) {
				return true
			}
		}
		return false
	}

	walk(doc)
	return strings.Join(parts, "\n\n")
}

func (p *PodmanBackend) GenerateAttribution(url string) string {
	return fmt.Sprintf("*Based on [podman-systemd.unit(5)](%s) official documentation.*\n\n", url)
}
