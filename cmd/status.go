/*
Copyright © 2025 ser-drephs
*/
package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

// statusCmd represents the status command
var statusCmd = &cobra.Command{
	Use:   "status",
	Short: "Get the status of current tracking",
	Long:  `Get the status for either a day or a week. Not providing additional options will return status for today.`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("status called")
	},
}

func init() {
	rootCmd.AddCommand(statusCmd)
	statusCmd.Flags().BoolP("table", "t", false, "Format week status as table.")
	statusCmd.Flags().Int8P("week", "w", 0, "Week to show the status for. Either enter the correct week of the year or a relative value eg. -1")
}
