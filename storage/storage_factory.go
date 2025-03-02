package storage

import (
	"errors"
	"fmt"
	"time"
)

type UnsupportedStorageError struct {
	Err error
}

func (e *UnsupportedStorageError) Error() string {
	return fmt.Sprintf("Unsupported storage provider '%v'", e.Err)
}

func GetStorage(storageType string) (Provider, error) {
	switch storageType {
	case "json":
		return JsonProvider{Path: time.Now().Format(time.DateOnly) + ".json"}, nil
	}
	return nil, &UnsupportedStorageError{Err: errors.New(storageType)}
}
