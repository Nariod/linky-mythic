package mythic

import (
	agentstructs "github.com/MythicMeta/MythicContainerPkg/agent_structs"
	"linky/mythic/agent_functions"
)

// Initialize registers the linky payload type and all its commands with Mythic.
func Initialize() {
	agentstructs.AllPayloadData.Get("linky").AddPayloadDefinition(payloadDefinition())
	agentstructs.AllPayloadData.Get("linky").AddBuildFunction(agent_functions.Build)
	agentstructs.AllPayloadData.Get("linky").AddIcon("./mythic/linky.svg")

	agent_functions.RegisterAllCommands()
}

func payloadDefinition() agentstructs.PayloadType {
	return agentstructs.PayloadType{
		Name:                   "linky",
		FileExtension:          "bin",
		Author:                 "@your-handle",
		SupportedOS:            []agentstructs.OS{agentstructs.MACOS, agentstructs.LINUX, agentstructs.WINDOWS},
		Wrapper:                false,
		CanBeWrappedBy:         []string{},
		SupportsDynamicLoading: false,
		Description:            "Rust-native cross-platform C2 agent. Minimal, auditable, container-first.",
		SupportedC2Profiles:    []string{"http"},
		MythicEncryptsData:     true,
		BuildParameters: []agentstructs.BuildParameter{
			{
				Name:          "target_os",
				Description:   "Target operating system",
				Required:      true,
				ParameterType: agentstructs.BUILD_PARAMETER_TYPE_CHOOSE_ONE,
				Choices:       []string{"linux", "windows", "macos"},
				DefaultValue:  "linux",
			},
			{
				Name:          "shellcode",
				Description:   "Export as shellcode (.bin via objcopy — Linux only)",
				Required:      false,
				ParameterType: agentstructs.BUILD_PARAMETER_TYPE_BOOLEAN,
				DefaultValue:  false,
			},
			{
				Name:          "debug",
				Description:   "Debug build (slower, larger, with symbols)",
				Required:      false,
				ParameterType: agentstructs.BUILD_PARAMETER_TYPE_BOOLEAN,
				DefaultValue:  false,
			},
		},
	}
}
