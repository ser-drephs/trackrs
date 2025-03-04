package storage

import (
	"errors"
)

var (
	ErrJsonEntriesRead  = errors.New("unable to read json file")
	ErrJsonMarshal      = errors.New("unable to marshal entries from json file")
	ErrJsonEntriesWrite = errors.New("unable to write file")
	ErrJsonUnMarshal    = errors.New("unable to unmarshal entries to json")
)
