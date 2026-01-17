package main

import (
	md "github.com/JohannesKaufmann/html-to-markdown"
)

// Directive represents a configuration directive with its documentation
type Directive struct {
	Name        string
	Description string
}

// Backend defines the interface for different documentation sources
type Backend interface {
	ParseDirectives(htmlContent string, sectionFilter string) ([]Directive, error)
	ExtractDescription(htmlContent string, sectionName string, converter *md.Converter) string
	GenerateAttribution(sourceURL string) string
}

// DocPage represents a documentation page to be processed
type DocPage struct {
	Name          string
	URL           string
	Filename      string
	Backend       Backend
	SectionFilter string // extract only directives from this section (e.g., "Install")
}
