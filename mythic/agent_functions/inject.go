package agent_functions

import (
	"fmt"

	agentstructs "github.com/MythicMeta/MythicContainerPkg/agent_structs"
)

func registerInject() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "inject", Description: "Inject base64 shellcode into a process (Windows)", HelpString: "inject --pid <pid> --shellcode <base64>", Version: 1,
		MitreAttackMappings: []string{"T1055"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.WINDOWS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "pid", CLIName: "pid",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_NUMBER,
				Required:      true,
				Description:   "Target process PID",
			},
			{
				Name: "shellcode", CLIName: "shellcode",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Required:      true,
				Description:   "Base64-encoded shellcode payload",
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
