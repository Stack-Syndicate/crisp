package main

import (
	"github.com/bzick/tokenizer"
)

const (
	TRPAREN tokenizer.TokenKey = iota + 1
	TLPAREN
	TOPERATOR
	TIDENTIFIER
	TSTRING
)

type TokenPair struct {
	Type  string
	Value string
}

func tokenize(input string) []TokenPair {
	parser := tokenizer.New()
	parser.DefineTokens(TRPAREN, []string{"("})
	parser.DefineTokens(TLPAREN, []string{")"})
	parser.DefineTokens(TOPERATOR, []string{"<=", ">=", "<", ">", "==", "+", "-", "*", "/"})
	parser.DefineStringToken(TSTRING, `"`, `"`).SetEscapeSymbol(tokenizer.BackSlash)
	parser.AllowKeywordSymbols(tokenizer.Underscore, tokenizer.Numbers)

	stream := parser.ParseString(input)
	defer stream.Close()
	var tokenPairs []TokenPair
	for stream.IsValid() {
		var tok = stream.CurrentToken()
		var typ string
		switch tok.Key() {
		case TRPAREN:
			typ = "RPAREN"
		case TLPAREN:
			typ = "LPAREN"
		case TOPERATOR:
			typ = "OPERATOR"
		case tokenizer.TokenFloat:
			typ = "FLOAT"
		case tokenizer.TokenKeyword:
			typ = "IDENTIFIER"
		case tokenizer.TokenInteger:
			typ = "INTEGER"
		case tokenizer.TokenString:
			typ = "STRING"
		default:
			typ = "MISC"
		}
		tokenPairs = append(tokenPairs, TokenPair{
			Type:  typ,
			Value: tok.ValueUnescapedString(),
		})
		stream.GoNext()
	}
	return tokenPairs
}
