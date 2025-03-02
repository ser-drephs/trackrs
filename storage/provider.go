package storage

import "github.com/ser-drephs/tracker-go/timesheet"

type Provider interface {
	Save(entries timesheet.Entries) error
	Read(entries *timesheet.Entries) error
}
