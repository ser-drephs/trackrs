/*
Copyright © 2025 ser-drephs
*/
package cmd

import (
	"github.com/rs/zerolog/log"
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
}
