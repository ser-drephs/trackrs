package cli

import (
	"github.com/rs/zerolog/log"
	"github.com/ser-drephs/tracker-go/model/action"
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
	if err := timesheet.Append(action.Break); err != nil {
		log.Error().Msgf("Error executing break: %s", err)
	}
}
