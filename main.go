package main

import (
	"github.com/pterm/pterm"
	"github.com/pterm/pterm/putils"
)

func main() {
	pterm.EnableDebugMessages()
	pterm.Println()
	pterm.DefaultBigText.WithLetters(
		putils.LettersFromStringWithStyle("CR", pterm.FgBlue.ToStyle()),
		putils.LettersFromStringWithStyle("ISP", pterm.FgYellow.ToStyle())).Render()
	for {
		input, _ := pterm.DefaultInteractiveTextInput.WithDelimiter(" -> ").Show("λ")
		if input == "exit" {
			pterm.Info.Println("Quitting...")
			break
		}
		tokens := tokenize(input)
		pterm.Debug.Println("Tokens:", tokens)
		parse(tokens)
		pterm.Println()
	}
}
