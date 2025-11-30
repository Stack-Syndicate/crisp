package main

import (
	"github.com/pterm/pterm"
)

type Expr interface{}

type Atom struct {
	Value string
}

type List struct {
	Elements []Expr
}

func parse(tokenPairs []TokenPair) {
	if !sanityCheck(tokenPairs) {
		panic("Sanity check failed.")
	}
}

func sanityCheck(tokenPairs []TokenPair) bool {
	var isOk = true
	if len(tokenPairs) == 0 {
		pterm.Fatal.Println("No tokens were sent for parsing.")
		isOk = false
	}
	var parenParity = 0
	for _, pair := range tokenPairs {
		switch pair.Type {
		case "RPAREN":
			parenParity++
		case "LPAREN":
			parenParity--
		}
	}
	if parenParity != 0 {
		pterm.Fatal.Println("Unmatched parantheses detected.")
		isOk = false
	}
	return isOk
}
