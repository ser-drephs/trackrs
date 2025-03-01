package cli

import (
	"github.com/rs/zerolog/log"
	"github.com/spf13/cobra"
)

// endCmd represents the end command
var endCmd = &cobra.Command{
	Use:   "end",
	Short: "End tracking work",
	Long:  `End tracking work for today.`,
	Run: func(cmd *cobra.Command, args []string) {
		log.Fatal().Msg("Not yet implemented")
	},
}

func init() {
	rootCmd.AddCommand(endCmd)
}
