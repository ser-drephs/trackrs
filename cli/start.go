package cli

import (
	"bytes"
	"encoding/json"
	"errors"
	"os"
	"time"

	"github.com/rs/zerolog/log"
	"github.com/ser-drephs/tracker-go/timesheet"
	"github.com/spf13/cobra"
)

// startCmd represents the start command
var startCmd = &cobra.Command{
	Use:   "start",
	Short: "Start tracking work",
	Long:  `Starts tracking work for today.`,
	Run: func(cmd *cobra.Command, args []string) {
		runStart()
	},
}

func init() {
	rootCmd.AddCommand(startCmd)
}

func runStart() {
	log.Info().Msg("Start tracking work")
	var entry = timesheet.NewEntry(timesheet.Start)

	today := time.Now().Format(time.DateOnly) + ".json"
	file, err := os.ReadFile(today)
	var entries timesheet.Entries
	if err != nil && errors.Is(err, os.ErrNotExist) {
		log.Debug().Msg("File does not exist. Creating it.")
		entries = timesheet.NewEntries()
	} else if err != nil {
		log.Fatal().Msgf("Error reading file %s: %s", today, err)
	} else {
		err = json.Unmarshal(file, &entries)
		if err != nil {
			log.Fatal().Msg(err.Error())
			// TODO: test with emptry string and number for action
		}
		log.Debug().Msgf("Found file with version %d", entries.Version)
	}

	log.Debug().Msgf("Write data %v", entry)
	entries.Data = append(entries.Data, entry)

	jd, err := json.Marshal(entries)
	if err != nil {
		log.Error().Msgf("Error converting entries to json. %s", err)
	}

	var buffer bytes.Buffer
	json.Compact(&buffer, jd)
	err = os.WriteFile(today, buffer.Bytes(), 0644)
	if err != nil {
		log.Fatal().Msgf("Failed writing file '%s': %s", today, err)
	}
}
