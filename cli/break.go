package cli

import (
	"github.com/rs/zerolog/log"
	"github.com/spf13/cobra"
)

// breakCmd represents the break command
var breakCmd = &cobra.Command{
	Use:   "break",
	Short: "Take a break",
	Long:  `Breaks current tracking.`,
	Run: func(cmd *cobra.Command, args []string) {
		log.Fatal().Msg("Not yet implemented")
	},
}

func init() {
	rootCmd.AddCommand(breakCmd)
}
