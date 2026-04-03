package mythic

import (
	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
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
		SupportedOS:            []string{agentstructs.SUPPORTED_OS_MACOS, agentstructs.SUPPORTED_OS_LINUX, agentstructs.SUPPORTED_OS_WINDOWS},
		Wrapper:                false,
		CanBeWrappedByTheFollowingPayloadTypes: []string{},
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
			{
				Name:          "callback_uri",
				Description:   "C2 callback URI path (must match HTTP profile configuration)",
				Required:      false,
				ParameterType: agentstructs.BUILD_PARAMETER_TYPE_STRING,
				DefaultValue:  "/",
			},
		},
	}
}
