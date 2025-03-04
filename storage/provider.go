package storage

import "github.com/ser-drephs/tracker-go/model"

type Provider interface {
	Save(entries model.Entries) error
	Read(entries *model.Entries) error
}
