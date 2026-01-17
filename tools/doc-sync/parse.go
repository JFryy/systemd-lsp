package main

import (
	"bytes"
	"regexp"
	"strings"

	md "github.com/JohannesKaufmann/html-to-markdown"
	"golang.org/x/net/html"
)

var directivePattern = regexp.MustCompile(`([A-Z][A-Za-z0-9]+)=`)
var sectionPattern = regexp.MustCompile(`\[([A-Z][a-z]+)\]`)

func parseHTML(content string) (*html.Node, error) {
	return html.Parse(strings.NewReader(content))
}

func getText(n *html.Node) string {
	if n.Type == html.TextNode {
		return n.Data
	}
	var buf strings.Builder
	for c := n.FirstChild; c != nil; c = c.NextSibling {
		buf.WriteString(getText(c))
	}
	return buf.String()
}

func toMarkdown(n *html.Node, converter *md.Converter) string {
	var buf bytes.Buffer
	html.Render(&buf, n)
	md, err := converter.ConvertString(buf.String())
	if err != nil {
		return getText(n)
	}
	return strings.TrimSpace(md)
}

func trackSection(text, filter string, current *string, inSection *bool) {
	if matches := sectionPattern.FindStringSubmatch(text); len(matches) > 1 {
		*current = matches[1]
		*inSection = filter == "" || *current == filter
	}
}

func nextElement(n *html.Node) *html.Node {
	for next := n.NextSibling; next != nil; next = next.NextSibling {
		if next.Type == html.ElementNode {
			return next
		}
	}
	return nil
}
