package model

import (
	"fmt"
	"strings"
)

type Entries struct {
	Data    []Entry `json:"data"`
	Version uint8   `json:"version"`
}

func NewEntries() Entries {
	var entries Entries
	entries.Version = 1
	return entries
}

func (e Entries) String() string {
	var data []string
	for _, element := range e.Data {
		data = append(data, element.String())
	}
	return fmt.Sprintf("[ %s ]", strings.Join(data, ", "))
}

// func (e Entries) MarshalJSON() (text []byte, err error) {
// 	var buf bytes.Buffer
// 	for _, element := range e.Data {
// 		b, _ := json.Marshal(element)
// 		buf.Write(b)
// 	}
// 	return buf.Bytes(), nil
// }

// func (e *Entries) UnmarshalJSON(text []byte) error {

// }
