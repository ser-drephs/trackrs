package cli

import (
	"log"
	"os"

	// log "github.com/sirupsen/logrus"

	"github.com/ser-drephs/tracker-go/common"
	"github.com/spf13/cobra"
)

var v int

// rootCmd represents the base command when called without any subcommands
var rootCmd = &cobra.Command{
	Use:   "tracker-go",
	Short: "Time Tracker CLI",
}

// Execute adds all child commands to the root command and sets flags appropriately.
// This is called by main.main(). It only needs to happen once to the rootCmd.
func Execute() {
	rootCmd.PersistentPreRunE = func(cmd *cobra.Command, args []string) error {
		common.SetLoggerLevel(v)
		return nil
	}
	rootCmd.PersistentFlags().CountVarP(&v, "verbose", "v", "counted verbosity")

	if err := common.NewCommon(); err != nil {
		log.Fatalf("Error on initialization: %s", err)
	}
	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}
