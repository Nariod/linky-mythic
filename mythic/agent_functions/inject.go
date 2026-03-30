package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainerPkg/agent_structs"

func registerInject() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "inject", Description: "Inject base64 shellcode into PID (Windows)", HelpString: "inject <pid> <base64_shellcode>", Version: 1,
		MitreAttackMappings: []string{"T1055"},
		CommandAttributes:   agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.WINDOWS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "args", CLIName: "args",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Required:      true,
			},
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			return args.SetArgValue("args", input)
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			return agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
		},
	})
}