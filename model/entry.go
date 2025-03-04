package model

import (
	"fmt"
	"time"

	"github.com/ser-drephs/tracker-go/model/action"
)

type Entry struct {
	Timestamp time.Time     `json:"time"`
	Action    action.Action `json:"action"`
}

func NewRawEntry(start time.Time, action action.Action) Entry {
	var entry Entry
	entry.Timestamp = start
	entry.Action = action
	return entry
}

func NewEntry(action action.Action) Entry {
	return NewRawEntry(time.Now(), action)
}

func (e Entry) String() string {
	return fmt.Sprintf("{ time: %s, action: %s }", e.Timestamp.Format(time.RFC3339), e.Action)
}
