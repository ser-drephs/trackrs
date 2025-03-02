package common

import (
	"io"
	"time"

	"github.com/rs/zerolog"
	"github.com/rs/zerolog/log"
)

func NewLogger(out io.Writer) error {
	log.Logger = log.Output(zerolog.ConsoleWriter{Out: out, TimeFormat: time.RFC3339})
	zerolog.SetGlobalLevel(zerolog.WarnLevel)
	return nil
}

func SetLoggerLevel(level int) error {
	if level == 1 {
		zerolog.SetGlobalLevel(zerolog.InfoLevel)
		log.Logger = log.With().Caller().Logger()
	} else if level == 2 {
		zerolog.SetGlobalLevel(zerolog.DebugLevel)
		log.Logger = log.With().Caller().Logger()
	} else if level > 2 {
		zerolog.SetGlobalLevel(zerolog.TraceLevel)
		log.Logger = log.With().Caller().Logger()
	} else {
		zerolog.SetGlobalLevel(zerolog.WarnLevel)
	}

	log.Trace().Msgf("Start called with verbosity %d", level)
	log.Debug().Msg("Logging debug messages")
	return nil
}
