package timesheet

import (
	"errors"
)

var ErrCalcNegativeDuration = errors.New("calculated duration is negative")
