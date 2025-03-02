package storage

import (
	"bytes"
	"encoding/json"
	"errors"
	"os"
	"strings"

	"github.com/rs/zerolog/log"
	"github.com/ser-drephs/tracker-go/timesheet"
)

type JsonProvider struct {
	Path string
}

func NewJsonProvider(path string) JsonProvider {
	var provider JsonProvider
	provider.Path = path
	return provider
}

func (j *JsonProvider) validatePath() {
	if !strings.Contains(j.Path, ".json") {
		j.Path = j.Path + ".json"
	}
}

func (j JsonProvider) Save(entries timesheet.Entries) error {
	j.validatePath()
	jd, err := json.Marshal(entries)
	if err != nil {
		return err
	}

	log.Debug().Msgf("Write entries to: %s", j.Path)
	log.Trace().Msgf("Entries to write: %s", jd)

	var buffer bytes.Buffer
	json.Compact(&buffer, jd)
	err = os.WriteFile(j.Path, buffer.Bytes(), 0644)
	if err != nil {
		return err
	}
	return nil
}

func (j JsonProvider) Read(entries *timesheet.Entries) error {
	j.validatePath()
	file, err := os.ReadFile(j.Path)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			log.Debug().Msg("File does not exist.")
		} else {
			return err
		}
	} else {
		err = json.Unmarshal(file, &entries)
		if err != nil {
			return err
			// TODO: test with emptry string and number for action
		}
		log.Debug().Msgf("Found file with version %d", entries.Version)
	}
	return nil
}
