package cli

import (
	"github.com/rs/zerolog/log"
	"github.com/ser-drephs/tracker-go/common"
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
	log.Trace().Msg("Execute start")
	var entry = timesheet.NewEntry(timesheet.Start)
	var entries timesheet.Entries

	if err := common.Storage.Read(&entries); err != nil {
		log.Error().Msgf("Error on reading entries: %s", err)
	}

	log.Debug().Msgf("Add entry: %s", entry)
	entries.Data = append(entries.Data, entry)

	if err := common.Storage.Save(entries); err != nil {
		log.Error().Msgf("Error on saving entries: %s", err)
	}
}
