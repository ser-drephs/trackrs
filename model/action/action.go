package action

import (
	"encoding/json"
	"fmt"
	"strings"
)

type Action uint8

const (
	None Action = iota + 1
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

func (a Action) String() string {
	return action_name[uint8(a)]
}

func parseAction(s string) (Action, error) {
	s = strings.TrimSpace(strings.ToLower(s))
	value, ok := action_value[s]
	if !ok {
		return Action(0), fmt.Errorf("%q is not a valid action", s)
	}
	return Action(value), nil
}

func (a Action) MarshalJSON() ([]byte, error) {
	return json.Marshal(a.String())
}

func (a *Action) UnmarshalJSON(data []byte) (err error) {
	var actionText string
	if err := json.Unmarshal(data, &actionText); err != nil {
		var actionNumber int
		if err := json.Unmarshal(data, &actionNumber); err != nil {
			return err
		}
		*a = Action(actionNumber)
		return nil
	}
	if *a, err = parseAction(actionText); err != nil {
		return err
	}
	return nil
}
