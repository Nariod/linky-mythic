package agent_functions

import (
	"fmt"

	agentstructs "github.com/MythicMeta/MythicContainer/agent_structs"
)

func registerInject() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "inject", Description: "Inject base64 shellcode into a process (Windows)", HelpString: "inject --pid <pid> --shellcode <base64>", Version: 1,
		MitreAttackMappings: []string{"T1055"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []string{agentstructs.SUPPORTED_OS_WINDOWS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "pid", CLIName: "pid",
				ParameterType:            agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
				Description:              "Target process PID",
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{{ParameterIsRequired: true, GroupName: "Default"}},
			},
			{
				Name: "shellcode", CLIName: "shellcode",
				ParameterType:            agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Description:              "Base64-encoded shellcode payload",
				ParameterGroupInformation: []agentstructs.ParameterGroupInfo{{ParameterIsRequired: true, GroupName: "Default"}},
			},
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			pid, _ := taskData.Args.GetNumberArg("pid")
			display := fmt.Sprintf("pid=%.0f", pid)
			resp.DisplayParams = &display
			return resp
		},
	})
}
