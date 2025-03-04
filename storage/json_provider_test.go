package storage

import (
	"bytes"
	"errors"
	"os"
	"strings"
	"testing"

	"github.com/ser-drephs/tracker-go/model"
	"github.com/ser-drephs/tracker-go/model/action"
)

func TestValidatePathAppendsJson(t *testing.T) {
	provider := JsonProvider{
		path: "test",
	}
	provider.validatePath()

	if !strings.HasSuffix(provider.path, "json") {
		t.FailNow()
	}
}

func TestValidatePathAlreadyHasJson(t *testing.T) {
	provider := JsonProvider{
		path: "test.json",
	}
	provider.validatePath()

	if !strings.HasSuffix(provider.path, "json") {
		t.FailNow()
	}
}

func TestWriteAndReadJsonFile(t *testing.T) {
	jsonUt := t.TempDir() + "/ut.json"
	if _, err := os.Stat(jsonUt); err == nil {
		t.Fatalf("File '%s' already exists!", jsonUt)
	}
	provier := JsonProvider{
		path: jsonUt,
	}
	entries := model.NewEntries()
	entries.Data = append(entries.Data, model.NewEntry(action.Start), model.NewEntry(action.Break), model.NewEntry(action.Start))
	provier.Save(entries)
	if _, err := os.Stat(jsonUt); err != nil {
		t.FailNow()
	}

	var readEntries model.Entries
	provier.Read(&readEntries)
	if len(readEntries.Data) < 3 {
		t.FailNow()
	}
}

func TestReadNotExistingJsonFileNoError(t *testing.T) {
	jsonUt := t.TempDir() + "/ut.json"
	if _, err := os.Stat(jsonUt); err == nil {
		t.Fatalf("File '%s' already exists!", jsonUt)
	}
	provier := JsonProvider{
		path: jsonUt,
	}
	var readEntries model.Entries
	if err := provier.Read(&readEntries); err != nil {
		t.FailNow()
	}
	if len(readEntries.Data) != 0 {
		t.FailNow()
	}
}

func TestReadMalformedJsonFileThrowsError(t *testing.T) {
	jsonUt := t.TempDir() + "/ut.json"
	sampleText := "[{\"id\":1,\"status\":\"Connect\"}]"
	buffer := bytes.NewBufferString(sampleText)
	if err := os.WriteFile(jsonUt, buffer.Bytes(), 0644); err != nil {
		t.Fatalf("Can not save test data> %s", err)
	}
	provier := JsonProvider{
		path: jsonUt,
	}
	var readEntries model.Entries
	if err := provier.Read(&readEntries); err == nil || !errors.Is(err, ErrJsonUnMarshal) {
		t.FailNow()
	}
}
