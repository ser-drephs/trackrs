package pers

import (
	"encoding/json"
	"os"

	"github.com/ser-drephs/tracker-go/timesheet"
)

type JsonPersistence struct {
	parent Persistence
}

func (j JsonPersistence) ReadFile(path string) (*timesheet.Entries, error) {
	var entries *timesheet.Entries

	var dat, err = os.ReadFile(path)
	if err != nil {
		return entries, err
	}

	err = json.Unmarshal(dat, &entries)
	if err != nil {
		return entries, err
	}

	return entries, err
}
