package cli

import (
	"github.com/rs/zerolog/log"
	"github.com/ser-drephs/tracker-go/model/action"
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
	if err := timesheet.Append(action.End); err != nil {
		log.Error().Msgf("Error executing end: %s", err)
	}
}
