package agent_functions

import agentstructs "github.com/MythicMeta/MythicContainerPkg/agent_structs"

func registerCd() {
	agentstructs.AllPayloadData.Get("linky").AddCommand(agentstructs.Command{
		Name: "cd", Description: "Change working directory", HelpString: "cd <path>", Version: 1,
		CommandAttributes: agentstructs.CommandAttribute{SupportedOS: []agentstructs.OS{agentstructs.LINUX, agentstructs.WINDOWS, agentstructs.MACOS}},
		CommandParameters: []agentstructs.CommandParameter{
			{
				Name: "path", CLIName: "path",
				ParameterType: agentstructs.COMMAND_PARAMETER_TYPE_STRING,
				Required:      true,
			},
		},
		TaskFunctionParseArgString: func(args *agentstructs.PTTaskMessageArgsData, input string) error {
			return args.SetArgValue("path", input)
		},
		TaskFunctionCreateTasking: func(taskData *agentstructs.PTTaskMessageAllData) agentstructs.PTTaskCreateTaskingMessageResponse {
			resp := agentstructs.PTTaskCreateTaskingMessageResponse{TaskID: taskData.Task.ID, Success: true}
			path, _ := taskData.Args.GetStringArg("path")
			resp.DisplayParams = &path
			return resp
		},
	})
}