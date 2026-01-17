package main

import (
	"bytes"
	"fmt"
	"strings"

	md "github.com/JohannesKaufmann/html-to-markdown"
	"golang.org/x/net/html"
)

type SystemdBackend struct{}

func (s *SystemdBackend) ParseDirectives(htmlContent, filter string) ([]Directive, error) {
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
			text := getText(n)

			// Track sections from h2 or strong tags
			if n.Data == "h2" || n.Data == "strong" {
				trackSection(text, filter, &section, &inSection)
			}

			// Extract directives from dt tags
			if n.Data == "dt" && inSection {
				for _, match := range directivePattern.FindAllStringSubmatch(text, -1) {
					name := match[1]
					if seen[name] {
						continue
					}
					seen[name] = true

					desc := ""
					if dd := nextElement(n); dd != nil && dd.Data == "dd" {
						desc = toMarkdown(dd, converter)
						desc = strings.TrimPrefix(desc, "<dd>")
						desc = strings.TrimSuffix(desc, "</dd>")
						desc = strings.TrimSpace(desc)
					}

					directives = append(directives, Directive{Name: name, Description: desc})
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

func (s *SystemdBackend) ExtractDescription(htmlContent, _ string, converter *md.Converter) string {
	doc, err := parseHTML(htmlContent)
	if err != nil {
		return ""
	}

	var found *html.Node
	var walk func(*html.Node) bool
	walk = func(n *html.Node) bool {
		if n.Type == html.ElementNode && n.Data == "h2" {
			for _, attr := range n.Attr {
				if attr.Key == "id" && attr.Val == "Description" {
					var buf bytes.Buffer
					for elem := nextElement(n); elem != nil && elem.Data != "h2"; elem = nextElement(elem) {
						html.Render(&buf, elem)
					}
					found, _ = parseHTML(buf.String())
					return true
				}
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
	if found == nil {
		return ""
	}
	return toMarkdown(found, converter)
}

func (s *SystemdBackend) GenerateAttribution(url string) string {
	name := strings.TrimSuffix(strings.TrimPrefix(url, "https://www.freedesktop.org/software/systemd/man/latest/systemd."), ".html")
	return fmt.Sprintf("*Based on [systemd.%s(5)](https://www.freedesktop.org/software/systemd/man/systemd.%s.html) official documentation.*\n\n", name, name)
}
