package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainerPkg/agent_structs"

func registerSleep() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "sleep", Description: "Set sleep interval and jitter", HelpString: "sleep <seconds> [jitter%]", Version: 1,
		CommandAttributes: agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
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
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			a, _ := taskData.Args.GetStringArg("args")
			resp.DisplayParams = &a
			return resp
		},
	})
}