package main

import (
	"linky/mythic"

	"github.com/MythicMeta/MythicContainer"
)

func main() {
	mythic.Initialize()
	MythicContainer.StartAndRunForever([]MythicContainer.MythicServices{
		MythicContainer.MythicServicePayload,
	})
}
