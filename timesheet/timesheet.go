package timesheet

import (
	"time"

	"github.com/ser-drephs/tracker-go/common"
	"github.com/ser-drephs/tracker-go/model"
	"github.com/ser-drephs/tracker-go/model/action"
)

func Append(action action.Action) error {
	var entry = model.NewEntry(action)
	var entries model.Entries

	if err := common.Storage.Read(&entries); err != nil {
		return err
	}
	entries.Data = append(entries.Data, entry)

	if err := common.Storage.Save(entries); err != nil {
		return err
	}
	return nil
}

func Difference(left *model.Entry, right *model.Entry) (*time.Duration, error) {
	difference := right.Timestamp.Sub(left.Timestamp)
	if difference.Seconds() < 0 {
		return nil, ErrCalcNegativeDuration
	}
	return &difference, nil
}
