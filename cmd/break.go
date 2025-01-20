/*
Copyright © 2025 ser-drephs
*/
package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

// breakCmd represents the break command
var breakCmd = &cobra.Command{
	Use:   "break",
	Short: "Take a break",
	Long:  `Breaks current tracking.`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("break called")
	},
}

func init() {
	rootCmd.AddCommand(breakCmd)
}
