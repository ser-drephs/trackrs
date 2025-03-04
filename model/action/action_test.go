package action

import (
	"bytes"
	"encoding/json"
	"testing"
)

func TestParseActionUnknownThrowsError(t *testing.T) {
	ut := "unknown"
	_, err := parseAction(ut)
	if err == nil {
		t.FailNow()
	}
}

func TestParseActionAllLower(t *testing.T) {
	ut := "start"
	res, err := parseAction(ut)
	if err != nil {
		t.FailNow()
	}
	if res != Start {
		t.FailNow()
	}
}

func TestParseActionAllCaps(t *testing.T) {
	ut := "END"
	res, err := parseAction(ut)
	if err != nil {
		t.FailNow()
	}
	if res != End {
		t.FailNow()
	}
}

func TestMarshalAction(t *testing.T) {
	ut := Break
	res, err := json.Marshal(ut)
	if err != nil {
		t.FailNow()
	}
	test := string(res[:])
	if test != "\"Break\"" {
		t.FailNow()
	}
}

func TestUnMarshalActionString(t *testing.T) {
	ut := bytes.NewBufferString("\"Break\"")
	var a Action
	err := json.Unmarshal(ut.Bytes(), &a)
	if err != nil {
		t.FailNow()
	}
	if a != Break {
		t.FailNow()
	}
}

func TestUnMarshalActionInteger(t *testing.T) {
	ut := bytes.NewBufferString("4")
	var a Action
	err := json.Unmarshal(ut.Bytes(), &a)
	if err != nil {
		t.FailNow()
	}
	if a != End {
		t.FailNow()
	}
}

func TestUnMarshalActionStringUnknown(t *testing.T) {
	ut := bytes.NewBufferString("\"Whatever\"")
	var a Action
	err := json.Unmarshal(ut.Bytes(), &a)
	if err == nil {
		t.FailNow()
	}
}

func TestUnMarshalActionIntegerUnknown(t *testing.T) {
	ut := bytes.NewBufferString("19")
	var a Action
	err := json.Unmarshal(ut.Bytes(), &a)
	if err != nil {
		// limitation - accepted that it does not fail
		t.FailNow()
	}
	if a != 19 {
		t.FailNow()
	}
}
