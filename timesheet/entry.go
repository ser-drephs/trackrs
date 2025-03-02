package timesheet

import (
	"fmt"
	"time"
)

type Entry struct {
	Timestamp time.Time `json:"time"`
	Action    action    `json:"action"`
}

func NewRawEntry(start time.Time, action action) Entry {
	var entry Entry
	entry.Timestamp = start
	entry.Action = action
	return entry
}

func NewEntry(action action) Entry {
	return NewRawEntry(time.Now(), action)
}

func (e Entry) String() string {
	return fmt.Sprintf("{ time: %s, action: %s }", e.Timestamp.Format(time.RFC3339), e.Action)
}
