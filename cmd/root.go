/*
Copyright © 2025 ser-drephs
*/
package cmd

import (
	"io"
	"os"
	"time"

	// log "github.com/sirupsen/logrus"
	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
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
	loggerSetup()
	err := rootCmd.Execute()
	if err != nil {
		os.Exit(1)
	}
}

func loggerSetup() *cobra.Command {
	rootCmd.PersistentPreRunE = func(cmd *cobra.Command, args []string) error {
		if err := setupLogs(os.Stderr, v); err != nil {
			return err
		}
		return nil
	}
	rootCmd.PersistentFlags().CountVarP(&v, "verbose", "v", "counted verbosity")

	return rootCmd
}

func setupLogs(out io.Writer, level int) error {
	log.Logger = log.Output(zerolog.ConsoleWriter{Out: out, TimeFormat: time.RFC3339})
	if level == 1 {
		zerolog.SetGlobalLevel(zerolog.DebugLevel)
		log.Logger = log.With().Caller().Logger()
	} else if level >= 2 {
		zerolog.SetGlobalLevel(zerolog.TraceLevel)
		log.Logger = log.With().Caller().Logger()
	} else {
		zerolog.SetGlobalLevel(zerolog.InfoLevel)
	}

	log.Trace().Msgf("Start called with verbosity %d", level)
	log.Debug().Msg("Logging debug messages")
	return nil
}
