/*
Copyright © 2025 ser-drephs
*/
package cmd

import (
	"fmt"

	"github.com/spf13/cobra"
)

// endCmd represents the end command
var endCmd = &cobra.Command{
	Use:   "end",
	Short: "End tracking work",
	Long:  `End tracking work for today.`,
	Run: func(cmd *cobra.Command, args []string) {
		fmt.Println("end called")
	},
}

func init() {
	rootCmd.AddCommand(endCmd)
}
