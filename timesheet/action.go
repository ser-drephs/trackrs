package timesheet

import (
	"encoding/json"
	"fmt"
	"strings"
)

type action uint8

const (
	None action = iota + 1
	Start
	Break
	End
)

var (
	action_name = map[uint8]string{
		1: "None",
		2: "Start",
		3: "Break",
		4: "End",
	}
	action_value = map[string]uint8{
		"none":  1,
		"start": 2,
		"break": 3,
		"end":   4,
	}
)

func (a action) String() string {
	return action_name[uint8(a)]
}

func parseAction(s string) (action, error) {
	s = strings.TrimSpace(strings.ToLower(s))
	value, ok := action_value[s]
	if !ok {
		return action(0), fmt.Errorf("%q is not a valid action", s)
	}
	return action(value), nil
}

func (a action) MarshalJSON() ([]byte, error) {
	return json.Marshal(a.String())
}

func (a *action) UnmarshalJSON(data []byte) (err error) {
	var actionText string
	if err := json.Unmarshal(data, &actionText); err != nil {
		var actionNumber int
		if err := json.Unmarshal(data, &actionNumber); err != nil {
			return err
		}
		*a = action(actionNumber)
		return nil
	}
	if *a, err = parseAction(actionText); err != nil {
		return err
	}
	return nil
}
