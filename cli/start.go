package cli

import (
	"github.com/rs/zerolog/log"
	"github.com/ser-drephs/tracker-go/model/action"
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
	if err := timesheet.Append(action.Start); err != nil {
		log.Error().Msgf("Error executing start: %s", err)
	}
}
