package main

import (
	"linky/mythic"

	mythicContainer "github.com/MythicMeta/MythicContainerPkg"
)

func main() {
	mythic.Initialize()
	mythicContainer.StartAndRunForever([]mythicContainer.MythicServices{
		mythicContainer.MythicServicePayload,
	})
}
