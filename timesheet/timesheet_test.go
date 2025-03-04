package timesheet

import (
	"errors"
	"testing"
	"time"

	"github.com/ser-drephs/tracker-go/model"
	"github.com/ser-drephs/tracker-go/model/action"
)

func TestDifferenceCalculation(t *testing.T) {
	leftTime, _ := time.Parse(time.RFC3339, "2025-03-04T20:30:00.661385908Z")
	rightTime, _ := time.Parse(time.RFC3339, "2025-03-04T20:45:00.661385908Z")
	expectedDuration, _ := time.ParseDuration("15m")

	diff, err := Difference(&model.Entry{Timestamp: leftTime, Action: action.Start}, &model.Entry{Timestamp: rightTime, Action: action.Break})
	if err != nil {
		t.FailNow()
	}

	if diff.Abs().Milliseconds() != expectedDuration.Abs().Milliseconds() {
		t.FailNow()
	}
}

func TestDifferenceCalculationNegative(t *testing.T) {
	leftTime, _ := time.Parse(time.RFC3339, "2025-03-04T20:30:00.661385908Z")
	rightTime, _ := time.Parse(time.RFC3339, "2025-03-04T20:45:00.661385908Z")

	_, err := Difference(&model.Entry{Timestamp: rightTime, Action: action.Break}, &model.Entry{Timestamp: leftTime, Action: action.Start})
	if err == nil || !errors.Is(err, ErrCalcNegativeDuration) {
		t.FailNow()
	}
}
