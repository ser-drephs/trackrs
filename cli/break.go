package cli

import (
	"github.com/rs/zerolog/log"
	"github.com/ser-drephs/tracker-go/common"
	"github.com/ser-drephs/tracker-go/timesheet"
	"github.com/spf13/cobra"
)

// breakCmd represents the break command
var breakCmd = &cobra.Command{
	Use:   "break",
	Short: "Take a break",
	Long:  `Breaks current tracking.`,
	Run: func(cmd *cobra.Command, args []string) {
		runBreak()
	},
}

func init() {
	rootCmd.AddCommand(breakCmd)
}

func runBreak() {
	log.Trace().Msg("Execute break")
	var entry = timesheet.NewEntry(timesheet.Break)
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
