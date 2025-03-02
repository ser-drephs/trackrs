package main

import (
	"fmt"
	"os"

	"github.com/ser-drephs/tracker-go/cli"
	"github.com/ser-drephs/tracker-go/common"
)

func main() {
	if err := common.NewLogger(os.Stderr); err != nil {
		fmt.Printf("Error on logger initialization: %s", err)
	}
	cli.Execute()
}
