package common

import (
	"github.com/ser-drephs/tracker-go/storage"
)

var Storage storage.Provider

func NewCommon() error {
	storage, err := storage.GetStorage("json")
	if err != nil {
		return err
	}
	Storage = storage

	return nil
}
