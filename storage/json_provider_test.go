package storage

import (
	"strings"
	"testing"
)

func TestValidatePathAppendsJson(t *testing.T) {
	provider := JsonProvider{
		Path: "test",
	}
	provider.validatePath()

	if !strings.HasSuffix(provider.Path, "json") {
		// t.Fatalf("expected json path to be appended with json")
		t.FailNow()
	}
}

func TestValidatePathAlreadyHasJson(t *testing.T) {
	provider := JsonProvider{
		Path: "test.json",
	}
	provider.validatePath()

	if !strings.HasSuffix(provider.Path, "json") {
		// t.Fatalf("expected json path to be appended with json")
		t.FailNow()
	}
}
