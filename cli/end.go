package cli

import (
	"github.com/rs/zerolog/log"
	"github.com/ser-drephs/tracker-go/common"
	"github.com/ser-drephs/tracker-go/timesheet"
	"github.com/spf13/cobra"
)

// endCmd represents the end command
var endCmd = &cobra.Command{
	Use:   "end",
	Short: "End tracking work",
	Long:  `End tracking work for today.`,
	Run: func(cmd *cobra.Command, args []string) {
		runEnd()
	},
}

func init() {
	rootCmd.AddCommand(endCmd)
}

func runEnd() {
	log.Trace().Msg("Execute end")
	var entry = timesheet.NewEntry(timesheet.End)
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
