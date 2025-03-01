package pers

import (
	"time"

	"github.com/ser-drephs/tracker-go/timesheet"
)

type Persistence interface {
	Read(date time.Time) (timesheet.Entries, error)
	ReadFile(path string) (timesheet.Entries, error)
	Write([]byte)
}
