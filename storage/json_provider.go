package storage

import (
	"bytes"
	"encoding/json"
	"errors"
	"os"
	"strings"

	"github.com/rs/zerolog/log"
	"github.com/ser-drephs/tracker-go/model"
)

type JsonProvider struct {
	path string
}

func (j *JsonProvider) validatePath() {
	if !strings.Contains(j.path, ".json") {
		j.path = j.path + ".json"
	}
}

func (j JsonProvider) Save(entries model.Entries) error {
	j.validatePath()
	jd, err := json.Marshal(entries)
	if err != nil {
		return ErrJsonMarshal
	}

	log.Debug().Msgf("Write entries to: %s", j.path)
	log.Trace().Msgf("Entries to write: %s", jd)

	var buffer bytes.Buffer
	json.Compact(&buffer, jd)
	err = os.WriteFile(j.path, buffer.Bytes(), 0644)
	if err != nil {
		return ErrJsonEntriesWrite
	}
	return nil
}

func (j JsonProvider) Read(entries *model.Entries) error {
	j.validatePath()
	file, err := os.ReadFile(j.path)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			log.Debug().Msg("File does not exist.")
		} else {
			return ErrJsonEntriesRead
		}
	} else {
		err = json.Unmarshal(file, &entries)
		if err != nil {
			return ErrJsonUnMarshal
		}
		log.Debug().Msgf("Found file with version %d", entries.Version)
	}
	return nil
}
